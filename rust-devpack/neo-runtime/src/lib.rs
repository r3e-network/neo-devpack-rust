//! Neo N3 Runtime facade
//!
//! This crate provides a lightweight façade over the Neo runtime surface so
//! integration tests and examples can exercise the canonical syscall
//! catalogue without depending on a full node implementation. The
//! implementation intentionally returns deterministic placeholder values –
//! enough to validate wiring and type conversions while remaining
//! self-contained for unit tests.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

extern crate alloc;

use alloc::vec::Vec;
use neo_syscalls::NeoVMSyscall;
use neo_types::*;

/// Lightweight view of the runtime context.
#[derive(Default)]
pub struct NeoRuntimeContext;

impl NeoRuntimeContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trigger(&self) -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_trigger()
    }

    pub fn gas_left(&self) -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_gas_left()
    }

    pub fn invocation_counter(&self) -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_invocation_counter()
    }

    pub fn calling_script_hash(&self) -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_calling_script_hash()
    }

    pub fn entry_script_hash(&self) -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_entry_script_hash()
    }

    pub fn executing_script_hash(&self) -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_executing_script_hash()
    }
}

/// Storage convenience helpers built on top of the syscall layer.
pub struct NeoStorage;

impl NeoStorage {
    pub fn get_context() -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_get_context()
    }

    pub fn get_read_only_context() -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_get_read_only_context()
    }

    pub fn as_read_only(context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_as_read_only(context)
    }

    pub fn get(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<NeoByteString> {
        NeoVMSyscall::storage_get(context, key)
    }

    pub fn put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        NeoVMSyscall::storage_put(context, key, value)
    }

    pub fn delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        NeoVMSyscall::storage_delete(context, key)
    }

    pub fn find(
        context: &NeoStorageContext,
        prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoByteString>> {
        NeoVMSyscall::storage_find(context, prefix)
    }
}

/// Direct wrappers for the canonical System.Runtime syscalls.
pub struct NeoRuntime;

impl NeoRuntime {
    pub fn get_time() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_time()
    }

    pub fn check_witness(account: &NeoByteString) -> NeoResult<NeoBoolean> {
        NeoVMSyscall::check_witness(account)
    }

    pub fn notify(event: &NeoString, state: &NeoArray<NeoValue>) -> NeoResult<()> {
        NeoVMSyscall::notify(event, state)
    }

    pub fn log(message: &NeoString) -> NeoResult<()> {
        NeoVMSyscall::log(message)
    }

    pub fn platform() -> NeoResult<NeoString> {
        NeoVMSyscall::platform()
    }

    pub fn get_trigger() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_trigger()
    }

    pub fn get_invocation_counter() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_invocation_counter()
    }

    pub fn get_random() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_random()
    }

    pub fn get_network() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_network()
    }

    pub fn get_address_version() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_address_version()
    }

    pub fn get_gas_left() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_gas_left()
    }

    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_calling_script_hash()
    }

    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_entry_script_hash()
    }

    pub fn get_executing_script_hash() -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_executing_script_hash()
    }

    pub fn get_notifications(script_hash: Option<&NeoByteString>) -> NeoResult<NeoArray<NeoValue>> {
        NeoVMSyscall::get_notifications(script_hash)
    }

    pub fn get_script_container() -> NeoResult<NeoArray<NeoValue>> {
        NeoVMSyscall::get_script_container()
    }

    pub fn get_storage_context() -> NeoResult<NeoStorageContext> {
        NeoStorage::get_context()
    }
}

/// Minimal representation of contract management utilities used in tests.
pub struct NeoContractRuntime;

impl NeoContractRuntime {
    pub fn create(
        script: &NeoByteString,
        _manifest: &NeoContractManifest,
    ) -> NeoResult<NeoByteString> {
        let mut data = script.as_slice().to_vec();
        data.extend_from_slice(&[0x01, 0x02, 0x03]);
        Ok(NeoByteString::new(data))
    }

