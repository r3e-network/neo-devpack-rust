// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! General utility functions for Neo N3 smart contracts.
//!
//! For JSON serialization helpers, prefer the canonical functions in
//! [`crate::storage`] (`read_json`, `write_json`, `struct_entry`, `value_to_json`).
//! The functions here are thin wrappers kept for backward compatibility.

use neo_types::{NeoByteString, NeoValue};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Deserializes a NeoByteString as JSON.
///
/// This is equivalent to [`crate::storage::read_json`].
pub fn bytes_to_json<T: for<'de> Deserialize<'de>>(bytes: &NeoByteString) -> Option<T> {
    crate::storage::read_json(bytes)
}

/// Serializes a value to JSON and returns it as a NeoByteString.
///
/// This is equivalent to [`crate::storage::write_json`].
pub fn json_to_bytes<T: Serialize>(value: &T) -> neo_types::NeoResult<NeoByteString> {
    crate::storage::write_json(value)
}

/// Creates a storage entry struct with key and value fields.
///
/// This is equivalent to [`crate::storage::struct_entry`].
pub fn storage_struct(key: &NeoByteString, value: &NeoByteString) -> NeoValue {
    crate::storage::struct_entry(key.clone(), value.clone())
}

/// Extracts JSON from a NeoValue containing a ByteString.
///
/// This is equivalent to [`crate::storage::value_to_json`].
pub fn json_from_value(value: &NeoValue) -> Option<JsonValue> {
    crate::storage::value_to_json(value)
}
