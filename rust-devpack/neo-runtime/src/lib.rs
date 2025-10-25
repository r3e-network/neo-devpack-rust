//! Neo N3 Runtime facade
//!
//! This crate provides a lightweight façade over the Neo runtime surface so
//! integration tests and examples can exercise the canonical syscall
//! catalogue without depending on a full node implementation. The
//! implementation intentionally returns deterministic placeholder values –
//! enough to validate wiring and type conversions while remaining
//! self-contained for unit tests.

use neo_syscalls::NeoVMSyscall;
use neo_types::*;
use num_bigint::BigInt;
use serde_json::{json, Value as JsonValue};
use std::vec::Vec;

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
    ) -> NeoResult<NeoIterator<NeoValue>> {
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
        let json_value = Self::value_to_json(value)?;
        let text = serde_json::to_string(&json_value)
            .map_err(|err| NeoError::new(&format!("failed to serialize JSON: {err}")))?;
        Ok(NeoString::from_str(&text))
    }

    pub fn deserialize(json: &NeoString) -> NeoResult<NeoValue> {
        let parsed: JsonValue = serde_json::from_str(json.as_str())
            .map_err(|err| NeoError::new(&format!("failed to parse JSON: {err}")))?;
        Self::json_to_value(&parsed)
    }

    fn value_to_json(value: &NeoValue) -> NeoResult<JsonValue> {
        match value {
            NeoValue::Integer(i) => Ok(json!({
                "type": "Integer",
                "value": i.as_bigint().to_string()
            })),
            NeoValue::Boolean(b) => Ok(json!({
                "type": "Boolean",
                "value": b.as_bool()
            })),
            NeoValue::String(s) => Ok(json!({
                "type": "String",
                "value": s.as_str()
            })),
            NeoValue::ByteString(bs) => {
                let bytes: Vec<JsonValue> = bs
                    .as_slice()
                    .iter()
                    .map(|b| JsonValue::from(*b))
                    .collect();
                Ok(json!({ "type": "ByteString", "value": bytes }))
            }
            NeoValue::Array(arr) => {
                let mut values = Vec::new();
                for item in arr.iter() {
                    values.push(Self::value_to_json(item)?);
                }
                Ok(json!({ "type": "Array", "value": values }))
            }
            NeoValue::Map(map) => {
                let mut entries = Vec::new();
                for (key, value) in map.iter() {
                    entries.push(json!({
                        "key": Self::value_to_json(key)?,
                        "value": Self::value_to_json(value)?,
                    }));
                }
                Ok(json!({ "type": "Map", "value": entries }))
            }
            NeoValue::Struct(st) => {
                let mut fields_json = Vec::new();
                for (name, value) in st.iter() {
                    fields_json.push(json!({
                        "name": name,
                        "value": Self::value_to_json(value)?,
                    }));
                }
                Ok(json!({ "type": "Struct", "value": fields_json }))
            }
            NeoValue::Null => Ok(json!({ "type": "Null" })),
        }
    }

    fn json_to_value(json: &JsonValue) -> NeoResult<NeoValue> {
        let obj = json
            .as_object()
            .ok_or_else(|| NeoError::InvalidType)?;
        let kind = obj
            .get("type")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| NeoError::InvalidType)?;

        match kind {
            "Integer" => {
                let value_str = obj
                    .get("value")
                    .and_then(JsonValue::as_str)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let bigint = BigInt::parse_bytes(value_str.as_bytes(), 10)
                    .ok_or_else(|| NeoError::InvalidType)?;
                Ok(NeoValue::from(NeoInteger::new(bigint)))
            }
            "Boolean" => {
                let value = obj
                    .get("value")
                    .and_then(JsonValue::as_bool)
                    .ok_or_else(|| NeoError::InvalidType)?;
                Ok(NeoValue::from(NeoBoolean::new(value)))
            }
            "String" => {
                let value = obj
                    .get("value")
                    .and_then(JsonValue::as_str)
                    .ok_or_else(|| NeoError::InvalidType)?;
                Ok(NeoValue::from(NeoString::from_str(value)))
            }
            "ByteString" => {
                let array = obj
                    .get("value")
                    .and_then(JsonValue::as_array)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let mut bytes = Vec::with_capacity(array.len());
                for item in array {
                    let byte = item
                        .as_u64()
                        .ok_or_else(|| NeoError::InvalidType)?;
                    bytes.push(byte as u8);
                }
                Ok(NeoValue::from(NeoByteString::new(bytes)))
            }
            "Array" => {
                let array = obj
                    .get("value")
                    .and_then(JsonValue::as_array)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let mut values = NeoArray::new();
                for item in array {
                    values.push(Self::json_to_value(item)?);
                }
                Ok(NeoValue::from(values))
            }
            "Map" => {
                let entries = obj
                    .get("value")
                    .and_then(JsonValue::as_array)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let mut map = NeoMap::new();
                for entry in entries {
                    let entry_obj = entry
                        .as_object()
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let key_json = entry_obj
                        .get("key")
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let value_json = entry_obj
                        .get("value")
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let key = Self::json_to_value(key_json)?;
                    let value = Self::json_to_value(value_json)?;
                    map.insert(key, value);
                }
                Ok(NeoValue::from(map))
            }
            "Struct" => {
                let fields = obj
                    .get("value")
                    .and_then(JsonValue::as_array)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let mut result = NeoStruct::new();
                for field in fields {
                    let field_obj = field
                        .as_object()
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let name = field_obj
                        .get("name")
                        .and_then(JsonValue::as_str)
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let value_json = field_obj
                        .get("value")
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let value = Self::json_to_value(value_json)?;
                    result = result.with_field(name, value);
                }
                Ok(NeoValue::from(result))
            }
            "Null" => Ok(NeoValue::Null),
            _ => Err(NeoError::InvalidType),
        }
    }
}