    pub fn update(
        _script_hash: &NeoByteString,
        _script: &NeoByteString,
        _manifest: &NeoContractManifest,
    ) -> NeoResult<()> {
        Ok(())
    }

    pub fn destroy(_script_hash: &NeoByteString) -> NeoResult<()> {
        Ok(())
    }

    pub fn call(
        _script_hash: &NeoByteString,
        _method: &NeoString,
        _args: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoValue> {
        Ok(NeoValue::Null)
    }
}

/// Deterministic crypto helpers for tests and examples.
pub struct NeoCrypto;

impl NeoCrypto {
    pub fn sha256(data: &NeoByteString) -> NeoResult<NeoByteString> {
        let mut hash = Vec::new();
        for i in 0..32 {
            hash.push(data.len() as u8 ^ i as u8 ^ 0xAB);
        }
        Ok(NeoByteString::new(hash))
    }

    pub fn ripemd160(data: &NeoByteString) -> NeoResult<NeoByteString> {
        let mut hash = Vec::new();
        for i in 0..20 {
            hash.push(data.len() as u8 ^ i as u8 ^ 0xCD);
        }
        Ok(NeoByteString::new(hash))
    }

    pub fn keccak256(data: &NeoByteString) -> NeoResult<NeoByteString> {
        let mut hash = Vec::new();
        for i in 0..32 {
            hash.push(data.len() as u8 ^ i as u8 ^ 0xEF);
        }
        Ok(NeoByteString::new(hash))
    }

    pub fn keccak512(data: &NeoByteString) -> NeoResult<NeoByteString> {
        let mut hash = Vec::new();
        for i in 0..64 {
            hash.push(data.len() as u8 ^ i as u8 ^ 0x12);
        }
        Ok(NeoByteString::new(hash))
    }

    pub fn murmur32(data: &NeoByteString, seed: NeoInteger) -> NeoResult<NeoInteger> {
        let hash_value = (data.len() as i32) ^ seed.as_i32() ^ 0x1234_5678;
        Ok(NeoInteger::new(hash_value))
    }

    pub fn verify_signature(
        _message: &NeoByteString,
        signature: &NeoByteString,
        public_key: &NeoByteString,
    ) -> NeoResult<NeoBoolean> {
        Ok(NeoBoolean::new(
            signature.len() == 64 && public_key.len() == 33,
        ))
    }

    pub fn verify_signature_with_recovery(
        _message: &NeoByteString,
        signature: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        let mut recovered = signature.as_slice().to_vec();
        recovered.resize(33, 0u8);
        Ok(NeoByteString::new(recovered))
    }
}

/// Minimal JSON helpers to support tests.
pub struct NeoJSON;

impl NeoJSON {
    pub fn serialize(value: &NeoValue) -> NeoResult<NeoString> {
        let json = match value {
            NeoValue::Integer(i) => format!("{{\"type\":\"integer\",\"value\":{}}}", i.as_i32()),
            NeoValue::Boolean(b) => format!("{{\"type\":\"boolean\",\"value\":{}}}", b.as_bool()),
            NeoValue::String(s) => format!("{{\"type\":\"string\",\"value\":\"{}\"}}", s.as_str()),
            NeoValue::ByteString(bs) => format!("{{\"type\":\"bytestring\",\"len\":{}}}", bs.len()),
            NeoValue::Null => "{\"type\":\"null\"}".to_string(),
            _ => "{\"type\":\"unsupported\"}".to_string(),
        };
        Ok(NeoString::from_str(&json))
    }

    pub fn deserialize(json: &NeoString) -> NeoResult<NeoValue> {
        let text = json.as_str();
        if text.contains("integer") {
            Ok(NeoValue::from(NeoInteger::new(42)))
        } else if text.contains("boolean") {
            Ok(NeoValue::from(NeoBoolean::TRUE))
        } else if text.contains("string") {
            Ok(NeoValue::from(NeoString::from_str("hello")))
        } else if text.contains("bytestring") {
            Ok(NeoValue::from(NeoByteString::from_slice(b"data")))
        } else {
            Ok(NeoValue::Null)
        }
    }
}
