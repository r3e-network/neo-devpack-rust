// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT
//! Stateful e2e: storage round-trips, seeded storage, and runtime env on a
//! real NeoVM (the syscall-handling oracle) — not just pure compute.
use neo_vm_test::Contract;

#[test]
fn storage_round_trip_on_real_vm() {
    let c = Contract::compile("contracts/feature-storage-raw").expect("compile");
    // put_i64 then get_i64 of the same key -> the value persists through storage.
    let out = c.invoke("roundI64", &[12345.into()]);
    out.assert_returns_i64(12345);
    assert!(!out.storage_diff.is_empty(), "a storage entry should be written");
}

#[test]
fn seeded_storage_is_visible() {
    let c = Contract::compile("contracts/feature-storage-raw").expect("compile");
    // Seed the integer key 7's slot, then read it back via get_i64_key_or_zero.
    // First discover the on-chain key encoding by writing it, then assert read.
    let written = c.call("setKeyed").arg(7).arg(999).run();
    written.assert_halt();
    let (key, val) = written.storage_diff[0].clone();
    // Now a fresh call with that storage seeded must read 999.
    c.call("getKeyed")
        .arg(7)
        .storage(&key, &val)
        .run()
        .assert_returns_i64(999);
}

#[test]
fn runtime_time_is_honored() {
    let c = Contract::compile("contracts/feature-runtime").expect("compile");
    c.call("nowMs").time(1_700_000_000_000).run().assert_returns_i64(1_700_000_000_000);
}

#[test]
fn heap_built_syscall_args_dont_fault() {
    // Regression: a heap-built NeoString/NeoByteString passed to log/check_witness
    // must marshal correctly out of CHUNKED memory (previously FAULTed at SUBSTR).
    let c = Contract::compile("contracts/feature-runtime").expect("compile");
    c.invoke("logMsg", &[]).assert_halt(); // NeoRuntime::log(&NeoString::from_str(..))
    c.invoke("witnessBytes", &[]).assert_returns_bool(false); // check_witness(&NeoByteString)/bytes, no signer
}

#[test]
fn event_payloads_round_trip_on_real_vm() {
    // Flagship wasm32-bridge gap, now wired: `#[neo_event]::emit()` reaches
    // `runtime_notify_with_state`, whose lowering decodes the serialised
    // state on-VM (StdLib.deserialize) and notifies with the real payload.
    // Both the event NAME and every FIELD VALUE must round-trip exactly.
    let c = Contract::compile("contracts/feature-events-manifest").expect("compile");

    // Transfer { from: [0u8;20], to: [1u8;20], amount } — the NEP-17 shape.
    let out = c.invoke("fireTransfer", &[1000.into()]);
    out.assert_halt();
    let ev = out.event("Transfer").expect("Transfer event emitted");
    assert_eq!(
        ev.state,
        vec![
            "00".repeat(20), // from: ByteString -> hex
            "01".repeat(20), // to
            "1000".to_string(), // amount: Integer -> decimal
        ],
        "Transfer payload must round-trip"
    );

    // Single Integer field, including a negative value (little-endian
    // two's-complement in the serialised state).
    let ev_state = |seq: i64| {
        let out = c.invoke("firePing", &[seq.into()]);
        out.assert_halt();
        out.event("Ping").expect("Ping event emitted").state.clone()
    };
    assert_eq!(ev_state(7), vec!["7"]);
    assert_eq!(ev_state(-123_456_789), vec!["-123456789"]);
}

#[test]
fn mixed_type_notify_state_round_trips_on_real_vm() {
    // NeoRuntime::notify called directly with Boolean + ByteString + Integer.
    let c = Contract::compile("contracts/feature-events-manifest").expect("compile");
    let out = c.invoke("rawNotify", &[true.into(), 987_654_321.into()]);
    out.assert_halt();
    let ev = out.event("Mixed").expect("Mixed event emitted");
    assert_eq!(
        ev.state,
        vec![
            "true".to_string(),
            "7061796c6f6164".to_string(), // hex of b"payload"
            "987654321".to_string(),
        ]
    );
}

#[test]
fn name_only_notify_carries_empty_state_on_real_vm() {
    // Regression: the name-only notify lowering pushed the empty state array
    // ON TOP of the name, so `System.Runtime.Notify` popped the Array as the
    // event name and FAULTed ("invalid conversion: Array/ByteString").
    let c = Contract::compile("contracts/feature-events-manifest").expect("compile");
    let out = c.invoke("rawNotifyEvent", &[]);
    out.assert_halt();
    let ev = out.event("Started").expect("Started event emitted");
    assert!(ev.state.is_empty(), "name-only notify has an empty state");
}

#[test]
fn storage_find_prefix_scan_on_real_vm() {
    // Prefix-scan bridge gap, now wired: `NeoStorage::find` on wasm32 lowers
    // to `System.Storage.Find` + `System.Iterator.Next/Value` (single live
    // iterator parked in a static slot; entries drained eagerly).
    //
    // `scanPrefix` puts fp:a=10, fp:b=20, fp:c=30 plus zz=99 (outside the
    // prefix), scans "fp:", and folds positionally:
    //   3 * 10_000 + (1*10 + 2*20 + 3*30) = 30_140.
    // The positional weighting pins the ascending-key element ORDER; the
    // count pins that zz is excluded; the key check inside the contract pins
    // that keys carry the full prefix (FindOptions.None). The host half of
    // this agreement is rust-devpack/tests/storage_find_host.rs, which must
    // compute the same 30_140.
    let c = Contract::compile("contracts/feature-storage-typed").expect("compile");
    let out = c.invoke("scanPrefix", &[]);
    out.assert_halt();
    out.assert_returns_i64(30_140);
    // All four writes must land in storage (the scan itself is read-only).
    out.assert_storage(b"fp:a", &10i64.to_le_bytes());
    out.assert_storage(b"fp:b", &20i64.to_le_bytes());
    out.assert_storage(b"fp:c", &30i64.to_le_bytes());
    out.assert_storage(b"zz", &99i64.to_le_bytes());
}

#[test]
fn storage_find_value_buffer_growth_on_real_vm() {
    // The SDK drain loop starts with a 128-byte buffer; `scanBigValue`'s
    // 200-byte value makes the flattened element 208 bytes, so the first
    // `runtime_iterator_value` call must return `-needed_len` WITHOUT
    // advancing the iterator and the retry must deliver the full payload:
    //   200 * 100_000 + Σ(i % 251 for i in 0..200) = 20_019_900.
    // A lowering that advanced on the too-small path would lose the entry
    // (-4 sentinel); a truncated payload would break the checksum.
    let c = Contract::compile("contracts/feature-storage-typed").expect("compile");
    c.invoke("scanBigValue", &[]).assert_returns_i64(20_019_900);
}

#[test]
fn storage_find_empty_prefix_counts_seeded_entries_on_real_vm() {
    // Empty-prefix full scan over seeded storage: the previously-documented
    // "always 0 on wasm32" limitation of `find_count` is gone.
    let c = Contract::compile("contracts/feature-storage-typed").expect("compile");
    c.call("findCount")
        .storage(b"a", b"1")
        .storage(b"b", b"2")
        .storage(b"c", b"3")
        .run()
        .assert_returns_i64(3);
}
