// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Host-side notification recorder.
//!
//! When contracts call `NeoRuntime::notify(event, state)` on the host
//! (i.e. not on the wasm32 path), the call is recorded here so tests
//! can assert what was emitted. The wasm32 path emits to the Neo VM
//! host directly via `runtime_notify_with_state`; for host tests we
//! mirror the event into this recorder so the test surface is
//! identical regardless of target.
//!
//! This module is host-only (`cfg(not(target_arch = "wasm32"))`) — on
//! the real VM, the engine itself records notifications in
//! `ApplicationEngine.cs::notifications`.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;

#[cfg(not(target_arch = "wasm32"))]
use neo_types::{NeoArray, NeoString, NeoValue};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct RecordedNotification {
    pub event: String,
    pub state: Vec<NeoValue>,
}

#[cfg(not(target_arch = "wasm32"))]
static RECORDED: Lazy<Mutex<Vec<RecordedNotification>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Record a notification. Called from `NeoVMSyscall::notify` on
/// both wasm32 and host paths so the recorder is consistent.
#[cfg(not(target_arch = "wasm32"))]
pub fn record(event: &NeoString, state: &NeoArray<NeoValue>) {
    let mut g = RECORDED.lock().expect("notification recorder poisoned");
    g.push(RecordedNotification {
        event: event.as_str().to_string(),
        state: state.iter().cloned().collect(),
    });
}

/// Take all recorded notifications (drains the buffer).
pub fn take() -> Vec<RecordedNotification> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut g = RECORDED.lock().expect("notification recorder poisoned");
        std::mem::take(&mut *g)
    }
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
}

/// Clear the recorder without returning the contents.
pub fn reset() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut g = RECORDED.lock().expect("notification recorder poisoned");
        g.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neo_types::{NeoBoolean, NeoInteger};

    #[test]
    fn record_and_take_round_trip() {
        reset();
        let evt = NeoString::from_str("Transfer");
        let mut state = NeoArray::new();
        state.push(NeoValue::Integer(NeoInteger::new(100i32)));
        state.push(NeoValue::Boolean(NeoBoolean::new(true)));
        record(&evt, &state);
        let got = take();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].event, "Transfer");
        assert_eq!(got[0].state.len(), 2);
    }
}
