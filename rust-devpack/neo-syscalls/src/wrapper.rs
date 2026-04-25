// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Neo N3 syscall wrapper and helper functions.

use neo_types::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::storage::*;
use crate::syscalls::SYSCALLS;
use crate::NeoVMSyscallInfo;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "neo")]
extern "C" {
    #[link_name = "runtime_check_witness_bytes"]
    fn neo_runtime_check_witness_bytes(ptr: i32, len: i32) -> i32;

    #[link_name = "runtime_check_witness_i64"]
    fn neo_runtime_check_witness_i64(account: i64) -> i32;

    /// Lowered to `CALL_L` -> helper that emits
    ///   `SYSCALL System.Storage.GetContext;
    ///    SUBSTR <key>; SUBSTR <value>;
    ///    SYSCALL System.Storage.Put`.
    #[link_name = "neo_storage_put_bytes"]
    fn neo_storage_put_bytes(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32);

    /// Lowered to `CALL_L` -> helper that emits
    ///   `SYSCALL System.Storage.GetContext;
    ///    SUBSTR <key>;
    ///    SYSCALL System.Storage.Delete`.
    #[link_name = "neo_storage_delete_bytes"]
    fn neo_storage_delete_bytes(key_ptr: i32, key_len: i32);

    /// Lowered to `CALL_L` -> helper that emits the `Get` SYSCALL and then
    /// copies the returned `ByteString` back into wasm memory at `out_ptr`.
    /// Returns:
    ///   - the stored value's length on success (`>= 0`),
    ///   - `-1` if the key is not present in storage,
    ///   - `-needed_len` if the caller-supplied buffer was too small.
    #[link_name = "neo_storage_get_into"]
    fn neo_storage_get_into(key_ptr: i32, key_len: i32, out_ptr: i32, out_cap: i32) -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
const CALL_FLAGS_VALID_MASK: i32 = 0x0F;
#[cfg(not(target_arch = "wasm32"))]
const CALL_FLAGS_READ_STATES: i32 = 0x01;
#[cfg(not(target_arch = "wasm32"))]
const CALL_FLAGS_WRITE_STATES: i32 = 0x02;

fn find_syscall(name: &str) -> Option<&'static NeoVMSyscallInfo> {
    SYSCALLS.iter().find(|info| info.name == name)
}

fn syscall_hash(name: &str) -> NeoResult<u32> {
    find_syscall(name)
        .map(|info| info.hash)
        .ok_or_else(|| NeoError::new(&format!("unknown syscall: {name}")))
}

fn default_value_for(return_type: &str) -> NeoValue {
    match return_type {
        "Void" => NeoValue::Null,
        // Fail-closed by default for unknown boolean-returning syscalls.
        "Boolean" => NeoBoolean::FALSE.into(),
        "Integer" => NeoInteger::new(0).into(),
        "Hash160" => NeoByteString::new(vec![0u8; 20]).into(),
        "ByteString" => NeoByteString::new(vec![0u8; 1]).into(),
        "String" => NeoString::from_str("Neo N3").into(),
        "Array" => NeoArray::<NeoValue>::new().into(),
        "Iterator" => NeoArray::<NeoValue>::new().into(),
        "StackItem" => NeoArray::<NeoValue>::new().into(),
        "StorageContext" => NeoValue::Null,
        _ => NeoValue::Null,
    }
}

