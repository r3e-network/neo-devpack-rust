// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Host-mode half of the storage prefix-scan agreement contract.
//!
//! `NeoStorage::find` uses the same entry shape on every target:
//! `Struct{ key (incl. prefix), value }` in ascending key order
//! (`FindOptions.None` — the only option the wasm32 bridge emits). The
//! real-VM half of this agreement lives in `neo-vm-test/tests/stateful.rs`
//! (`storage_find_prefix_scan_on_real_vm` et al.), which drives
//! `contracts/feature-storage-typed::scan_prefix` through the neo-go oracle
//! and asserts the exact same folded value computed here — if either side
//! drifts, one of the two suites fails.

use std::sync::Mutex;

use neo_devpack::prelude::*;
use neo_devpack::NeoVMSyscall;

// Host storage/iterator state is process-global; serialise the tests in
// this binary (the same pattern as `b5_b9_host_state.rs`).
static HOST_STATE_GUARD: Mutex<()> = Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    HOST_STATE_GUARD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn reset() {
    NeoVMSyscall::reset_host_state().expect("host state resets");
}

fn put(ctx: &NeoStorageContext, key: &[u8], value: &[u8]) {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(value),
    )
    .expect("put");
}

/// Mirrors `contracts/feature-storage-typed::scan_prefix` exactly: 3 keys
/// under `fp:` plus one outside, scan the prefix, fold positionally. The
/// real-VM test asserts the same 30_140 for the same contract logic.
#[test]
fn scan_prefix_agrees_with_real_vm() {
    let _guard = lock();
    reset();

    let ctx = NeoStorage::get_context().expect("context");
    put(&ctx, b"fp:a", &10i64.to_le_bytes());
    put(&ctx, b"fp:b", &20i64.to_le_bytes());
    put(&ctx, b"fp:c", &30i64.to_le_bytes());
    put(&ctx, b"zz", &99i64.to_le_bytes());

    let iter = NeoStorage::find(&ctx, &NeoByteString::from_slice(b"fp:")).expect("find");
    let mut count: i64 = 0;
    let mut weighted: i64 = 0;
    for entry in iter {
        let st = entry.as_struct().cloned().expect("Struct entry");
        let key = st
            .get_field("key")
            .and_then(NeoValue::as_byte_string)
            .cloned()
            .expect("key field");
        assert!(
            key.as_slice().starts_with(b"fp:"),
            "FindOptions.None keys carry the full prefix"
        );
        let value = st
            .get_field("value")
            .and_then(NeoValue::as_byte_string)
            .cloned()
            .expect("value field");
        let mut b = [0u8; 8];
        b.copy_from_slice(value.as_slice());
        count += 1;
        weighted += count * i64::from_le_bytes(b);
    }

    // 3 * 10_000 + (1*10 + 2*20 + 3*30): the weighting pins ascending key
    // order, the count pins that `zz` is excluded. Must match the real-VM
    // `scanPrefix` return value in neo-vm-test/tests/stateful.rs.
    assert_eq!(count * 10_000 + weighted, 30_140);

    reset();
}

/// Mirrors `contracts/feature-storage-typed::scan_big_value`: one 200-byte
/// value under the prefix (on wasm32 this forces the drain loop's
/// `-needed_len` grow-and-retry path; the host must fold to the same
/// 20_019_900 the real-VM test asserts).
#[test]
fn scan_big_value_agrees_with_real_vm() {
    let _guard = lock();
    reset();

    let ctx = NeoStorage::get_context().expect("context");
    let mut big = [0u8; 200];
    for (i, b) in big.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    put(&ctx, b"bp:k", &big);

    let mut iter = NeoStorage::find(&ctx, &NeoByteString::from_slice(b"bp:")).expect("find");
    let entry = iter.next().expect("one entry");
    assert!(iter.next().is_none(), "exactly one entry under the prefix");
    let value = entry
        .as_struct()
        .and_then(|st| st.get_field("value"))
        .and_then(NeoValue::as_byte_string)
        .cloned()
        .expect("value field");
    let sum: i64 = value.as_slice().iter().map(|b| i64::from(*b)).sum();

    // 200 * 100_000 + 19_900: must match the real-VM `scanBigValue` return
    // value in neo-vm-test/tests/stateful.rs.
    assert_eq!((value.len() as i64) * 100_000 + sum, 20_019_900);

    reset();
}

/// Mirrors `feature-storage-typed::find_count` with 3 seeded entries (the
/// real-VM test seeds a/b/c via the harness and expects 3).
#[test]
fn empty_prefix_full_scan_counts_all_entries() {
    let _guard = lock();
    reset();

    let ctx = NeoStorage::get_context().expect("context");
    put(&ctx, b"a", b"1");
    put(&ctx, b"b", b"2");
    put(&ctx, b"c", b"3");

    let iter = NeoStorage::find(&ctx, &NeoByteString::from_slice(b"")).expect("find");
    assert_eq!(iter.count(), 3);

    reset();
}

/// Dispatch-level Find -> Next -> Value round trip: the host session arms
/// (`System.Storage.Find` seeds the single live session,
/// `System.Iterator.Next`/`Value` walk it) must follow the real VM's
/// iterator protocol, including the fault past the end.
#[test]
fn find_next_value_syscall_round_trip() {
    let _guard = lock();
    reset();

    let ctx = NeoStorage::get_context().expect("context");
    put(&ctx, b"it:x", b"left");
    put(&ctx, b"it:y", b"right");
    put(&ctx, b"other", b"skip");

    let find = neo_devpack::SYSCALL_REGISTRY
        .get_syscall("System.Storage.Find")
        .expect("registered");
    neo_devpack::neovm_syscall(
        find.hash,
        &[
            NeoValue::Null, // context is carried out-of-band on the host
            NeoValue::from(NeoByteString::from_slice(b"it:")),
            NeoValue::from(NeoInteger::new(0)), // FindOptions.None
        ],
    )
    .expect("find seeds the session");

    let entries = NeoArray::<NeoValue>::new(); // session-based: array unused
    let mut seen: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    while NeoVMSyscall::iterator_next(&entries)
        .expect("next")
        .as_bool()
    {
        let entry = NeoVMSyscall::iterator_value(&entries).expect("value");
        let st = entry.as_struct().cloned().expect("Struct entry");
        let key = st
            .get_field("key")
            .and_then(NeoValue::as_byte_string)
            .cloned()
            .expect("key");
        let value = st
            .get_field("value")
            .and_then(NeoValue::as_byte_string)
            .cloned()
            .expect("value");
        seen.push((key.as_slice().to_vec(), value.as_slice().to_vec()));
    }
    assert_eq!(
        seen,
        vec![
            (b"it:x".to_vec(), b"left".to_vec()),
            (b"it:y".to_vec(), b"right".to_vec()),
        ],
        "ascending key order, prefix kept, non-matching keys excluded"
    );

    // Past the end: Next stays false, Value faults (like the real VM).
    assert!(!NeoVMSyscall::iterator_next(&entries).expect("next").as_bool());
    assert!(NeoVMSyscall::iterator_value(&entries).is_err());

    reset();
}
