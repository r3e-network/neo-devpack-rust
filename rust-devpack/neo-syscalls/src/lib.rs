//! Neo N3 System Calls
//!
//! This crate provides bindings to Neo N3 system calls for smart contract development.

use neo_types::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::slice::Iter;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

/// Neo N3 System Call Registry
pub struct NeoVMSyscallRegistry {
    syscalls: &'static [NeoVMSyscallInfo],
}

impl NeoVMSyscallRegistry {
    pub const fn new(syscalls: &'static [NeoVMSyscallInfo]) -> Self {
        Self { syscalls }
    }

    pub fn get_syscall(&self, name: &str) -> Option<&NeoVMSyscallInfo> {
        self.syscalls.iter().find(|s| s.name == name)
    }

    pub fn get_syscall_by_hash(&self, hash: u32) -> Option<&NeoVMSyscallInfo> {
        self.syscalls.iter().find(|s| s.hash == hash)
    }

    pub fn has_syscall(&self, name: &str) -> bool {
        self.get_syscall(name).is_some()
    }

    pub fn get_instance() -> Self {
        Self::new(&SYSCALLS)
    }

    pub fn iter(&self) -> Iter<'static, NeoVMSyscallInfo> {
        self.syscalls.iter()
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> {
        self.syscalls.iter().map(|info| info.name)
    }
}

/// Neo N3 System Call Information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoVMSyscallInfo {
    pub name: &'static str,
    pub hash: u32,
    pub parameters: &'static [&'static str],
    pub return_type: &'static str,
    pub gas_cost: u32,
    pub description: &'static str,
}

/// Neo N3 System Call Lowering
pub struct NeoVMSyscallLowering;

impl NeoVMSyscallLowering {
    pub fn new() -> Self {
        Self
    }

    pub fn lower_syscall(&self, name: &str) -> NeoResult<u32> {
        let registry = NeoVMSyscallRegistry::get_instance();
        if let Some(syscall) = registry.get_syscall(name) {
            Ok(syscall.hash)
        } else {
            Err(NeoError::new(&format!("Unknown syscall: {}", name)))
        }
    }

    pub fn can_lower(&self, name: &str) -> bool {
        let registry = NeoVMSyscallRegistry::get_instance();
        registry.has_syscall(name)
    }
}

/// Neo N3 System Call Registry Instance
pub static SYSCALL_REGISTRY: NeoVMSyscallRegistry = NeoVMSyscallRegistry::new(&SYSCALLS);

