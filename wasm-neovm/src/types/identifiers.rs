// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Type-safe identifier wrappers.

use std::fmt;
use std::ops::Deref;

/// A contract name (validated, non-empty string)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractName(String);

impl Default for ContractName {
    /// Returns a default contract name of `"Contract"`.
    fn default() -> Self {
        Self("Contract".to_string())
    }
}

impl ContractName {
    /// Create a new contract name
    ///
    /// # Panics
    /// Panics if the name is empty
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.is_empty(), "Contract name cannot be empty");
        Self(name)
    }

    /// Try to create a contract name, returning None if empty
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if name.is_empty() {
            None
        } else {
            Some(Self(name))
        }
    }

    /// Get the name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner String
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for ContractName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ContractName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContractName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ContractName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ContractName {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_name() {
        let name = ContractName::new("TestContract");
        assert_eq!(name.as_str(), "TestContract");
        assert_eq!(name.to_string(), "TestContract");
    }

    #[test]
    fn test_contract_name_from_str() {
        let name: ContractName = "TestContract".into();
        assert_eq!(name.as_str(), "TestContract");
    }
}
