// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Neo N3 syscall wrapper and helper functions.

use neo_types::*;

use crate::storage::*;
use crate::syscalls::SYSCALLS;
use crate::NeoVMSyscallInfo;

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
        "Boolean" => NeoBoolean::TRUE.into(),
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

    /// Set the active contract hash used by host-mode storage contexts and script-hash syscalls.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_active_contract_hash(hash: &NeoByteString) -> NeoResult<()> {
        set_current_contract_hash(Self::parse_hash160(hash)?);
        Ok(())
    }

    /// Set the active contract hash used by host-mode storage contexts and script-hash syscalls.
    #[cfg(target_arch = "wasm32")]
    pub fn set_active_contract_hash(_hash: &NeoByteString) -> NeoResult<()> {
        Ok(())
    }

    /// Clear host-mode syscall/storage simulation state.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn reset_host_state() -> NeoResult<()> {
        STORAGE_STATE.reset()?;
        reset_current_contract_hash();
        Ok(())
    }

    /// Clear host-mode syscall/storage simulation state.
    #[cfg(target_arch = "wasm32")]
    pub fn reset_host_state() -> NeoResult<()> {
        let mut store = STORAGE_ENTRIES.lock().map_err(|_| NeoError::InvalidState)?;
        store.clear();
        Ok(())
    }

    fn call_value(name: &str, args: &[NeoValue]) -> NeoResult<NeoValue> {
        neovm_syscall(syscall_hash(name)?, args)
    }

    fn call_integer(name: &str) -> NeoResult<NeoInteger> {
        let value = Self::call_value(name, &[])?;
        value.as_integer().ok_or(NeoError::InvalidType)
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

    /// Get current timestamp
    pub fn get_time() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Runtime.GetTime")
    }

    /// Check if the specified account is a witness
    pub fn check_witness(account: &NeoByteString) -> NeoResult<NeoBoolean> {
        let args = [NeoValue::from(account.clone())];
        Self::call_boolean("System.Runtime.CheckWitness", &args)
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
        Ok(NeoByteString::from_slice(&current_contract_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        Self::call_bytes("System.Runtime.GetCallingScriptHash")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&current_contract_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        Self::call_bytes("System.Runtime.GetEntryScriptHash")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_executing_script_hash() -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&current_contract_hash()))
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
        Self::call_value("System.Contract.Call", &values)
    }

    /// Call a native contract by id.
    pub fn contract_call_native(native_id: &NeoInteger) -> NeoResult<NeoValue> {
        let values = [NeoValue::from(native_id.clone())];
        Self::call_value("System.Contract.CallNative", &values)
    }

    pub fn get_call_flags() -> NeoResult<NeoInteger> {
        Self::call_integer("System.Contract.GetCallFlags")
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
        STORAGE_STATE.create_context(current_contract_hash(), false)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn storage_get_context() -> NeoResult<NeoStorageContext> {
        Ok(NeoStorageContext::new(1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get_read_only_context() -> NeoResult<NeoStorageContext> {
        STORAGE_STATE.create_context(current_contract_hash(), true)
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
        let handle = STORAGE_STATE.get_handle(context)?;
        if handle.read_only {
            return Err(NeoError::InvalidOperation);
        }
        let mut store = handle.store.write().map_err(|_| NeoError::InvalidState)?;
        store.insert(key.as_slice().to_vec(), value.as_slice().to_vec());
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn storage_put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        if context.is_read_only() {
            return Err(NeoError::InvalidOperation);
        }

        let mut store = STORAGE_ENTRIES.lock().map_err(|_| NeoError::InvalidState)?;
        if let Some((_, existing_value)) = store
            .iter_mut()
            .find(|(entry_key, _)| entry_key.as_slice() == key.as_slice())
        {
            *existing_value = value.as_slice().to_vec();
        } else {
            store.push((key.as_slice().to_vec(), value.as_slice().to_vec()));
        }

        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn storage_get(
        _context: &NeoStorageContext,
        key: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        let store = STORAGE_ENTRIES.lock().map_err(|_| NeoError::InvalidState)?;
        let value = store
            .iter()
            .find(|(entry_key, _)| entry_key.as_slice() == key.as_slice())
            .map(|(_, entry_value)| entry_value.clone())
            .unwrap_or_default();
        Ok(NeoByteString::new(value))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        let handle = STORAGE_STATE.get_handle(context)?;
        if handle.read_only {
            return Err(NeoError::InvalidOperation);
        }
        let mut store = handle.store.write().map_err(|_| NeoError::InvalidState)?;
        store.remove(key.as_slice());
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn storage_delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        if context.is_read_only() {
            return Err(NeoError::InvalidOperation);
        }

        let mut store = STORAGE_ENTRIES.lock().map_err(|_| NeoError::InvalidState)?;
        store.retain(|(entry_key, _)| entry_key.as_slice() != key.as_slice());
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_find(
        context: &NeoStorageContext,
        prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
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

    #[cfg(target_arch = "wasm32")]
    pub fn storage_find(
        _context: &NeoStorageContext,
        prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
        let prefix_bytes = prefix.as_slice();
        let store = STORAGE_ENTRIES.lock().map_err(|_| NeoError::InvalidState)?;
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
}
