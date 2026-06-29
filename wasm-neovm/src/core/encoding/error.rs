// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Error types for encoding operations.

/// Error type for encoding operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EncodingError {
    /// Buffer too small for the operation
    #[error("Buffer too small")]
    BufferTooSmall,
    /// Invalid input data
    #[error("Invalid data: {0}")]
    InvalidData(String),
    /// Value out of range
    #[error("Value out of range")]
    OutOfRange,
    /// Unsupported encoding
    #[error("Unsupported encoding")]
    UnsupportedEncoding,
}

/// Result type for encoding operations
pub type EncodingResult<T> = Result<T, EncodingError>;
