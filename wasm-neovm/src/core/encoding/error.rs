// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Error types for encoding operations.

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
