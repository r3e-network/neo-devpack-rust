// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use std::fmt;
use std::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 String type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoString {
    data: String,
}

impl NeoString {
    pub fn new(data: String) -> Self {
        Self { data }
    }

    /// Creates a `NeoString` from a string slice.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self {
            data: String::from(s),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl fmt::Display for NeoString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl std::str::FromStr for NeoString {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            data: String::from(s),
        })
    }
}

impl From<&str> for NeoString {
    fn from(s: &str) -> Self {
        Self {
            data: String::from(s),
        }
    }
}

impl From<String> for NeoString {
    fn from(data: String) -> Self {
        Self { data }
    }
}

impl AsRef<str> for NeoString {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl PartialEq<str> for NeoString {
    fn eq(&self, other: &str) -> bool {
        self.data == other
    }
}

impl PartialEq<&str> for NeoString {
    fn eq(&self, other: &&str) -> bool {
        self.data == *other
    }
}
