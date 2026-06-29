// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! B5–B9: host-mode state for runtime syscalls.
//!
//! The v0.7.0 audit identified B5–B9 as TIER 2 silent-wrong-
//! value bugs. The wasm32 path is correct (each wrapper calls
//! an extern); the host-mode test framework returns 0/empty
//! because the host has no state. v0.13.0 fixes the host-side
//! state so tests can configure and assert real values.
//!
//! Each test: set host state via the public setter, call the
//! syscall via `neovm_syscall`, assert the return value. Each
//! test cleans up via `reset_host_state()`.

use neo_devpack::prelude::*;
use neo_devpack::NeoVMSyscall;
use std::sync::{Mutex, MutexGuard};

/// Host-mode state (`ACTIVE_TIME`, `ACTIVE_RANDOM`, …) lives in
/// process-global statics, so these tests must not interleave:
/// one test's `reset_all()` would otherwise clobber another's
/// just-set value. Each test holds this guard for its whole body
/// to run serially regardless of `--test-threads`. A poisoned
/// lock (from a prior panic) is recovered so one failure does not
/// cascade into the rest of the suite.
static HOST_STATE_GUARD: Mutex<()> = Mutex::new(());

fn serial_guard() -> MutexGuard<'static, ()> {
    HOST_STATE_GUARD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn call_syscall(name: &str, args: &[NeoValue]) -> NeoResult<NeoValue> {
    let info = neo_devpack::SYSCALL_REGISTRY
        .get_syscall(name)
        .expect("known syscall");
    neo_devpack::neovm_syscall(info.hash, args)
}

fn reset_all() {
    let _ = NeoVMSyscall::reset_host_state();
    neo_devpack::reset_recorded_notifications();
}

#[test]
fn b5_get_random_returns_active_value() {
    let _guard = serial_guard();
    reset_all();
    NeoVMSyscall::set_active_random(0xDEAD_BEEF_CAFE_BABE_u64 as i64).expect("set random");
    let v = call_syscall("System.Runtime.GetRandom", &[]).expect("get_random");
    match v {
        NeoValue::Integer(i) => {
            assert_eq!(i.try_as_i64(), Some(0xDEAD_BEEF_CAFE_BABE_u64 as i64));
        }
        other => panic!("expected Integer, got {other:?}"),
    }
    reset_all();
}

#[test]
fn b6_get_time_returns_active_value() {
    let _guard = serial_guard();
    reset_all();
    NeoVMSyscall::set_active_time(1_700_000_000_000).expect("set time");
    let v = call_syscall("System.Runtime.GetTime", &[]).expect("get_time");
    match v {
        NeoValue::Integer(i) => {
            assert_eq!(i.try_as_i64(), Some(1_700_000_000_000));
        }
        other => panic!("expected Integer, got {other:?}"),
    }
    reset_all();
}

#[test]
fn b6_invocation_counter_returns_active_value() {
    let _guard = serial_guard();
    reset_all();
    NeoVMSyscall::set_active_invocation_counter(42).expect("set counter");
    let v =
        call_syscall("System.Runtime.GetInvocationCounter", &[]).expect("get_invocation_counter");
    match v {
        NeoValue::Integer(i) => {
            assert_eq!(i.try_as_i64(), Some(42));
        }
        other => panic!("expected Integer, got {other:?}"),
    }
    reset_all();
}

#[test]
fn b7_get_gas_left_returns_active_value() {
    let _guard = serial_guard();
    reset_all();
    NeoVMSyscall::set_active_gas_left(1_000_000).expect("set gas");
    let v = call_syscall("System.Runtime.GasLeft", &[]).expect("gas_left");
    match v {
        NeoValue::Integer(i) => {
            assert_eq!(i.try_as_i64(), Some(1_000_000));
        }
        other => panic!("expected Integer, got {other:?}"),
    }
    reset_all();
}

#[test]
fn b8_current_signers_returns_active_signers() {
    let _guard = serial_guard();
    reset_all();
    // Signers are recorded via the existing
    // `set_active_witnesses` mechanism. For B8, the host
    // dispatch should return one signer entry per active
    // witness.
    NeoVMSyscall::set_active_witnesses(&[NeoByteString::from_slice(&[0xAA; 20])])
        .expect("set witness");
    let v = call_syscall("System.Runtime.CurrentSigners", &[]).expect("current_signers");
    match v {
        NeoValue::Array(a) => {
            assert_eq!(a.len(), 1, "expected 1 signer, got array len");
        }
        other => panic!("expected Array, got {other:?}"),
    }
    reset_all();
}

#[test]
fn b9_get_notifications_returns_recorded_notifications() {
    let _guard = serial_guard();
    reset_all();
    // Record a notification via the existing
    // `record_notification` path. The B9 dispatch should
    // return it.
    let event_name = NeoString::from_str("Transfer");
    let state: NeoArray<NeoValue> = NeoArray::new();
    neo_devpack::record_notification(&event_name, &state);
    // The arg is a script hash filter; pass Null to mean
    // "all notifications".
    let v = call_syscall("System.Runtime.GetNotifications", &[NeoValue::Null])
        .expect("get_notifications");
    match v {
        NeoValue::Array(a) => {
            assert_eq!(
                a.len(),
                1,
                "expected 1 recorded notification, got array len"
            );
        }
        other => panic!("expected Array, got {other:?}"),
    }
    reset_all();
}
