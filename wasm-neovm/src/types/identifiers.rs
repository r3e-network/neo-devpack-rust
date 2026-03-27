// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Type-safe identifier wrappers
//!
//! These newtypes provide compile-time guarantees that identifiers
//! are used correctly and cannot be confused with one another.

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

/// A method index in the function table (guaranteed to be valid)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MethodIndex(u32);

impl MethodIndex {
    /// Create a new method index
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Get the index value
    pub const fn value(&self) -> u32 {
        self.0
    }

    /// Convert to usize for array indexing
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Deref for MethodIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for MethodIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for MethodIndex {
    fn from(v: u32) -> Self {
        Self::new(v)
    }
}

impl From<usize> for MethodIndex {
    fn from(v: usize) -> Self {
        Self::new(v as u32)
    }
}

/// A local variable index (guaranteed to be valid)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LocalIndex(u32);

impl LocalIndex {
    /// Create a new local index
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Get the index value
    pub const fn value(&self) -> u32 {
        self.0
    }

    /// Convert to usize for array indexing
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Deref for LocalIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for LocalIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for LocalIndex {
    fn from(v: u32) -> Self {
        Self::new(v)
    }
}

impl From<usize> for LocalIndex {
    fn from(v: usize) -> Self {
        Self::new(v as u32)
    }
}

/// A global variable index (guaranteed to be valid)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct GlobalIndex(u32);

impl GlobalIndex {
    /// Create a new global index
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Get the index value
    pub const fn value(&self) -> u32 {
        self.0
    }

    /// Convert to usize for array indexing
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Deref for GlobalIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for GlobalIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for GlobalIndex {
    fn from(v: u32) -> Self {
        Self::new(v)
    }
}

impl From<usize> for GlobalIndex {
    fn from(v: usize) -> Self {
        Self::new(v as u32)
    }
}

/// A memory offset (guaranteed to be valid)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MemoryOffset(u32);

impl MemoryOffset {
    /// Create a new memory offset
    pub const fn new(offset: u32) -> Self {
        Self(offset)
    }

    /// Get the offset value
    pub const fn value(&self) -> u32 {
        self.0
    }

    /// Convert to usize for array indexing
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    /// Add another offset, returning a new offset
    pub const fn add(&self, other: MemoryOffset) -> Self {
        Self(self.0 + other.0)
    }
}

impl Deref for MemoryOffset {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for MemoryOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for MemoryOffset {
    fn from(v: u32) -> Self {
        Self::new(v)
    }
}

impl From<usize> for MemoryOffset {
    fn from(v: usize) -> Self {
        Self::new(v as u32)
    }
}

/// A bytecode offset in the generated NeoVM script
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BytecodeOffset(usize);

impl BytecodeOffset {
    /// Create a new bytecode offset
    pub const fn new(offset: usize) -> Self {
        Self(offset)
    }

    /// Get the offset value
    pub const fn value(&self) -> usize {
        self.0
    }

    /// Add bytes to the offset
    pub const fn add(&self, bytes: usize) -> Self {
        Self(self.0 + bytes)
    }
}

impl Deref for BytecodeOffset {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for BytecodeOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:04x}", self.0)
    }
}

impl From<usize> for BytecodeOffset {
    fn from(v: usize) -> Self {
        Self::new(v)
    }
}

/// A syscall descriptor (validated format)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyscallDescriptor(String);

impl SyscallDescriptor {
    /// Create a new syscall descriptor
    ///
    /// # Panics
    /// Panics if the descriptor doesn't match the expected format (e.g., "System.Runtime.GetTime")
    pub fn new(descriptor: impl Into<String>) -> Self {
        let descriptor = descriptor.into();
        // Basic validation - should contain dots and follow pattern
        assert!(
            descriptor.contains('.'),
            "Syscall descriptor must contain dots: {}",
            descriptor
        );
        Self(descriptor)
    }

    /// Try to create a syscall descriptor, returning None if invalid
    pub fn try_new(descriptor: impl Into<String>) -> Option<Self> {
        let descriptor = descriptor.into();
        if descriptor.contains('.') {
            Some(Self(descriptor))
        } else {
            None
        }
    }

    /// Get the descriptor as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner String
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for SyscallDescriptor {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for SyscallDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SyscallDescriptor {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for SyscallDescriptor {
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

    #[test]
    fn test_method_index() {
        let idx = MethodIndex::new(42);
        assert_eq!(idx.value(), 42);
        assert_eq!(idx.as_usize(), 42);
    }

    #[test]
    fn test_local_index() {
        let idx = LocalIndex::new(5);
        assert_eq!(idx.value(), 5);
    }

    #[test]
    fn test_global_index() {
        let idx = GlobalIndex::new(3);
        assert_eq!(idx.value(), 3);
    }

    #[test]
    fn test_memory_offset() {
        let off = MemoryOffset::new(1024);
        assert_eq!(off.value(), 1024);
        assert_eq!(off.add(MemoryOffset::new(256)).value(), 1280);
    }

    #[test]
    fn test_bytecode_offset() {
        let off = BytecodeOffset::new(0x100);
        assert_eq!(off.value(), 256);
        assert_eq!(off.add(16).value(), 272);
    }

    #[test]
    fn test_syscall_descriptor() {
        let desc = SyscallDescriptor::new("System.Runtime.GetTime");
        assert_eq!(desc.as_str(), "System.Runtime.GetTime");
    }

    #[test]
    fn test_syscall_descriptor_try_new() {
        assert!(SyscallDescriptor::try_new("System.Runtime.GetTime").is_some());
        assert!(SyscallDescriptor::try_new("invalid").is_none());
    }
}
