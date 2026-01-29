use std::fmt;
use std::string::String;

/// Neo N3 Error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeoError {
    InvalidOperation,
    InvalidArgument,
    InvalidType,
    OutOfBounds,
    DivisionByZero,
    Overflow,
    Underflow,
    NullReference,
    InvalidState,
    Custom(String),
}

impl fmt::Display for NeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoError::InvalidOperation => write!(f, "Invalid operation"),
            NeoError::InvalidArgument => write!(f, "Invalid argument"),
            NeoError::InvalidType => write!(f, "Invalid type"),
            NeoError::OutOfBounds => write!(f, "Out of bounds"),
            NeoError::DivisionByZero => write!(f, "Division by zero"),
            NeoError::Overflow => write!(f, "Overflow"),
            NeoError::Underflow => write!(f, "Underflow"),
            NeoError::NullReference => write!(f, "Null reference"),
            NeoError::InvalidState => write!(f, "Invalid state"),
            NeoError::Custom(msg) => write!(f, "Custom error: {}", msg),
        }
    }
}

impl NeoError {
    pub fn new(message: &str) -> Self {
        NeoError::Custom(message.to_string())
    }

    pub fn message(&self) -> &str {
        match self {
            NeoError::Custom(msg) => msg,
            _ => "Unknown error",
        }
    }
}

/// Neo N3 Result type
pub type NeoResult<T> = Result<T, NeoError>;
