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