fn value_matches_param_type(value: &NeoValue, param_type: &str) -> bool {
    match param_type {
        "Boolean" => value.as_boolean().is_some(),
        "Integer" => value.as_integer().is_some(),
        "Hash160" => {
            value.is_null()
                || value
                    .as_byte_string()
                    .map(|bytes| bytes.len() == 20)
                    .unwrap_or(false)
        }
        "ByteString" => value.as_byte_string().is_some(),
        "String" => value.as_string().is_some(),
        "Array" => value.as_array().is_some(),
        "Iterator" => value.as_array().is_some(),
        "StorageContext" => value.is_null() || value.as_integer().is_some(),
        "StackItem" | "Any" | "ExecutionContext" => true,
        _ => true,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn call_flags_allow_write(flags: i32) -> bool {
    (flags & CALL_FLAGS_WRITE_STATES) != 0
}

#[cfg(not(target_arch = "wasm32"))]
fn call_flags_allow_read(flags: i32) -> bool {
    (flags & CALL_FLAGS_READ_STATES) != 0
}

/// Neo N3 System Call Function
pub fn neovm_syscall(hash: u32, args: &[NeoValue]) -> NeoResult<NeoValue> {
    let registry = crate::NeoVMSyscallRegistry::get_instance();
    let info = registry
        .get_syscall_by_hash(hash)
        .ok_or_else(|| NeoError::new(&format!("unknown syscall hash: 0x{hash:08x}")))?;

    if args.len() != info.parameters.len() {
        return Err(NeoError::new(&format!(
            "invalid syscall argument count for {}: expected {}, got {}",
            info.name,
            info.parameters.len(),
            args.len()
        )));
    }

    for (index, (arg, expected_type)) in args.iter().zip(info.parameters.iter()).enumerate() {
        if !value_matches_param_type(arg, expected_type) {
            return Err(NeoError::new(&format!(
                "invalid syscall argument type for {} param #{}: expected {}",
                info.name, index, expected_type
            )));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if info.name == "System.Runtime.CheckWitness" {
            let has_witness = args
                .first()
                .and_then(NeoValue::as_byte_string)
                .map(|account| has_active_witness(account.as_slice()))
                .unwrap_or(false);
            return Ok(NeoBoolean::new(has_witness).into());
        }

        if info.name == "System.Crypto.CheckSig" {
            let results = active_crypto_verification_results();
            return Ok(NeoBoolean::new(results.check_sig).into());
        }

        if info.name == "System.Crypto.CheckMultisig" {
            let results = active_crypto_verification_results();
            return Ok(NeoBoolean::new(results.check_multisig).into());
        }

        if info.name == "Neo.Crypto.VerifyWithECDsa" {
            let results = active_crypto_verification_results();
            return Ok(NeoBoolean::new(results.verify_with_ecdsa).into());
        }

        if info.name == "System.Runtime.GetCallingScriptHash" {
            return Ok(NeoByteString::from_slice(&current_calling_script_hash()).into());
        }

        if info.name == "System.Runtime.GetEntryScriptHash" {
            return Ok(NeoByteString::from_slice(&current_entry_script_hash()).into());
        }

        if info.name == "System.Runtime.GetExecutingScriptHash" {
            return Ok(NeoByteString::from_slice(&current_executing_script_hash()).into());
        }

        if info.name == "System.Contract.GetCallFlags" {
            return Ok(NeoInteger::new(current_call_flags()).into());
        }
    }

    Ok(default_value_for(info.return_type))
}

/// Neo N3 System Call Wrapper
pub struct NeoVMSyscall;

impl NeoVMSyscall {
    #[cfg(not(target_arch = "wasm32"))]
    fn parse_hash160(hash: &NeoByteString) -> NeoResult<[u8; 20]> {
        if hash.len() != 20 {
            return Err(NeoError::InvalidArgument);
        }
        let mut value = [0u8; 20];
        value.copy_from_slice(hash.as_slice());
        Ok(value)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn parse_call_flags(flags: &NeoInteger) -> NeoResult<i32> {
        let parsed = flags.as_i32_saturating();
        if parsed < 0 || (parsed & !CALL_FLAGS_VALID_MASK) != 0 {
            return Err(NeoError::InvalidArgument);
        }
        Ok(parsed)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn begin_contract_invocation_with_flags(
        next_executing: &NeoByteString,
        call_flags: i32,
    ) -> NeoResult<()> {
        if call_flags < 0 || (call_flags & !CALL_FLAGS_VALID_MASK) != 0 {
            return Err(NeoError::InvalidArgument);
        }
        push_current_executing_script_hash(Self::parse_hash160(next_executing)?, call_flags)
    }

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

    fn call_value(name: &str, args: &[NeoValue]) -> NeoResult<NeoValue> {
        neovm_syscall(syscall_hash(name)?, args)
    }

    fn call_integer(name: &str) -> NeoResult<NeoInteger> {
        let value = Self::call_value(name, &[])?;
        value.as_integer().cloned().ok_or(NeoError::InvalidType)
    }

    fn call_boolean(name: &str, args: &[NeoValue]) -> NeoResult<NeoBoolean> {
        let value = Self::call_value(name, args)?;
        value.as_boolean().ok_or(NeoError::InvalidType)
    }

    #[cfg(target_arch = "wasm32")]
    fn call_bytes(name: &str) -> NeoResult<NeoByteString> {
        let value = Self::call_value(name, &[])?;
        value.as_byte_string().cloned().ok_or(NeoError::InvalidType)
    }

    fn call_bytes_with_args(name: &str, args: &[NeoValue]) -> NeoResult<NeoByteString> {
        let value = Self::call_value(name, args)?;
        value.as_byte_string().cloned().ok_or(NeoError::InvalidType)
    }

    fn call_string(name: &str) -> NeoResult<NeoString> {
        let value = Self::call_value(name, &[])?;
        value.as_string().cloned().ok_or(NeoError::InvalidType)
    }

    fn call_array(name: &str, args: &[NeoValue]) -> NeoResult<NeoArray<NeoValue>> {
        let value = Self::call_value(name, args)?;
        value.as_array().cloned().ok_or(NeoError::InvalidType)
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

    /// Get current timestamp
    pub fn get_time() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Runtime.GetTime")
    }

    /// Check if the specified account is a witness
    pub fn check_witness(account: &NeoByteString) -> NeoResult<NeoBoolean> {
        Self::check_witness_bytes(account.as_slice())
    }

    /// Check if the specified account hash/public key bytes are a witness.
    pub fn check_witness_bytes(account: &[u8]) -> NeoResult<NeoBoolean> {
        #[cfg(target_arch = "wasm32")]
        {
            let result = unsafe {
                neo_runtime_check_witness_bytes(account.as_ptr() as i32, account.len() as i32)
            };
            return Ok(NeoBoolean::new(result != 0));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let args = [NeoValue::from(NeoByteString::from_slice(account))];
            Self::call_boolean("System.Runtime.CheckWitness", &args)
        }
    }

    /// Check a compact sample-account identifier as a witness.
    ///
    /// This helper exists for the repository sample contracts that expose
    /// account IDs as integers. Production contracts should prefer
    /// `check_witness`/`check_witness_bytes` with real Hash160 account bytes.
    pub fn check_witness_i64(account: i64) -> NeoResult<NeoBoolean> {
        #[cfg(target_arch = "wasm32")]
        {
            let result = unsafe { neo_runtime_check_witness_i64(account) };
            return Ok(NeoBoolean::new(result != 0));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut bytes = [0u8; 20];
            bytes[..8].copy_from_slice(&account.to_le_bytes());
            Self::check_witness_bytes(&bytes)
        }
    }

    /// Send notification to the runtime.
    pub fn notify(event: &NeoString, state: &NeoArray<NeoValue>) -> NeoResult<()> {
        let event_bytes = NeoByteString::from_slice(event.as_str().as_bytes());
        let args = [NeoValue::from(event_bytes), NeoValue::from(state.clone())];
        neovm_syscall(syscall_hash("System.Runtime.Notify")?, &args)?;
        Ok(())
    }

    /// Log message to the runtime.
    pub fn log(message: &NeoString) -> NeoResult<()> {
        let message_bytes = NeoByteString::from_slice(message.as_str().as_bytes());
        let args = [NeoValue::from(message_bytes)];
        neovm_syscall(syscall_hash("System.Runtime.Log")?, &args)?;
        Ok(())
    }

    /// Platform identifier
    pub fn platform() -> NeoResult<NeoString> {
        Self::call_string("System.Runtime.Platform")
    }

    pub fn get_trigger() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Runtime.GetTrigger")
    }

    pub fn get_invocation_counter() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Runtime.GetInvocationCounter")
    }

    pub fn get_random() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Runtime.GetRandom")
    }

    pub fn get_network() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Runtime.GetNetwork")
    }

    pub fn get_address_version() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Runtime.GetAddressVersion")
    }

    pub fn get_gas_left() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Runtime.GasLeft")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&current_calling_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        Self::call_bytes("System.Runtime.GetCallingScriptHash")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&current_entry_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        Self::call_bytes("System.Runtime.GetEntryScriptHash")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_executing_script_hash() -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&current_executing_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_executing_script_hash() -> NeoResult<NeoByteString> {
        Self::call_bytes("System.Runtime.GetExecutingScriptHash")
    }

    /// Get notifications for the specified script hash, or all notifications if None.
    pub fn get_notifications(script_hash: Option<&NeoByteString>) -> NeoResult<NeoArray<NeoValue>> {
        let script_hash_value = script_hash
            .map(|hash| NeoValue::from(hash.clone()))
            .unwrap_or(NeoValue::Null);
        let args = [script_hash_value];
        Self::call_array("System.Runtime.GetNotifications", &args)
    }

    pub fn get_script_container() -> NeoResult<NeoArray<NeoValue>> {
        Self::call_array("System.Runtime.GetScriptContainer", &[])
    }

    /// Burn GAS.
    pub fn burn_gas(gas: &NeoInteger) -> NeoResult<()> {
        let args = [NeoValue::from(gas.clone())];
        Self::call_value("System.Runtime.BurnGas", &args)?;
        Ok(())
    }

    /// Get active transaction signers.
    pub fn current_signers() -> NeoResult<NeoArray<NeoValue>> {
        Self::call_array("System.Runtime.CurrentSigners", &[])
    }

    /// Dynamically load and execute a script.
    pub fn load_script(
        script: &NeoByteString,
        call_flags: &NeoInteger,
        args: &NeoArray<NeoValue>,
    ) -> NeoResult<()> {
        let values = [
            NeoValue::from(script.clone()),
            NeoValue::from(call_flags.clone()),
            NeoValue::from(args.clone()),
        ];
        Self::call_value("System.Runtime.LoadScript", &values)?;
        Ok(())
    }

    /// Call any contract method.
    pub fn contract_call(
        script_hash: &NeoByteString,
        method: &NeoString,
        call_flags: &NeoInteger,
        args: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoValue> {
        let values = [
            NeoValue::from(script_hash.clone()),
            NeoValue::from(method.clone()),
            NeoValue::from(call_flags.clone()),
            NeoValue::from(args.clone()),
        ];

        #[cfg(not(target_arch = "wasm32"))]
        {
            let parsed_flags = Self::parse_call_flags(call_flags)?;
            Self::begin_contract_invocation_with_flags(script_hash, parsed_flags)?;
            let call_result = Self::call_value("System.Contract.Call", &values);
            let unwind_result = Self::end_contract_invocation();
            match (call_result, unwind_result) {
                (Ok(value), Ok(())) => Ok(value),
                (Err(err), Ok(())) => Err(err),
                (Ok(_), Err(unwind_err)) => Err(unwind_err),
                (Err(call_err), Err(unwind_err)) => Err(NeoError::new(&format!(
                    "contract_call failed ({}) and invocation unwind failed ({})",
                    call_err.message(),
                    unwind_err.message()
                ))),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self::call_value("System.Contract.Call", &values)
        }
    }

    /// Call a native contract by id.
    pub fn contract_call_native(native_id: &NeoInteger) -> NeoResult<NeoValue> {
        let values = [NeoValue::from(native_id.clone())];
        Self::call_value("System.Contract.CallNative", &values)
    }

    pub fn get_call_flags() -> NeoResult<NeoInteger> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(NeoInteger::new(current_call_flags()))
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self::call_integer("System.Contract.GetCallFlags")
        }
    }

    pub fn create_standard_account(pubkey: &NeoByteString) -> NeoResult<NeoByteString> {
        let values = [NeoValue::from(pubkey.clone())];
        Self::call_bytes_with_args("System.Contract.CreateStandardAccount", &values)
    }

    pub fn create_multisig_account(
        threshold: &NeoInteger,
        public_keys: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoByteString> {
        let values = [
            NeoValue::from(threshold.clone()),
            NeoValue::from(public_keys.clone()),
        ];
        Self::call_bytes_with_args("System.Contract.CreateMultisigAccount", &values)
    }

    pub fn native_on_persist() -> NeoResult<()> {
        Self::call_value("System.Contract.NativeOnPersist", &[])?;
        Ok(())
    }

    pub fn native_post_persist() -> NeoResult<()> {
        Self::call_value("System.Contract.NativePostPersist", &[])?;
        Ok(())
    }

    pub fn check_sig(pubkey: &NeoByteString, signature: &NeoByteString) -> NeoResult<NeoBoolean> {
        let values = [
            NeoValue::from(pubkey.clone()),
            NeoValue::from(signature.clone()),
        ];
        Self::call_boolean("System.Crypto.CheckSig", &values)
    }

    pub fn check_multisig(
        pubkeys: &NeoArray<NeoValue>,
        signatures: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoBoolean> {
        let values = [
            NeoValue::from(pubkeys.clone()),
            NeoValue::from(signatures.clone()),
        ];
        Self::call_boolean("System.Crypto.CheckMultisig", &values)
    }

    pub fn verify_with_ecdsa(
        message: &NeoByteString,
        pubkey: &NeoByteString,
        signature: &NeoByteString,
        curve: &NeoInteger,
    ) -> NeoResult<NeoBoolean> {
        let values = [
            NeoValue::from(message.clone()),
            NeoValue::from(pubkey.clone()),
            NeoValue::from(signature.clone()),
            NeoValue::from(curve.clone()),
        ];
        Self::call_boolean("Neo.Crypto.VerifyWithECDsa", &values)
    }

    pub fn iterator_next(items: &NeoArray<NeoValue>) -> NeoResult<NeoBoolean> {
        let values = [NeoValue::from(items.clone())];
        Self::call_boolean("System.Iterator.Next", &values)
    }

    pub fn iterator_value(items: &NeoArray<NeoValue>) -> NeoResult<NeoValue> {
        let values = [NeoValue::from(items.clone())];
        Self::call_value("System.Iterator.Value", &values)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get_context() -> NeoResult<NeoStorageContext> {
        let flags = current_call_flags();
        if !call_flags_allow_read(flags) {
            return Err(NeoError::InvalidOperation);
        }
        let read_only = !call_flags_allow_write(flags);
        STORAGE_STATE.create_context(current_executing_script_hash(), read_only)
    }

    /// On wasm32 we return a sentinel `NeoStorageContext`. The translator
    /// emits a fresh `SYSCALL System.Storage.GetContext` inside each storage
    /// helper, so the i32 id carried by this struct is irrelevant to NeoVM —
    /// the only field that affects translated bytecode is the `read_only`
    /// marker, which is enforced by the wasm32 wrappers below.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_get_context() -> NeoResult<NeoStorageContext> {
        Ok(NeoStorageContext::new(1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get_read_only_context() -> NeoResult<NeoStorageContext> {
        if !call_flags_allow_read(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        STORAGE_STATE.create_context(current_executing_script_hash(), true)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn storage_get_read_only_context() -> NeoResult<NeoStorageContext> {
        Ok(NeoStorageContext::read_only(1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_as_read_only(context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        STORAGE_STATE.clone_as_read_only(context)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn storage_as_read_only(context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        Ok(context.as_read_only())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get(
        context: &NeoStorageContext,
        key: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        if !call_flags_allow_read(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        let store = handle.store.read().map_err(|_| NeoError::InvalidState)?;
        let value = store.get(key.as_slice()).cloned().unwrap_or_else(Vec::new);
        Ok(NeoByteString::new(value))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        if !call_flags_allow_write(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        if handle.read_only {
            return Err(NeoError::InvalidOperation);
        }
        let mut store = handle.store.write().map_err(|_| NeoError::InvalidState)?;
        store.insert(key.as_slice().to_vec(), value.as_slice().to_vec());
        Ok(())
    }

    /// Writes through to real Neo persistent storage. The translator lowers
    /// `neo_storage_put_bytes` to a `CALL_L` that emits the
    /// `System.Storage.GetContext + System.Storage.Put` SYSCALL pair. The
    /// `read_only` check on the supplied marker still runs first so contracts
    /// that hand a read-only context to `put` short-circuit before crossing
    /// the wasm boundary.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        if context.is_read_only() {
            return Err(NeoError::InvalidOperation);
        }

        let key_slice = key.as_slice();
        let value_slice = value.as_slice();
        unsafe {
            neo_storage_put_bytes(
                key_slice.as_ptr() as i32,
                key_slice.len() as i32,
                value_slice.as_ptr() as i32,
                value_slice.len() as i32,
            );
        }
        Ok(())
    }

    /// Reads through to real Neo persistent storage via the translator-emitted
    /// `neo_storage_get_into` helper. The helper writes the stored bytes into
    /// the local `buffer` (sized up on demand) and reports the actual length;
    /// missing keys return an empty `NeoByteString`, matching the host-mode
    /// semantics already exercised by the devpack tests.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_get(
        _context: &NeoStorageContext,
        key: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        const INITIAL_CAPACITY: usize = 64;
        const MAX_CAPACITY: usize = 64 * 1024;

        let key_slice = key.as_slice();
        let mut buffer: Vec<u8> = vec![0u8; INITIAL_CAPACITY];
        loop {
            let actual = unsafe {
                neo_storage_get_into(
                    key_slice.as_ptr() as i32,
                    key_slice.len() as i32,
                    buffer.as_mut_ptr() as i32,
                    buffer.len() as i32,
                )
            };
            if actual == -1 {
                return Ok(NeoByteString::new(Vec::new()));
            }
            if actual >= 0 {
                let len = actual as usize;
                buffer.truncate(len);
                return Ok(NeoByteString::new(buffer));
            }
            // -needed_len: grow buffer and retry.
            let needed = (-actual) as usize;
            if needed > MAX_CAPACITY {
                return Err(NeoError::InvalidState);
            }
            buffer.resize(needed, 0);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        if !call_flags_allow_write(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        if handle.read_only {
            return Err(NeoError::InvalidOperation);
        }
        let mut store = handle.store.write().map_err(|_| NeoError::InvalidState)?;
        store.remove(key.as_slice());
        Ok(())
    }

    /// Deletes the key from real Neo persistent storage via
    /// `neo_storage_delete_bytes`, which the translator lowers to
    /// `System.Storage.GetContext + System.Storage.Delete`.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        if context.is_read_only() {
            return Err(NeoError::InvalidOperation);
        }

        let key_slice = key.as_slice();
        unsafe {
            neo_storage_delete_bytes(key_slice.as_ptr() as i32, key_slice.len() as i32);
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_find(
        context: &NeoStorageContext,
        prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
        if !call_flags_allow_read(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        let prefix_bytes = prefix.as_slice();
        let store = handle.store.read().map_err(|_| NeoError::InvalidState)?;
        let matches: Vec<NeoValue> = store
            .iter()
            .filter_map(|(key_bytes, value)| {
                if key_bytes.starts_with(prefix_bytes) {
                    let mut entry = NeoStruct::new();
                    entry.set_field("key", NeoValue::from(NeoByteString::from_slice(key_bytes)));
                    entry.set_field("value", NeoValue::from(NeoByteString::from_slice(value)));
                    Some(NeoValue::from(entry))
                } else {
                    None
                }
            })
            .collect();
        Ok(NeoIterator::new(matches))
    }

    /// On wasm32 `storage_find` returns an empty iterator. Bridging a real
    /// `System.Storage.Find` iterator handle through wasm would require
    /// special-cased translator support for `System.Iterator.Next/Value`
    /// on top of the byte-marshalled `Get/Put/Delete` primitives that this
    /// module already lowers; contracts that need prefix iteration must use
    /// indexed enumeration backed by `storage_get` until that lands.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_find(
        _context: &NeoStorageContext,
        _prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
        Ok(NeoIterator::new(Vec::new()))
    }
}
