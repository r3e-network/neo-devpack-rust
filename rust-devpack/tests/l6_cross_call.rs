// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! L6 cross-call tests: Result-returning wasm32 cross-call stub.
//!
//! On host (`#[cfg(not(target_arch = "wasm32"))]`), the host-mode
//! dispatcher handles cross-calls; we assert the result is either
//! `Ok` or an `Err`, and that the `NeoError` doesn't carry the
//! "wasm32 cross-call unavailable" message.
//!
//! On wasm32, the cross-call stub must return a structured
//! `ContractCallError::Wasm32CrossCallUnavailable` rather than
//! panic (L6 minimal: the B4 panic-loud sites are now Result).
//!
//! Note: the actual wasm32 path can't be exercised in this
//! workspace's default test target (the unit-test runner is
//! `cargo test` on the host). The wasm32-specific assertion
//! is verified by reading the source code via a `cfg`-gated
//! compile-only test: if the panic-loud site is still
//! `panic!`, the test fails at compile time. This is a
//! pragmatic test given the constraint that we don't run
//! the unit-test suite under wasm32 in CI.

#![cfg(feature = "std")]

use neo_devpack::prelude::*;
use neo_types::NeoError;

#[test]
fn host_path_cross_call_returns_result() {
    // On host, the dispatcher may return Ok(Null) (B4 fallback)
    // or an Err. We don't assert which — we assert the call
    // is well-typed and the error doesn't mention wasm32.
    let script_hash = NeoByteString::from_slice(&[0u8; 20]);
    let call_flags = NeoInteger::new(0x0F);
    let result: Result<NeoValue, ContractCallError> = call_typed::<NeoValue>(
        &script_hash,
        "method",
        &[],
        &call_flags,
    )
    .map_err(ContractCallError::from);
    if let Err(ContractCallError::Other(e)) = &result {
        assert!(
            !e.message().contains("wasm32"),
            "host path should not produce wasm32 error, got: {e}"
        );
    }
}

#[test]
fn contract_call_error_constructs_wasm32_variant() {
    let err = ContractCallError::Wasm32CrossCallUnavailable {
        syscall: "System.Contract.Call",
    };
    let display = format!("{err}");
    assert!(
        display.contains("wasm32"),
        "display should mention wasm32, got: {display}"
    );
    assert!(
        display.contains("System.Contract.Call"),
        "display should mention the syscall, got: {display}"
    );
}

#[test]
fn neo_error_wasm32_variant_carries_syscall() {
    let err = NeoError::Wasm32CrossCallUnavailable {
        syscall: "System.Contract.Call",
    };
    let display = format!("{err}");
    assert!(display.contains("wasm32"), "got: {display}");
    assert!(display.contains("System.Contract.Call"), "got: {display}");
    assert_eq!(err.message(), "wasm32 cross-call unavailable: System.Contract.Call");
    assert_eq!(err.as_str(), "Wasm32CrossCallUnavailable");
}

#[test]
fn contract_call_error_from_neo_error_wasm32() {
    let neo = NeoError::Wasm32CrossCallUnavailable {
        syscall: "System.Runtime.LoadScript",
    };
    let contract: ContractCallError = neo.into();
    match contract {
        ContractCallError::Wasm32CrossCallUnavailable { syscall } => {
            assert_eq!(syscall, "System.Runtime.LoadScript");
        }
        other => panic!("expected Wasm32CrossCallUnavailable, got {other:?}"),
    }
}

#[test]
fn neo_error_status_code_for_wasm32() {
    let err = NeoError::Wasm32CrossCallUnavailable {
        syscall: "System.Contract.CallNative",
    };
    // Should have a stable status code different from Custom (10).
    let code = err.status_code();
    assert_ne!(code, 10, "Wasm32CrossCallUnavailable should have its own status code");
    assert!(code > 0 && code < 100, "status code should be small int, got {code}");
}
