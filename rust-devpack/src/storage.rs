use crate::{NeoStorage, NeoStorageContext};
use neo_types::{NeoByteString, NeoIterator, NeoResult, NeoStruct, NeoValue};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

pub fn read_json<T: for<'de> Deserialize<'de>>(bytes: &NeoByteString) -> Option<T> {
    serde_json::from_slice(bytes.as_slice()).ok()
}

pub fn write_json<T: Serialize>(value: &T) -> NeoByteString {
    let data = serde_json::to_vec(value).unwrap_or_default();
    NeoByteString::from_slice(&data)
}

pub fn load<T: for<'de> Deserialize<'de>>(ctx: &NeoStorageContext, key: &[u8]) -> Option<T> {
    let key_bytes = NeoByteString::from_slice(key);
    let bytes = NeoStorage::get(ctx, &key_bytes).ok()?;
    if bytes.is_empty() {
        None
    } else {
        read_json(&bytes)
    }
}

pub fn store<T: Serialize>(ctx: &NeoStorageContext, key: &[u8], value: &T) -> NeoResult<()> {
    let key_bytes = NeoByteString::from_slice(key);
    NeoStorage::put(ctx, &key_bytes, &write_json(value))
}

pub fn delete(ctx: &NeoStorageContext, key: &[u8]) -> NeoResult<()> {
    let key_bytes = NeoByteString::from_slice(key);
    NeoStorage::delete(ctx, &key_bytes)
}

pub fn find_prefix(ctx: &NeoStorageContext, prefix: &[u8]) -> NeoResult<NeoIterator<NeoValue>> {
    let prefix_bytes = NeoByteString::from_slice(prefix);
    NeoStorage::find(ctx, &prefix_bytes)
}

pub fn struct_entry(key: NeoByteString, value: NeoByteString) -> NeoValue {
    let mut s = NeoStruct::new();
    s.set_field("key", NeoValue::from(key));
    s.set_field("value", NeoValue::from(value));
    NeoValue::from(s)
}

pub fn value_to_json(value: &NeoValue) -> Option<JsonValue> {
    value
        .as_byte_string()
        .and_then(|bytes| serde_json::from_slice(bytes.as_slice()).ok())
}
