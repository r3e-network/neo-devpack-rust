use neo_types::{NeoByteString, NeoStruct, NeoValue};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value as JsonValue};

pub fn bytes_to_json<T: for<'de> Deserialize<'de>>(bytes: &NeoByteString) -> Option<T> {
    serde_json::from_slice(bytes.as_slice()).ok()
}

pub fn json_to_bytes<T: Serialize>(value: &T) -> NeoByteString {
    let data = serde_json::to_vec(value).unwrap_or_default();
    NeoByteString::from_slice(&data)
}

pub fn storage_struct(key: &NeoByteString, value: &NeoByteString) -> NeoValue {
    let mut entry = NeoStruct::new();
    entry.set_field("key", NeoValue::from(key.clone()));
    entry.set_field("value", NeoValue::from(value.clone()));
    NeoValue::from(entry)
}

pub fn json_from_value(value: &NeoValue) -> Option<JsonValue> {
    if let Some(bytes) = value.as_byte_string() {
        serde_json::from_slice(bytes.as_slice()).ok()
    } else {
        None
    }
}
