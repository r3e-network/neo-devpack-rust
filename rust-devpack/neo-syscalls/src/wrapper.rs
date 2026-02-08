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

fn syscall_hash(name: &str) -> u32 {
    find_syscall(name).expect("unknown syscall").hash
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

/// Neo N3 System Call Function
pub fn neovm_syscall(hash: u32, _args: &[NeoValue]) -> NeoResult<NeoValue> {
    let registry = crate::NeoVMSyscallRegistry::get_instance();
    if let Some(info) = registry.get_syscall_by_hash(hash) {
        Ok(default_value_for(info.return_type))
    } else {
        Ok(NeoValue::Null)
    }
}

/// Neo N3 System Call Wrapper
pub struct NeoVMSyscall;

impl NeoVMSyscall {
    fn call_integer(name: &str) -> NeoResult<NeoInteger> {
        let value = neovm_syscall(syscall_hash(name), &[])?;
        value.as_integer().ok_or(NeoError::InvalidType)
    }

    fn call_boolean(name: &str, args: &[NeoValue]) -> NeoResult<NeoBoolean> {
        let value = neovm_syscall(syscall_hash(name), args)?;
        value.as_boolean().ok_or(NeoError::InvalidType)
    }

    fn call_bytes(name: &str) -> NeoResult<NeoByteString> {
        let value = neovm_syscall(syscall_hash(name), &[])?;
        value.as_byte_string().cloned().ok_or(NeoError::InvalidType)
    }

    fn call_string(name: &str) -> NeoResult<NeoString> {
        let value = neovm_syscall(syscall_hash(name), &[])?;
        value.as_string().cloned().ok_or(NeoError::InvalidType)
    }

    fn call_array(name: &str, args: &[NeoValue]) -> NeoResult<NeoArray<NeoValue>> {
        let value = neovm_syscall(syscall_hash(name), args)?;
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
        let args = [NeoValue::from(event.clone()), NeoValue::from(state.clone())];
        neovm_syscall(syscall_hash("System.Runtime.Notify"), &args)?;
        Ok(())
    }

    /// Log message to the runtime.
    pub fn log(message: &NeoString) -> NeoResult<()> {
        let args = [NeoValue::from(message.clone())];
        neovm_syscall(syscall_hash("System.Runtime.Log"), &args)?;
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

    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        Self::call_bytes("System.Runtime.GetCallingScriptHash")
    }

    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        Self::call_bytes("System.Runtime.GetEntryScriptHash")
    }

    pub fn get_executing_script_hash() -> NeoResult<NeoByteString> {
        Self::call_bytes("System.Runtime.GetExecutingScriptHash")
    }

    /// Get notifications for the specified script hash, or all notifications if None.
    pub fn get_notifications(script_hash: Option<&NeoByteString>) -> NeoResult<NeoArray<NeoValue>> {
        let args: Vec<NeoValue> = script_hash
            .map(|hash| vec![NeoValue::from(hash.clone())])
            .unwrap_or_default();
        Self::call_array("System.Runtime.GetNotifications", &args)
    }

    pub fn get_script_container() -> NeoResult<NeoArray<NeoValue>> {
        Self::call_array("System.Runtime.GetScriptContainer", &[])
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get_context() -> NeoResult<NeoStorageContext> {
        STORAGE_STATE.create_context(DEFAULT_CONTRACT_HASH, false)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn storage_get_context() -> NeoResult<NeoStorageContext> {
        Ok(NeoStorageContext::new(1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get_read_only_context() -> NeoResult<NeoStorageContext> {
        STORAGE_STATE.create_context(DEFAULT_CONTRACT_HASH, true)
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