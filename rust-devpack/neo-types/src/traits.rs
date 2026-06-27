// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use crate::error::NeoResult;
use crate::value::NeoValue;

/// Neo N3 Contract trait
pub trait NeoContract {
    fn name() -> &'static str;
    fn version() -> &'static str;
    fn author() -> &'static str;
    fn description() -> &'static str;
}

/// Neo N3 Contract Entry Point
pub trait NeoContractEntry {
    fn deploy() -> NeoResult<()>;
    fn update() -> NeoResult<()>;
    fn destroy() -> NeoResult<()>;
}

/// Neo N3 Contract Method trait
pub trait NeoContractMethodTrait {
    fn name() -> &'static str;
    fn parameters() -> &'static [&'static str];
    fn return_type() -> &'static str;
    fn execute(args: &[NeoValue]) -> NeoResult<NeoValue>;
}

/// A type that can be deserialised from a NeoVM `StackItem` /
/// `NeoValue`. This is the L9 `IInteroperable` equivalent in
/// the C# devpack: any contract return type (or argument type)
/// implements this trait, enabling `NeoContract::call_typed<T>`
/// to round-trip the on-chain value into a typed Rust value.
///
/// The default `from_value` for the common types (`NeoInteger`,
/// `NeoBoolean`, `NeoByteString`, `NeoString`) is provided;
/// contract-specific types (e.g. a custom `Token` struct) can
/// implement it by parsing the underlying `NeoValue` into the
/// concrete type.
pub trait FromNeoValue: Sized {
    /// Convert a `NeoValue` into `Self`. Returns an error if
    /// the value's runtime type doesn't match `Self`'s expected
    /// type (e.g. trying to extract an Integer from a String).
    fn from_value(value: &NeoValue) -> NeoResult<Self>;
}

/// Trait for static-method-style cross-contract calls. The L9
/// `call_typed<T>` helper invokes a remote contract's method
/// and decodes the return value into a typed Rust value via
/// the `FromNeoValue` trait. The default implementation
/// delegates to the existing `NeoVMSyscall::contract_call`
/// path; contract code that wants a custom transport (e.g.
/// off-chain simulation) can override `call_raw` and route
/// the value through `call_typed`.
///
/// **L9 status**: shipped as a trait + impl. The default
/// `call_raw` calls `NeoVMSyscall::contract_call`, which on
/// the wasm32 path panics with a clear "see L6 design" message
/// (B4 fix). On the host path it returns `NeoValue::Null`
/// (existing B4 behaviour). The L6 cross-call executor will
/// upgrade the wasm32 path to a real implementation; `call_typed`
/// needs no changes for that.
pub trait ContractCaller {
    /// Invoke a remote contract's method with the given args
    /// and return the raw `NeoValue`. The default impl uses
    /// `NeoVMSyscall::contract_call`.
    fn call_raw(
        &self,
        script_hash: &crate::bytestring::NeoByteString,
        method: &str,
        args: &[NeoValue],
        call_flags: &crate::integer::NeoInteger,
    ) -> NeoResult<NeoValue>;

    /// Invoke a remote contract's method and decode the return
    /// value into a typed Rust value via `FromNeoValue`.
    fn call_typed<T: FromNeoValue>(
        &self,
        script_hash: &crate::bytestring::NeoByteString,
        method: &str,
        args: &[NeoValue],
        call_flags: &crate::integer::NeoInteger,
    ) -> NeoResult<T> {
        let raw = self.call_raw(script_hash, method, args, call_flags)?;
        T::from_value(&raw)
    }
}

/// Default `ContractCaller` impl: routes to the existing
/// `NeoVMSyscall::contract_call`. The host-mode test harness
/// uses this; production wasm32 calls go through the L6
/// cross-call executor.
///
/// **Note**: the default `call_raw` body is defined in
/// `neo-runtime::contract_caller` (which has the dependency on
/// `neo-syscalls`); this trait here just declares the interface.
/// Contract code uses `DefaultContractCaller` via
/// `neo-devpack::prelude::*`.

/// Common FromNeoValue impls for the standard Neo N3 types.

impl FromNeoValue for NeoValue {
    fn from_value(value: &NeoValue) -> NeoResult<Self> {
        Ok(value.clone())
    }
}

impl FromNeoValue for () {
    fn from_value(_value: &NeoValue) -> NeoResult<Self> {
        Ok(())
    }
}

impl FromNeoValue for bool {
    fn from_value(value: &NeoValue) -> NeoResult<Self> {
        value
            .as_boolean()
            .map(|b| b.as_bool())
            .ok_or_else(|| crate::error::NeoError::new("expected Boolean"))
    }
}

impl FromNeoValue for String {
    fn from_value(value: &NeoValue) -> NeoResult<Self> {
        let bytes = value
            .as_byte_string()
            .map(|bs| bs.as_slice().to_vec())
            .or_else(|| value.as_string().map(|s| s.as_str().as_bytes().to_vec()))
            .ok_or_else(|| crate::error::NeoError::new("expected String"))?;
        String::from_utf8(bytes)
            .map_err(|e| crate::error::NeoError::new(&format!("invalid UTF-8: {e}")))
    }
}

impl FromNeoValue for Vec<u8> {
    fn from_value(value: &NeoValue) -> NeoResult<Self> {
        value
            .as_byte_string()
            .map(|bs| bs.as_slice().to_vec())
            .ok_or_else(|| crate::error::NeoError::new("expected ByteArray"))
    }
}

// Pull NeoInteger in scope for the impls below.
use crate::integer::NeoInteger;

impl FromNeoValue for NeoInteger {
    fn from_value(value: &NeoValue) -> NeoResult<Self> {
        value
            .as_integer()
            .cloned()
            .ok_or_else(|| crate::error::NeoError::new("expected Integer"))
    }
}

impl FromNeoValue for i64 {
    fn from_value(value: &NeoValue) -> NeoResult<Self> {
        let n = <NeoInteger as FromNeoValue>::from_value(value)?;
        Ok(n.as_i64_saturating())
    }
}
