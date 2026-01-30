// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use std::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 String type
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoString {
    data: String,
}

impl NeoString {
    pub fn new(data: String) -> Self {
        Self { data }
    }

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

impl From<&str> for NeoString {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for NeoString {
    fn from(data: String) -> Self {
        Self { data }
    }
}
