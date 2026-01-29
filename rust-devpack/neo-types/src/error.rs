use std::fmt;
use std::string::String;

/// Neo N3 Error type
///
/// Represents all possible error conditions that can occur during
/// Neo N3 smart contract execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeoError {
    /// Operation is not valid in the current context
    InvalidOperation,
    /// Argument has an invalid value or type
    InvalidArgument,
    /// Type mismatch encountered during execution
    InvalidType,
    /// Index or offset is out of valid bounds
    OutOfBounds,
    /// Attempted division by zero
    DivisionByZero,
    /// Arithmetic overflow occurred
    Overflow,
    /// Arithmetic underflow occurred
    Underflow,
    /// Dereferenced a null or invalid reference
    NullReference,
    /// Internal state is invalid or corrupted
    InvalidState,
    /// Application-specific error with custom message
    Custom(String),
}

impl fmt::Display for NeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoError::InvalidOperation => write!(f, "Invalid operation: the requested operation cannot be performed in the current context"),
            NeoError::InvalidArgument => write!(f, "Invalid argument: one or more arguments have invalid values or types"),
            NeoError::InvalidType => write!(f, "Invalid type: type mismatch encountered during execution"),
            NeoError::OutOfBounds => write!(f, "Out of bounds: index or offset exceeds valid range"),
            NeoError::DivisionByZero => write!(f, "Division by zero: cannot divide by zero"),
            NeoError::Overflow => write!(f, "Overflow: arithmetic operation resulted in overflow"),
            NeoError::Underflow => write!(f, "Underflow: arithmetic operation resulted in underflow"),
            NeoError::NullReference => write!(f, "Null reference: attempted to access a null or invalid reference"),
            NeoError::InvalidState => write!(f, "Invalid state: internal state is invalid or corrupted"),
            NeoError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl NeoError {
    /// Creates a new custom error with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use neo_types::NeoError;
    ///
    /// let err = NeoError::new("Custom error message");
    /// ```
    pub fn new(message: &str) -> Self {
        NeoError::Custom(message.to_string())
    }

    /// Returns the error message if this is a custom error, otherwise returns a generic description.
    pub fn message(&self) -> &str {
        match self {
            NeoError::Custom(msg) => msg,
            _ => self.as_str(),
        }
    }

    /// Returns a static string description of the error variant.
    pub fn as_str(&self) -> &'static str {
        match self {
            NeoError::InvalidOperation => "InvalidOperation",
            NeoError::InvalidArgument => "InvalidArgument",
            NeoError::InvalidType => "InvalidType",
            NeoError::OutOfBounds => "OutOfBounds",
            NeoError::DivisionByZero => "DivisionByZero",
            NeoError::Overflow => "Overflow",
            NeoError::Underflow => "Underflow",
            NeoError::NullReference => "NullReference",
            NeoError::InvalidState => "InvalidState",
            NeoError::Custom(_) => "Custom",
        }
    }
}

/// Neo N3 Result type
pub type NeoResult<T> = Result<T, NeoError>;
