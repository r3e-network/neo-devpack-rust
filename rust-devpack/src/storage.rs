// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Storage utilities for Neo N3 smart contracts.
//!
//! This module provides convenient functions for storing and retrieving
//! structured data using JSON serialization.

use crate::{NeoStorage, NeoStorageContext};
use neo_types::{NeoByteString, NeoIterator, NeoResult, NeoStruct, NeoValue};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Reads a JSON-serialized value from a NeoByteString.
///
/// # Type Parameters
/// * `T` - The type to deserialize
///
/// # Arguments
/// * `bytes` - The bytes containing JSON data
///
/// # Returns
/// * `Some(T)` if deserialization succeeds
/// * `None` if deserialization fails
pub fn read_json<T: for<'de> Deserialize<'de>>(bytes: &NeoByteString) -> Option<T> {
    serde_json::from_slice(bytes.as_slice()).ok()
}

/// Writes a value as JSON to a NeoByteString.
///
/// # Type Parameters
/// * `T` - The type to serialize
///
/// # Arguments
/// * `value` - The value to serialize
///
/// # Returns
/// A NeoByteString containing the JSON representation, or an empty byte string on error.
pub fn write_json<T: Serialize>(value: &T) -> NeoByteString {
    match serde_json::to_vec(value) {
        Ok(data) => NeoByteString::from_slice(&data),
        Err(_) => NeoByteString::new(Vec::new()),
    }
}

/// Loads a typed value from storage.
///
/// # Type Parameters
/// * `T` - The type to load
///
/// # Arguments
/// * `ctx` - The storage context
/// * `key` - The storage key
///
/// # Returns
/// * `Some(T)` if the key exists and deserialization succeeds
/// * `None` if the key doesn't exist or deserialization fails
pub fn load<T: for<'de> Deserialize<'de>>(ctx: &NeoStorageContext, key: &[u8]) -> Option<T> {
    let key_bytes = NeoByteString::from_slice(key);
    let bytes = NeoStorage::get(ctx, &key_bytes).ok()?;
    if bytes.is_empty() {
        None
    } else {
        read_json(&bytes)
    }
}

/// Stores a typed value to storage.
///
/// # Type Parameters
/// * `T` - The type to store
///
/// # Arguments
/// * `ctx` - The storage context
/// * `key` - The storage key
/// * `value` - The value to store
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(NeoError)` if storage fails (e.g., read-only context)
pub fn store<T: Serialize>(ctx: &NeoStorageContext, key: &[u8], value: &T) -> NeoResult<()> {
    let key_bytes = NeoByteString::from_slice(key);
    NeoStorage::put(ctx, &key_bytes, &write_json(value))
}

/// Deletes a key from storage.
///
/// # Arguments
/// * `ctx` - The storage context
/// * `key` - The storage key to delete
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(NeoError)` if deletion fails (e.g., read-only context)
pub fn delete(ctx: &NeoStorageContext, key: &[u8]) -> NeoResult<()> {
    let key_bytes = NeoByteString::from_slice(key);
    NeoStorage::delete(ctx, &key_bytes)
}

/// Finds all storage entries with the given prefix.
///
/// # Arguments
/// * `ctx` - The storage context
/// * `prefix` - The key prefix to search for
///
/// # Returns
/// * `Ok(NeoIterator<NeoValue>)` containing matching entries
/// * `Err(NeoError)` if the search fails
pub fn find_prefix(ctx: &NeoStorageContext, prefix: &[u8]) -> NeoResult<NeoIterator<NeoValue>> {
    let prefix_bytes = NeoByteString::from_slice(prefix);
    NeoStorage::find(ctx, &prefix_bytes)
}

/// Creates a NeoStruct entry with key and value fields.
///
/// This is useful for returning storage entries from contract methods.
pub fn struct_entry(key: NeoByteString, value: NeoByteString) -> NeoValue {
    let mut s = NeoStruct::new();
    s.set_field("key", NeoValue::from(key));
    s.set_field("value", NeoValue::from(value));
    NeoValue::from(s)
}

/// Converts a NeoValue containing a ByteString to a JSON value.
///
/// # Arguments
/// * `value` - The NeoValue to convert
///
/// # Returns
/// * `Some(JsonValue)` if the value is a valid ByteString containing JSON
/// * `None` otherwise
pub fn value_to_json(value: &NeoValue) -> Option<JsonValue> {
    value
        .as_byte_string()
        .and_then(|bytes| serde_json::from_slice(bytes.as_slice()).ok())
}
