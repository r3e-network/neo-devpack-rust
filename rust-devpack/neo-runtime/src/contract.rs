// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use neo_types::*;

/// Neo N3 contract management operations.
///
/// **These are test/example stubs.** In deployed contracts, the `#[neo_contract]`
/// macro generates Wasm imports that map to NeoVM syscalls
/// (`System.Contract.Create`, `System.Contract.Update`, `System.Contract.Call`).
/// The methods here allow compiling and unit-testing contract code off-chain.
///
/// - `create()` returns a zero-filled 20-byte hash (placeholder script hash).
/// - `update()` and `destroy()` succeed immediately with no side effects.
/// - `call()` always returns `NeoValue::Null`.
pub struct NeoContractRuntime;

impl NeoContractRuntime {
    /// Deploy a new contract. Returns a placeholder 20-byte script hash.
    pub fn create(
        script: &NeoByteString,
        manifest: &NeoContractManifest,
    ) -> NeoResult<NeoByteString> {
        let _ = (script, manifest);
        Ok(NeoByteString::new(vec![0u8; 20]))
    }

    /// Update an existing contract's script and manifest.
    pub fn update(
        _script_hash: &NeoByteString,
        script: &NeoByteString,
        manifest: &NeoContractManifest,
    ) -> NeoResult<()> {
        let _ = (script, manifest);
        Ok(())
    }

    /// Destroy a contract.
    pub fn destroy(script_hash: &NeoByteString) -> NeoResult<()> {
        let _ = script_hash;
        Ok(())
    }

    /// Call another contract's method. Always returns `NeoValue::Null` in stubs.
    pub fn call(
        script_hash: &NeoByteString,
        method: &NeoString,
        args: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoValue> {
        let _ = (script_hash, method, args);
        Ok(NeoValue::Null)
    }
}