/// Neo N3 System Calls
pub const SYSCALLS: &[NeoVMSyscallInfo] = &[
    NeoVMSyscallInfo {
        name: "System.Contract.Call",
        hash: 0x525b7d62,
        parameters: &["Hash160", "String", "Integer", "Array"],
        return_type: "Void",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Contract.CallNative",
        hash: 0x677bf71a,
        parameters: &["Integer"],
        return_type: "Void",
        gas_cost: 0,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Contract.CreateMultisigAccount",
        hash: 0x09e9336a,
        parameters: &["Integer", "Array"],
        return_type: "Hash160",
        gas_cost: 0,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Contract.CreateStandardAccount",
        hash: 0x028799cf,
        parameters: &["ByteString"],
        return_type: "Hash160",
        gas_cost: 0,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Contract.GetCallFlags",
        hash: 0x813ada95,
        parameters: &[],
        return_type: "Integer",
        gas_cost: 1024,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Contract.NativeOnPersist",
        hash: 0x93bcdb2e,
        parameters: &[],
        return_type: "Void",
        gas_cost: 0,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Contract.NativePostPersist",
        hash: 0x165da144,
        parameters: &[],
        return_type: "Void",
        gas_cost: 0,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Crypto.CheckMultisig",
        hash: 0x3adcd09e,
        parameters: &["Array", "Array"],
        return_type: "Boolean",
        gas_cost: 0,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Crypto.CheckSig",
        hash: 0x27b3e756,
        parameters: &["ByteString", "ByteString"],
        return_type: "Boolean",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Iterator.Next",
        hash: 0x9ced089c,
        parameters: &["Iterator"],
        return_type: "Boolean",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Iterator.Value",
        hash: 0x1dbf54f3,
        parameters: &["Iterator"],
        return_type: "StackItem",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.BurnGas",
        hash: 0xbc8c5ac3,
        parameters: &["Integer"],
        return_type: "Void",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.CheckWitness",
        hash: 0x8cec27f8,
        parameters: &["ByteString"],
        return_type: "Boolean",
        gas_cost: 1024,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.CurrentSigners",
        hash: 0x8b18f1ac,
        parameters: &[],
        return_type: "Array",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GasLeft",
        hash: 0xced88814,
        parameters: &[],
        return_type: "Integer",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetAddressVersion",
        hash: 0xdc92494c,
        parameters: &[],
        return_type: "Integer",
        gas_cost: 8,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetCallingScriptHash",
        hash: 0x3c6e5339,
        parameters: &[],
        return_type: "Hash160",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetEntryScriptHash",
        hash: 0x38e2b4f9,
        parameters: &[],
        return_type: "Hash160",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetExecutingScriptHash",
        hash: 0x74a8fedb,
        parameters: &[],
        return_type: "Hash160",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetInvocationCounter",
        hash: 0x43112784,
        parameters: &[],
        return_type: "Integer",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetNetwork",
        hash: 0xe0a0fbc5,
        parameters: &[],
        return_type: "Integer",
        gas_cost: 8,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetNotifications",
        hash: 0xf1354327,
        parameters: &["Hash160"],
        return_type: "Array",
        gas_cost: 4096,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetRandom",
        hash: 0x28a9de6b,
        parameters: &[],
        return_type: "Integer",
        gas_cost: 0,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetScriptContainer",
        hash: 0x3008512d,
        parameters: &[],
        return_type: "StackItem",
        gas_cost: 8,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetTime",
        hash: 0x0388c3b7,
        parameters: &[],
        return_type: "Integer",
        gas_cost: 8,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.GetTrigger",
        hash: 0xa0387de9,
        parameters: &[],
        return_type: "Integer",
        gas_cost: 8,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.LoadScript",
        hash: 0x8f800cb3,
        parameters: &["ByteString", "Integer", "Array"],
        return_type: "Void",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.Log",
        hash: 0x9647e7cf,
        parameters: &["ByteString"],
        return_type: "Void",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.Notify",
        hash: 0x616f0195,
        parameters: &["ByteString", "Array"],
        return_type: "Void",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Runtime.Platform",
        hash: 0xf6fc79b2,
        parameters: &[],
        return_type: "String",
        gas_cost: 8,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Storage.AsReadOnly",
        hash: 0xe9bf4c76,
        parameters: &["StorageContext"],
        return_type: "StorageContext",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Storage.Delete",
        hash: 0xedc5582f,
        parameters: &["StorageContext", "ByteString"],
        return_type: "Void",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Storage.Find",
        hash: 0x9ab830df,
        parameters: &["StorageContext", "ByteString", "Integer"],
        return_type: "Iterator",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Storage.Get",
        hash: 0x31e85d92,
        parameters: &["StorageContext", "ByteString"],
        return_type: "ByteString",
        gas_cost: 32768,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Storage.GetContext",
        hash: 0xce67f69b,
        parameters: &[],
        return_type: "StorageContext",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Storage.GetReadOnlyContext",
        hash: 0xe26bb4f6,
        parameters: &[],
        return_type: "StorageContext",
        gas_cost: 16,
        description: "",
    },
    NeoVMSyscallInfo {
        name: "System.Storage.Put",
        hash: 0x84183fe6,
        parameters: &["StorageContext", "ByteString", "ByteString"],
        return_type: "Void",
        gas_cost: 32768,
        description: "",
    },
];

const DEFAULT_CONTRACT_HASH: [u8; 20] = [0u8; 20];

#[derive(Clone)]
struct ContextHandle {
    read_only: bool,
    contract: [u8; 20],
    store: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

struct StorageState {
    next_context: AtomicU32,
    contexts: RwLock<HashMap<u32, ContextHandle>>,
    contract_stores: RwLock<HashMap<[u8; 20], Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>>>,
}

impl StorageState {
    fn new() -> Self {
        Self {
            next_context: AtomicU32::new(1),
            contexts: RwLock::new(HashMap::new()),
            contract_stores: RwLock::new(HashMap::new()),
        }
    }

    fn create_context(&self, contract: [u8; 20], read_only: bool) -> NeoResult<NeoStorageContext> {
        let store = self.get_or_create_store(contract);
        let id = self.next_context.fetch_add(1, Ordering::SeqCst);
        let handle = ContextHandle {
            read_only,
            contract,
            store,
        };
        self.contexts
            .write()
            .map_err(|_| NeoError::InvalidState)?
            .insert(id, handle);
        Ok(if read_only {
            NeoStorageContext::read_only(id)
        } else {
            NeoStorageContext::new(id)
        })
    }

