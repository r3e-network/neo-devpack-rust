// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Byte writer for building byte sequences.

use super::primitives::{encode_bytes, encode_string, encode_varint};

/// A writer for building byte sequences
#[derive(Debug, Clone, Default)]
pub struct ByteWriter {
    data: Vec<u8>,
}

impl ByteWriter {
    /// Create a new byte writer
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a new byte writer with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Write a single byte
    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.data.push(value);
        self
    }

    /// Write a u16 in little-endian format
    pub fn write_u16_le(&mut self, value: u16) -> &mut Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Write a u32 in little-endian format
    pub fn write_u32_le(&mut self, value: u32) -> &mut Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Write a u64 in little-endian format
    pub fn write_u64_le(&mut self, value: u64) -> &mut Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Write a variable-length integer
    pub fn write_varint(&mut self, value: u64) -> &mut Self {
        self.data.extend_from_slice(&encode_varint(value));
        self
    }

    /// Write a length-prefixed string
    pub fn write_string(&mut self, value: &str) -> &mut Self {
        self.data.extend_from_slice(&encode_string(value));
        self
    }

    /// Write a length-prefixed byte array
    pub fn write_bytes(&mut self, value: &[u8]) -> &mut Self {
        self.data.extend_from_slice(&encode_bytes(value));
        self
    }

    /// Write raw bytes
    pub fn write_raw(&mut self, value: &[u8]) -> &mut Self {
        self.data.extend_from_slice(value);
        self
    }

    /// Get the written data
    pub fn finish(self) -> Vec<u8> {
        self.data
    }

    /// Get a reference to the data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the current length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the writer
    pub fn clear(&mut self) {
        self.data.clear();
    }
}
