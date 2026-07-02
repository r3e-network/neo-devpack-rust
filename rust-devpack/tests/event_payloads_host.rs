// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Host-mode half of the event-payload agreement contract.
//!
//! `#[neo_event]::emit()` and `NeoRuntime::notify` use ONE state-carrying
//! path on every target. The real-VM half of this agreement lives in
//! `neo-vm-test/tests/stateful.rs` (`event_payloads_round_trip_on_real_vm`
//! et al.), which drives `contracts/feature-events-manifest` through the
//! neo-go oracle and asserts the exact same event names and payload values
//! asserted here against the host recorder — if either side drifts, one of
//! the two suites fails.

use std::sync::Mutex;

use neo_devpack::prelude::*;
use neo_devpack::{reset_recorded_notifications, take_recorded_notifications};

/// Same shape as `contracts/feature-events-manifest`'s `Transfer`.
#[neo_event]
pub struct Transfer {
    pub from: NeoByteString,
    pub to: NeoByteString,
    pub amount: NeoInteger,
}

/// Same shape as `contracts/feature-events-manifest`'s `Ping`.
#[neo_event]
pub struct Ping {
    pub seq: NeoInteger,
}

// The notification recorder is process-global; serialise the tests in this
// binary (the same pattern as `b5_b9_host_state.rs`).
static RECORDER_GUARD: Mutex<()> = Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    RECORDER_GUARD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn expect_bytes(value: &NeoValue, expected: &[u8]) {
    match value {
        NeoValue::ByteString(bs) => assert_eq!(bs.as_slice(), expected),
        other => panic!("expected ByteString, got {other:?}"),
    }
}

fn expect_int(value: &NeoValue, expected: i64) {
    match value {
        NeoValue::Integer(i) => assert_eq!(i.try_as_i64(), Some(expected)),
        other => panic!("expected Integer, got {other:?}"),
    }
}

#[test]
fn emit_records_transfer_payload_on_host() {
    let _guard = lock();
    reset_recorded_notifications();

    // Mirrors `fireTransfer(1000)` in feature-events-manifest; the real-VM
    // test asserts state ["00"*20, "01"*20, "1000"] for the same emission.
    Transfer {
        from: NeoByteString::from_slice(&[0u8; 20]),
        to: NeoByteString::from_slice(&[1u8; 20]),
        amount: NeoInteger::new(1000i64),
    }
    .emit()
    .expect("emit");

    let got = take_recorded_notifications();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].event, "Transfer");
    assert_eq!(got[0].state.len(), 3);
    expect_bytes(&got[0].state[0], &[0u8; 20]);
    expect_bytes(&got[0].state[1], &[1u8; 20]);
    expect_int(&got[0].state[2], 1000);
}

#[test]
fn emit_records_negative_integer_payload_on_host() {
    let _guard = lock();
    reset_recorded_notifications();

    // Mirrors `firePing(-123456789)`; real-VM state: ["-123456789"].
    Ping {
        seq: NeoInteger::new(-123_456_789i64),
    }
    .emit()
    .expect("emit");

    let got = take_recorded_notifications();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].event, "Ping");
    assert_eq!(got[0].state.len(), 1);
    expect_int(&got[0].state[0], -123_456_789);
}

#[test]
fn direct_notify_records_mixed_payload_on_host() {
    let _guard = lock();
    reset_recorded_notifications();

    // Mirrors `rawNotify(true, 987654321)`; real-VM state:
    // ["true", hex(b"payload"), "987654321"].
    let name = NeoString::from_str("Mixed");
    let mut state = NeoArray::new();
    state.push(NeoValue::from(NeoBoolean::new(true)));
    state.push(NeoValue::from(NeoByteString::from_slice(b"payload")));
    state.push(NeoValue::from(NeoInteger::new(987_654_321i64)));
    NeoRuntime::notify(&name, &state).expect("notify");

    let got = take_recorded_notifications();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].event, "Mixed");
    assert_eq!(got[0].state.len(), 3);
    match &got[0].state[0] {
        NeoValue::Boolean(b) => assert!(b.as_bool()),
        other => panic!("expected Boolean, got {other:?}"),
    }
    expect_bytes(&got[0].state[1], b"payload");
    expect_int(&got[0].state[2], 987_654_321);
}

#[test]
fn name_only_notify_records_empty_state_on_host() {
    let _guard = lock();
    reset_recorded_notifications();

    // Mirrors `rawNotifyEvent()`; real-VM state: [].
    NeoRuntime::notify_event("Started").expect("notify_event");

    let got = take_recorded_notifications();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].event, "Started");
    assert!(got[0].state.is_empty());
}
