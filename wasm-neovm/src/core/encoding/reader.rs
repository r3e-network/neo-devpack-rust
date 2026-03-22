// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Byte reader for parsing byte sequences.

use super::error::{EncodingError, EncodingResult};
use super::primitives::{decode_bytes, decode_string, decode_varint};

/// A reader for parsing byte sequences
#[derive(Debug, Clone)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> ByteReader<'a> {
    /// Create a new byte reader
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Read a single byte
    pub fn read_u8(&mut self) -> EncodingResult<u8> {
        if self.position >= self.data.len() {
            return Err(EncodingError::BufferTooSmall);
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    /// Read a u16 in little-endian format
    pub fn read_u16_le(&mut self) -> EncodingResult<u16> {
        if self.position + 2 > self.data.len() {
            return Err(EncodingError::BufferTooSmall);
        }
        let value = u16::from_le_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;
        Ok(value)
    }

    /// Read a u32 in little-endian format
    pub fn read_u32_le(&mut self) -> EncodingResult<u32> {
        if self.position + 4 > self.data.len() {
            return Err(EncodingError::BufferTooSmall);
        }
        let value = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        Ok(value)
    }

    /// Read a u64 in little-endian format
    pub fn read_u64_le(&mut self) -> EncodingResult<u64> {
        if self.position + 8 > self.data.len() {
            return Err(EncodingError::BufferTooSmall);
        }
        let value = u64::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
            self.data[self.position + 4],
            self.data[self.position + 5],
            self.data[self.position + 6],
            self.data[self.position + 7],
        ]);
        self.position += 8;
        Ok(value)
    }

    /// Read a variable-length integer
    pub fn read_varint(&mut self) -> EncodingResult<u64> {
        let (value, len) = decode_varint(&self.data[self.position..])?;
        self.position += len;
        Ok(value)
    }

    /// Read a length-prefixed string
    pub fn read_string(&mut self) -> EncodingResult<String> {
        let (s, len) = decode_string(&self.data[self.position..])?;
        self.position += len;
        Ok(s)
    }

    /// Read a length-prefixed byte array
    pub fn read_bytes(&mut self) -> EncodingResult<Vec<u8>> {
        let (data, len) = decode_bytes(&self.data[self.position..])?;
        self.position += len;
        Ok(data)
    }

    /// Read raw bytes
    pub fn read_raw(&mut self, len: usize) -> EncodingResult<&[u8]> {
        if self.position + len > self.data.len() {
            return Err(EncodingError::BufferTooSmall);
        }
        let value = &self.data[self.position..self.position + len];
        self.position += len;
        Ok(value)
    }

    /// Get the current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> &[u8] {
        &self.data[self.position..]
    }

    /// Check if at end of data
    pub fn is_at_end(&self) -> bool {
        self.position >= self.data.len()
    }

    /// Get total length of data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if at end of data
    pub fn is_empty(&self) -> bool {
        self.position >= self.data.len()
    }

    /// Set position
    pub fn seek(&mut self, position: usize) -> EncodingResult<()> {
        if position > self.data.len() {
            return Err(EncodingError::OutOfRange);
        }
        self.position = position;
        Ok(())
    }
}