    fn clone_as_read_only(&self, context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        let handle = self
            .contexts
            .read()
            .map_err(|_| NeoError::InvalidState)?
            .get(&context.id())
            .cloned()
            .ok_or(NeoError::InvalidState)?;
        let id = self.next_context.fetch_add(1, Ordering::SeqCst);
        let ro_handle = ContextHandle {
            read_only: true,
            contract: handle.contract,
            store: handle.store,
        };
        self.contexts
            .write()
            .map_err(|_| NeoError::InvalidState)?
            .insert(id, ro_handle);
        Ok(NeoStorageContext::read_only(id))
    }

    fn get_handle(&self, context: &NeoStorageContext) -> NeoResult<ContextHandle> {
        self.contexts
            .read()
            .map_err(|_| NeoError::InvalidState)?
            .get(&context.id())
            .cloned()
            .ok_or(NeoError::InvalidState)
    }

    fn get_or_create_store(
        &self,
        contract: [u8; 20],
    ) -> Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>> {
        let mut stores = self
            .contract_stores
            .write()
            .expect("storage contract lock poisoned");
        stores
            .entry(contract)
            .or_insert_with(|| Arc::new(RwLock::new(HashMap::new())))
            .clone()
    }
}

static STORAGE_STATE: Lazy<StorageState> = Lazy::new(StorageState::new);

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
    let registry = NeoVMSyscallRegistry::get_instance();
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

    /// Send notification
    pub fn notify(event: &NeoString, state: &NeoArray<NeoValue>) -> NeoResult<()> {
        let args = [NeoValue::from(event.clone()), NeoValue::from(state.clone())];
        neovm_syscall(syscall_hash("System.Runtime.Notify"), &args)?;
        Ok(())
    }

    /// Log message
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

    pub fn get_notifications(script_hash: Option<&NeoByteString>) -> NeoResult<NeoArray<NeoValue>> {
        let args: Vec<NeoValue> = script_hash
            .map(|hash| vec![NeoValue::from(hash.clone())])
            .unwrap_or_default();
        Self::call_array("System.Runtime.GetNotifications", args.as_slice())
    }

    pub fn get_script_container() -> NeoResult<NeoArray<NeoValue>> {
        Self::call_array("System.Runtime.GetScriptContainer", &[])
    }

    pub fn storage_get_context() -> NeoResult<NeoStorageContext> {
        STORAGE_STATE.create_context(DEFAULT_CONTRACT_HASH, false)
    }

    pub fn storage_get_read_only_context() -> NeoResult<NeoStorageContext> {
        STORAGE_STATE.create_context(DEFAULT_CONTRACT_HASH, true)
    }

    pub fn storage_as_read_only(context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        STORAGE_STATE.clone_as_read_only(context)
    }

    pub fn storage_get(
        context: &NeoStorageContext,
        key: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        let handle = STORAGE_STATE.get_handle(context)?;
        let store = handle
            .store
            .read()
            .map_err(|_| NeoError::InvalidState)?;
        let value = store
            .get(key.as_slice())
            .cloned()
            .unwrap_or_else(Vec::new);
        Ok(NeoByteString::new(value))
    }

    pub fn storage_put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        let handle = STORAGE_STATE.get_handle(context)?;
        if handle.read_only {
            return Err(NeoError::InvalidOperation);
        }
        let mut store = handle
            .store
            .write()
            .map_err(|_| NeoError::InvalidState)?;
        store.insert(key.as_slice().to_vec(), value.as_slice().to_vec());
        Ok(())
    }

    pub fn storage_delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        let handle = STORAGE_STATE.get_handle(context)?;
        if handle.read_only {
            return Err(NeoError::InvalidOperation);
        }
        let mut store = handle
            .store
            .write()
            .map_err(|_| NeoError::InvalidState)?;
        store.remove(key.as_slice());
        Ok(())
    }

    pub fn storage_find(
        context: &NeoStorageContext,
        prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
        let handle = STORAGE_STATE.get_handle(context)?;
        let prefix_bytes = prefix.as_slice();
        let store = handle
            .store
            .read()
            .map_err(|_| NeoError::InvalidState)?;
        let matches: Vec<NeoValue> = store
            .iter()
            .filter_map(|(key_bytes, value)| {
                if key_bytes.starts_with(prefix_bytes) {
                    let mut entry = NeoStruct::new();
                    entry.set_field(
                        "key",
                        NeoValue::from(NeoByteString::from_slice(key_bytes)),
                    );
                    entry.set_field(
                        "value",
                        NeoValue::from(NeoByteString::from_slice(value)),
                    );
                    Some(NeoValue::from(entry))
                } else {
                    None
                }
            })
            .collect();
        Ok(NeoIterator::new(matches))
    }
}
