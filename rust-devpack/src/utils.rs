// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! General utility functions for Neo N3 smart contracts.

use neo_types::{NeoByteString, NeoStruct, NeoValue};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value as JsonValue};

/// Deserializes a NeoByteString as JSON.
///
/// # Type Parameters
/// * `T` - The target type for deserialization
///
/// # Returns
/// * `Some(T)` if deserialization succeeds
/// * `None` if the bytes are not valid JSON for the target type
pub fn bytes_to_json<T: for<'de> Deserialize<'de>>(bytes: &NeoByteString) -> Option<T> {
    serde_json::from_slice(bytes.as_slice()).ok()
}

/// Serializes a value to JSON and returns it as a NeoByteString.
///
/// # Type Parameters
/// * `T` - The type to serialize
///
/// # Returns
/// * `Ok(NeoByteString)` containing JSON bytes
/// * `Err(NeoError)` if serialization fails
pub fn json_to_bytes<T: Serialize>(value: &T) -> neo_types::NeoResult<NeoByteString> {
    let data = serde_json::to_vec(value).map_err(|err| {
        neo_types::NeoError::new(&format!("failed to serialize JSON bytes: {err}"))
    })?;
    Ok(NeoByteString::from_slice(&data))
}

/// Creates a storage entry struct with key and value fields.
///
/// This is a convenience function for creating the standard storage
/// entry format used by Neo N3 storage find operations.
pub fn storage_struct(key: &NeoByteString, value: &NeoByteString) -> NeoValue {
    let mut entry = NeoStruct::new();
    entry.set_field("key", NeoValue::from(key.clone()));
    entry.set_field("value", NeoValue::from(value.clone()));
    NeoValue::from(entry)
}

/// Extracts JSON from a NeoValue containing a ByteString.
///
/// # Returns
/// * `Some(JsonValue)` if the value is a ByteString containing valid JSON
/// * `None` otherwise
pub fn json_from_value(value: &NeoValue) -> Option<JsonValue> {
    value
        .as_byte_string()
        .and_then(|bytes| serde_json::from_slice(bytes.as_slice()).ok())
}
