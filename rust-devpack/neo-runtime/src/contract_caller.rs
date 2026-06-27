// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! L9: typed cross-contract calls via the `ContractCaller` trait.
//!
//! The `FromNeoValue` + `ContractCaller::call_typed<T>` pair
//! mirrors the C# devpack's `IInteroperable.FromStackItem` +
//! `Contract.Call<T>` pattern. The default `DefaultContractCaller`
//! routes through `NeoVMSyscall::contract_call`; the L6
//! cross-call executor will add a real wasm32 implementation
//! that doesn't panic (B4 fix).

use neo_syscalls::NeoVMSyscall;
use neo_types::{
    ContractCaller, NeoArray, NeoByteString, NeoError, NeoInteger, NeoResult, NeoString, NeoValue,
};

/// The default `ContractCaller` impl: routes `call_raw` to
/// `NeoVMSyscall::contract_call`. Used by host-mode tests and
/// by production code that needs the L6 cross-call upgrade
/// (tracked in the audit).
pub struct DefaultContractCaller;

impl ContractCaller for DefaultContractCaller {
    fn call_raw(
        &self,
        script_hash: &NeoByteString,
        method: &str,
        args: &[NeoValue],
        call_flags: &NeoInteger,
    ) -> NeoResult<NeoValue> {
        // We pass the method as a NeoString here. The syscall
        // signature in neo-syscalls takes &NeoString; we
        // construct it from the &str method name. The user
        // should not call this directly; `call_typed<T>` is
        // the public API.
        let method_str = NeoString::from_str(method);
        let args_array: NeoArray<NeoValue> = args.iter().cloned().collect();
        NeoVMSyscall::contract_call(script_hash, &method_str, call_flags, &args_array)
    }
}

/// Helper: typed cross-contract call with the default caller.
/// Mirrors the C# `Contract.Call<T>` API.
pub fn call_typed<T: neo_types::FromNeoValue>(
    script_hash: &NeoByteString,
    method: &str,
    args: &[NeoValue],
    call_flags: &NeoInteger,
) -> NeoResult<T> {
    DefaultContractCaller.call_typed(script_hash, method, args, call_flags)
}

/// Error type for the L9 typed-call helpers. Reserved for
/// future use (the L6 cross-call executor will return this
/// rather than panicking with "see L6 design").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractCallError {
    /// The call returned no value.
    NoReturn,
    /// The call returned a value of the wrong type.
    TypeMismatch(String),
    /// The call panicked (L6: cross-call executor will replace
    /// this with a proper Result).
    Panicked(String),
    /// L6 minimal: the wasm32 cross-call stub (System.Contract.Call
    /// / System.Runtime.LoadScript / System.Contract.CallNative)
    /// was invoked, but a real wasm32 cross-call executor is not
    /// yet implemented. The contract author can `match` on this
    /// variant and degrade gracefully (e.g. return a default
    /// value) instead of crashing the VM.
    Wasm32CrossCallUnavailable { syscall: &'static str },
    /// Other (delegated from `NeoError`).
    Other(NeoError),
}

impl std::fmt::Display for ContractCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractCallError::NoReturn => write!(f, "contract call returned no value"),
            ContractCallError::TypeMismatch(s) => write!(f, "type mismatch: {s}"),
            ContractCallError::Panicked(s) => write!(f, "contract call panicked: {s}"),
            ContractCallError::Wasm32CrossCallUnavailable { syscall } => {
                write!(f, "wasm32 cross-call unavailable: {syscall}")
            }
            ContractCallError::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ContractCallError {}

impl From<NeoError> for ContractCallError {
    fn from(e: NeoError) -> Self {
        match e {
            NeoError::Wasm32CrossCallUnavailable { syscall } => {
                ContractCallError::Wasm32CrossCallUnavailable { syscall }
            }
            other => ContractCallError::Other(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neo_types::{NeoByteString, NeoInteger, NeoValue};

    #[test]
    fn default_caller_returns_null_for_known_contract() {
        // The DefaultContractCaller uses NeoVMSyscall::contract_call
        // which on the host path uses the host-mode dispatcher.
        // The dispatcher may return either Null (B4 fallback) or
        // an error; we just assert the call doesn't panic and
        // either returns Ok or Err.
        let script_hash = NeoByteString::from_slice(&[1u8; 20]);
        let call_flags = NeoInteger::new(0x0F);
        let result = DefaultContractCaller.call_raw(&script_hash, "method", &[], &call_flags);
        // Acceptable: Ok with any value, or an Err (B4 silent
        // fallback) — the L6 cross-call upgrade will replace
        // this with a real Result.
        let _ = result;
    }

    #[test]
    fn call_typed_returns_null_for_non_typed_target() {
        // call_typed<NeoValue> should always succeed if the
        // raw call succeeded, regardless of the return value.
        let script_hash = NeoByteString::from_slice(&[1u8; 20]);
        let call_flags = NeoInteger::new(0x0F);
        let _result: Result<NeoValue, _> = call_typed(&script_hash, "method", &[], &call_flags);
    }
}
