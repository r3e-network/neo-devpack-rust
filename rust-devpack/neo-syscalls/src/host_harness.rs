// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Host test-harness setters for `NeoVMSyscall`: configure the host-mode
//! simulation state (script hashes, witnesses, crypto results, runtime
//! scalars, storage seeds) that the dispatch in `dispatch.rs` reads. Every
//! setter has a wasm32 no-op twin so contract code can call them
//! unconditionally; on-chain the real values come from the node.

use neo_types::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::storage::*;

use crate::NeoVMSyscall;

impl NeoVMSyscall {
    /// Set the active contract hash used by host-mode storage contexts and script-hash syscalls.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_contract_hash(hash: &NeoByteString) -> NeoResult<()> {
        set_current_contract_hash(Self::parse_hash160(hash)?);
        Ok(())
    }

    /// Configure host-mode calling/entry/executing script hashes.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_script_hashes(
        calling: &NeoByteString,
        entry: &NeoByteString,
        executing: &NeoByteString,
    ) -> NeoResult<()> {
        set_current_script_hashes(
            Self::parse_hash160(calling)?,
            Self::parse_hash160(entry)?,
            Self::parse_hash160(executing)?,
        );
        Ok(())
    }

    /// Configure host-mode calling script hash.
    /// Clears nested invocation frames and applies this value as a new base state.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_calling_script_hash(hash: &NeoByteString) -> NeoResult<()> {
        set_current_calling_script_hash(Self::parse_hash160(hash)?);
        Ok(())
    }

    /// Configure host-mode entry script hash.
    /// Clears nested invocation frames and applies this value as a new base state.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_entry_script_hash(hash: &NeoByteString) -> NeoResult<()> {
        set_current_entry_script_hash(Self::parse_hash160(hash)?);
        Ok(())
    }

    /// Configure host-mode executing script hash.
    /// Clears nested invocation frames and applies this value as a new base state.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_executing_script_hash(hash: &NeoByteString) -> NeoResult<()> {
        set_current_executing_script_hash(Self::parse_hash160(hash)?);
        Ok(())
    }

    /// Configure host-mode active call flags (Neo N3 CallFlags mask: 0x00..=0x0F).
    /// Clears nested invocation frames and applies this value as a new base state.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_call_flags(call_flags: &NeoInteger) -> NeoResult<()> {
        set_current_call_flags(Self::parse_call_flags(call_flags)?);
        Ok(())
    }

    /// Enter a nested contract invocation frame in host mode.
    ///
    /// The new frame preserves `entry`, shifts `calling <- previous executing`,
    /// and sets `executing` to `next_executing`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn begin_contract_invocation(next_executing: &NeoByteString) -> NeoResult<()> {
        Self::begin_contract_invocation_with_flags(next_executing, current_call_flags())
    }

    /// Exit the most recent nested contract invocation frame in host mode.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn end_contract_invocation() -> NeoResult<()> {
        pop_current_script_hash_frame()
    }

    /// Run an operation in a nested host invocation frame, always unwinding the frame.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_contract_invocation<T, F>(
        next_executing: &NeoByteString,
        operation: F,
    ) -> NeoResult<T>
    where
        F: FnOnce() -> NeoResult<T>,
    {
        Self::begin_contract_invocation(next_executing)?;
        let operation_result = operation();
        let unwind_result = Self::end_contract_invocation();

        match (operation_result, unwind_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(err), Ok(())) => Err(err),
            (Ok(_), Err(unwind_err)) => Err(unwind_err),
            (Err(operation_err), Err(unwind_err)) => Err(NeoError::new(&format!(
                "invocation operation failed ({}) and frame unwind failed ({})",
                operation_err.message(),
                unwind_err.message()
            ))),
        }
    }

    /// Set the active contract hash used by host-mode storage contexts and script-hash syscalls.
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_contract_hash(_hash: &NeoByteString) -> NeoResult<()> {
        Ok(())
    }

    /// Configure host-mode calling/entry/executing script hashes.
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_script_hashes(
        _calling: &NeoByteString,
        _entry: &NeoByteString,
        _executing: &NeoByteString,
    ) -> NeoResult<()> {
        Ok(())
    }

    /// Configure host-mode calling script hash.
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_calling_script_hash(_hash: &NeoByteString) -> NeoResult<()> {
        Ok(())
    }

    /// Configure host-mode entry script hash.
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_entry_script_hash(_hash: &NeoByteString) -> NeoResult<()> {
        Ok(())
    }

    /// Configure host-mode executing script hash.
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_executing_script_hash(_hash: &NeoByteString) -> NeoResult<()> {
        Ok(())
    }

    /// Configure host-mode active call flags.
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_call_flags(_call_flags: &NeoInteger) -> NeoResult<()> {
        Ok(())
    }

    /// Enter a nested contract invocation frame in host mode.
    #[cfg(target_arch = "wasm32")]
    pub fn begin_contract_invocation(_next_executing: &NeoByteString) -> NeoResult<()> {
        Ok(())
    }

    /// Exit the most recent nested contract invocation frame in host mode.
    #[cfg(target_arch = "wasm32")]
    pub fn end_contract_invocation() -> NeoResult<()> {
        Ok(())
    }

    /// Run an operation in a nested host invocation frame, always unwinding the frame.
    #[cfg(target_arch = "wasm32")]
    pub fn with_contract_invocation<T, F>(
        _next_executing: &NeoByteString,
        operation: F,
    ) -> NeoResult<T>
    where
        F: FnOnce() -> NeoResult<T>,
    {
        operation()
    }

    /// Clear host-mode syscall/storage simulation state.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn reset_host_state() -> NeoResult<()> {
        STORAGE_STATE.reset()?;
        reset_current_contract_hash();
        clear_active_witnesses();
        reset_crypto_verification_results();
        // B5-B9: clear the runtime syscall host state.
        *crate::storage::ACTIVE_RANDOM
            .write()
            .expect("ACTIVE_RANDOM poisoned") = 0;
        *crate::storage::ACTIVE_TIME
            .write()
            .expect("ACTIVE_TIME poisoned") = 0;
        *crate::storage::ACTIVE_GAS_LEFT
            .write()
            .expect("ACTIVE_GAS_LEFT poisoned") = 0;
        *crate::storage::ACTIVE_INVOCATION_COUNTER
            .write()
            .expect("ACTIVE_INVOCATION_COUNTER poisoned") = 0;
        // Also drain any recorded notifications so the B9
        // dispatch doesn't see state leaked from prior tests.
        crate::host_notifications::reset();
        // And the single-live-iterator scan session (Storage.Find bridge).
        crate::dispatch::reset_host_iterator_session();
        Ok(())
    }

    /// Clear host-mode syscall/storage simulation state.
    ///
    /// On wasm32 this is a no-op: storage state lives in the Neo node's real
    /// persistent store and is reset at the chain level (e.g. by tearing down
    /// the Neo Express chain), not by the contract itself.
    #[cfg(target_arch = "wasm32")]
    pub fn reset_host_state() -> NeoResult<()> {
        Ok(())
    }

    /// Seed host-mode storage with the given key/value pairs (D6: bridges
    /// `neo-test::TestEnvironment::set_storage` to the global syscall mock so
    /// contract code reading via `NeoStorage`/`RawStorage` sees the same
    /// store). Pairs are written under the *currently executing* contract
    /// hash (set via `set_active_contract_hash` / `set_current_contract_hash`;
    /// default zero-sentinel). On wasm32 this is a no-op.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn seed_storage(entries: &[(&[u8], &[u8])]) -> NeoResult<()> {
        for (k, v) in entries {
            STORAGE_STATE.put(k.to_vec(), v.to_vec());
        }
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn seed_storage(_entries: &[(&[u8], &[u8])]) -> NeoResult<()> {
        Ok(())
    }

    /// Replace the active witness set used by host-mode `check_witness`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_witnesses(witnesses: &[NeoByteString]) -> NeoResult<()> {
        crate::storage::set_active_witnesses(
            witnesses.iter().map(|witness| witness.as_slice().to_vec()),
        );
        Ok(())
    }

    /// Replace the active witness set used by host-mode `check_witness`.
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_witnesses(_witnesses: &[NeoByteString]) -> NeoResult<()> {
        Ok(())
    }

    /// B5: set the value returned by host-mode `get_random`.
    /// On the wasm32 path this is a no-op (the extern returns
    /// the chain's real random value).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_random(value: i64) -> NeoResult<()> {
        *crate::storage::ACTIVE_RANDOM
            .write()
            .expect("ACTIVE_RANDOM poisoned") = value;
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_random(_value: i64) -> NeoResult<()> {
        Ok(())
    }

    /// B6: set the value returned by host-mode `get_time`.
    /// On the wasm32 path this is a no-op.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_time(value: i64) -> NeoResult<()> {
        *crate::storage::ACTIVE_TIME
            .write()
            .expect("ACTIVE_TIME poisoned") = value;
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_time(_value: i64) -> NeoResult<()> {
        Ok(())
    }

    /// B6: set the value returned by host-mode
    /// `get_invocation_counter`. On the wasm32 path this is a
    /// no-op.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_invocation_counter(value: i32) -> NeoResult<()> {
        *crate::storage::ACTIVE_INVOCATION_COUNTER
            .write()
            .expect("ACTIVE_INVOCATION_COUNTER poisoned") = value;
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_invocation_counter(_value: i32) -> NeoResult<()> {
        Ok(())
    }

    /// B7: set the value returned by host-mode `get_gas_left`.
    /// On the wasm32 path this is a no-op.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_gas_left(value: i64) -> NeoResult<()> {
        *crate::storage::ACTIVE_GAS_LEFT
            .write()
            .expect("ACTIVE_GAS_LEFT poisoned") = value;
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_gas_left(_value: i64) -> NeoResult<()> {
        Ok(())
    }

    /// Configure host-mode CheckSig/CheckMultisig results.
    ///
    /// `verify_with_ecdsa` tracks `check_sig` unless overridden explicitly.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_crypto_verification_results(check_sig: bool, check_multisig: bool) -> NeoResult<()> {
        Self::set_crypto_verification_results_full(check_sig, check_multisig, check_sig)
    }

    /// Configure host-mode crypto syscall results (secure default: all false).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_crypto_verification_results_full(
        check_sig: bool,
        check_multisig: bool,
        verify_with_ecdsa: bool,
    ) -> NeoResult<()> {
        crate::storage::set_crypto_verification_results(CryptoVerificationResults {
            check_sig,
            check_multisig,
            verify_with_ecdsa,
        });
        Ok(())
    }

    /// Configure host-mode VerifyWithECDsa syscall result.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_verify_with_ecdsa_result(result: bool) -> NeoResult<()> {
        let mut current = active_crypto_verification_results();
        current.verify_with_ecdsa = result;
        crate::storage::set_crypto_verification_results(current);
        Ok(())
    }

    /// Configure host-mode CheckSig/CheckMultisig results.
    #[cfg(target_arch = "wasm32")]
    pub fn set_crypto_verification_results(
        _check_sig: bool,
        _check_multisig: bool,
    ) -> NeoResult<()> {
        Ok(())
    }

    /// Configure host-mode crypto syscall results (secure default: all false).
    #[cfg(target_arch = "wasm32")]
    pub fn set_crypto_verification_results_full(
        _check_sig: bool,
        _check_multisig: bool,
        _verify_with_ecdsa: bool,
    ) -> NeoResult<()> {
        Ok(())
    }

    /// Configure host-mode VerifyWithECDsa syscall result.
    #[cfg(target_arch = "wasm32")]
    pub fn set_verify_with_ecdsa_result(_result: bool) -> NeoResult<()> {
        Ok(())
    }
}
