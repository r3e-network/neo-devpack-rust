// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Encoding utilities for NeoVM types
//!
//! This module provides encoding/decoding for types used in the
//! NeoVM ecosystem.

use std::fmt;

/// Error type for encoding operations
#[derive(Debug, Clone, PartialEq)]
pub enum EncodingError {
    /// Buffer too small for the operation
    BufferTooSmall,
    /// Invalid input data
    InvalidData(String),
    /// Value out of range
    OutOfRange,
    /// Unsupported encoding
    UnsupportedEncoding,
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooSmall => write!(f, "Buffer too small"),
            Self::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            Self::OutOfRange => write!(f, "Value out of range"),
            Self::UnsupportedEncoding => write!(f, "Unsupported encoding"),
        }
    }
}

impl std::error::Error for EncodingError {}

/// Result type for encoding operations
pub type EncodingResult<T> = Result<T, EncodingError>;

/// Encode a variable-length integer (compact encoding)
pub fn encode_varint(value: u64) -> Vec<u8> {
    if value < 253 {
        vec![value as u8]
    } else if value <= u16::MAX as u64 {
        let mut result = vec![0xFD];
        result.extend_from_slice(&(value as u16).to_le_bytes());
        result
    } else if value <= u32::MAX as u64 {
        let mut result = vec![0xFE];
        result.extend_from_slice(&(value as u32).to_le_bytes());
        result
    } else {
        let mut result = vec![0xFF];
        result.extend_from_slice(&value.to_le_bytes());
        result
    }
}

/// Decode a variable-length integer
pub fn decode_varint(bytes: &[u8]) -> EncodingResult<(u64, usize)> {
    if bytes.is_empty() {
        return Err(EncodingError::BufferTooSmall);
    }

    match bytes[0] {
        n if n < 0xFD => Ok((n as u64, 1)),
        0xFD => {
            if bytes.len() < 3 {
                return Err(EncodingError::BufferTooSmall);
            }
            let value = u16::from_le_bytes([bytes[1], bytes[2]]) as u64;
            Ok((value, 3))
        }
        0xFE => {
            if bytes.len() < 5 {
                return Err(EncodingError::BufferTooSmall);
            }
            let value = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
            Ok((value, 5))
        }
        0xFF => {
            if bytes.len() < 9 {
                return Err(EncodingError::BufferTooSmall);
            }
            let value = u64::from_le_bytes([
                bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            ]);
            Ok((value, 9))
        }
        _ => Err(EncodingError::InvalidData(
            "Unknown varint prefix".to_string(),
        )),
    }
}

/// Encode a string with length prefix
pub fn encode_string(s: &str) -> Vec<u8> {
    let mut result = encode_varint(s.len() as u64);
    result.extend_from_slice(s.as_bytes());
    result
}

/// Decode a length-prefixed string
pub fn decode_string(bytes: &[u8]) -> EncodingResult<(String, usize)> {
    let (len, prefix_len) = decode_varint(bytes)?;
    let len = len as usize;

    if bytes.len() < prefix_len + len {
        return Err(EncodingError::BufferTooSmall);
    }

    let s = String::from_utf8(bytes[prefix_len..prefix_len + len].to_vec())
        .map_err(|_| EncodingError::InvalidData("Invalid UTF-8".to_string()))?;

    Ok((s, prefix_len + len))
}

/// Encode a byte array with length prefix
pub fn encode_bytes(data: &[u8]) -> Vec<u8> {
    let mut result = encode_varint(data.len() as u64);
    result.extend_from_slice(data);
    result
}

/// Decode a length-prefixed byte array
pub fn decode_bytes(bytes: &[u8]) -> EncodingResult<(Vec<u8>, usize)> {
    let (len, prefix_len) = decode_varint(bytes)?;
    let len = len as usize;

    if bytes.len() < prefix_len + len {
        return Err(EncodingError::BufferTooSmall);
    }

    Ok((
        bytes[prefix_len..prefix_len + len].to_vec(),
        prefix_len + len,
    ))
}

/// Encode a boolean
pub fn encode_bool(value: bool) -> u8 {
    if value {
        0x01
    } else {
        0x00
    }
}

/// Decode a boolean
pub fn decode_bool(byte: u8) -> EncodingResult<bool> {
    match byte {
        0x00 => Ok(false),
        0x01 => Ok(true),
        _ => Err(EncodingError::InvalidData(
            "Invalid boolean value".to_string(),
        )),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encoding() {
        // Small values
        assert_eq!(encode_varint(0), vec![0]);
        assert_eq!(encode_varint(252), vec![252]);

        // 16-bit values
        assert_eq!(encode_varint(253), vec![0xFD, 0xFD, 0x00]);
        assert_eq!(encode_varint(1000), vec![0xFD, 0xE8, 0x03]);

        // 32-bit values
        assert_eq!(encode_varint(65536), vec![0xFE, 0x00, 0x00, 0x01, 0x00]);

        // 64-bit values
        assert_eq!(
            encode_varint(u64::MAX),
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        );
    }

    #[test]
    fn test_varint_roundtrip() {
        let test_values = [0u64, 100, 253, 1000, 65536, 100000, u64::MAX];

        for value in test_values {
            let encoded = encode_varint(value);
            let (decoded, _) = decode_varint(&encoded).unwrap();
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_string_encoding() {
        let test_strings = ["", "hello", "Hello, 世界!"];

        for s in test_strings {
            let encoded = encode_string(s);
            let (decoded, _) = decode_string(&encoded).unwrap();
            assert_eq!(s, decoded);
        }
    }

    #[test]
    fn test_byte_writer() {
        let mut writer = ByteWriter::with_capacity(32);
        writer
            .write_u8(0x01)
            .write_u16_le(0x1234)
            .write_u32_le(0x567890AB)
            .write_string("test");

        let bytes = writer.finish();
        assert!(!bytes.is_empty());

        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_u8().unwrap(), 0x01);
        assert_eq!(reader.read_u16_le().unwrap(), 0x1234);
        assert_eq!(reader.read_u32_le().unwrap(), 0x567890AB);
        assert_eq!(reader.read_string().unwrap(), "test");
    }

    #[test]
    fn test_byte_reader_seek() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = ByteReader::new(&data);

        assert_eq!(reader.read_u8().unwrap(), 0x01);
        reader.seek(3).unwrap();
        assert_eq!(reader.read_u8().unwrap(), 0x04);
    }
}
