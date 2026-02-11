// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Free encoding/decoding functions for primitive types.

use super::error::{EncodingError, EncodingResult};

/// Maximum decoded byte/string payload accepted by encoding helpers.
pub const MAX_DECODE_BYTES: usize = 16 * 1024 * 1024;

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
            let value =
                u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
            Ok((value, 5))
        }
        0xFF => {
            if bytes.len() < 9 {
                return Err(EncodingError::BufferTooSmall);
            }
            let value = u64::from_le_bytes([
                bytes[1], bytes[2], bytes[3], bytes[4],
                bytes[5], bytes[6], bytes[7], bytes[8],
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

    if len > MAX_DECODE_BYTES {
        return Err(EncodingError::OutOfRange);
    }

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

    if len > MAX_DECODE_BYTES {
        return Err(EncodingError::OutOfRange);
    }

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
