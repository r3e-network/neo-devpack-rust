// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Serialization and deserialization utilities for Neo N3 smart contracts.
//!
//! This module provides binary serialization using bincode, which is efficient
//! for storage and cross-contract communication.

use neo_types::{NeoError, NeoResult};
use serde::de::DeserializeOwned;
use serde::Serialize;

use bincode::Options;

const MAX_CODEC_BYTES: u64 = 1024 * 1024;

/// Serializes a value to bytes using bincode.
///
/// # Type Parameters
/// * `T` - The type to serialize, must implement `Serialize`
///
/// # Arguments
/// * `value` - A reference to the value to serialize
///
/// # Returns
/// * `Ok(Vec<u8>)` - The serialized bytes on success
/// * `Err(NeoError)` - If serialization fails
///
/// # Examples
///
/// ```
/// use neo_devpack::codec::serialize;
///
/// let value = 42i32;
/// let bytes = serialize(&value).unwrap();
/// ```
pub fn serialize<T: Serialize>(value: &T) -> NeoResult<Vec<u8>> {
    bincode::DefaultOptions::new()
        .with_limit(MAX_CODEC_BYTES)
        .serialize(value)
        .map_err(|err| NeoError::new(&format!("Failed to serialize value: {err}")))
}

/// Deserializes bytes to a value using bincode.
///
/// # Type Parameters
/// * `T` - The type to deserialize, must implement `DeserializeOwned`
///
/// # Arguments
/// * `bytes` - The bytes to deserialize from
///
/// # Returns
/// * `Ok(T)` - The deserialized value on success
/// * `Err(NeoError)` - If deserialization fails (e.g., invalid format)
///
/// # Examples
///
/// ```
/// use neo_devpack::codec::{serialize, deserialize};
///
/// let value = 42i32;
/// let bytes = serialize(&value).unwrap();
/// let restored: i32 = deserialize(&bytes).unwrap();
/// assert_eq!(value, restored);
/// ```
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> NeoResult<T> {
    bincode::DefaultOptions::new()
        .with_limit(MAX_CODEC_BYTES)
        .deserialize(bytes)
        .map_err(|err| NeoError::new(&format!("Failed to deserialize bytes: {err}")))
}
