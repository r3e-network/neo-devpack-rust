// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Move runtime support for NeoVM
//!
//! This module provides runtime helpers that emulate Move-specific
//! semantics on NeoVM, particularly around resource types.

use std::collections::HashMap;

/// Resource ownership tracker
///
/// Move's linear type system requires that resources cannot be copied
/// or implicitly dropped. This tracker enforces those semantics at runtime.
#[derive(Debug, Default)]
pub struct ResourceTracker {
    /// Map from (address, type) -> exists
    resources: HashMap<(Vec<u8>, String), bool>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a resource exists at an address
    pub fn exists(&self, address: &[u8], type_name: &str) -> bool {
        self.resources
            .get(&(address.to_vec(), type_name.to_string()))
            .copied()
            .unwrap_or(false)
    }

    /// Move a resource to an address (move_to)
    ///
    /// Fails if resource already exists at that address
    pub fn move_to(&mut self, address: &[u8], type_name: &str) -> Result<(), ResourceError> {
        let key = (address.to_vec(), type_name.to_string());
        if self.resources.get(&key).copied().unwrap_or(false) {
            return Err(ResourceError::AlreadyExists {
                type_name: type_name.to_string(),
            });
        }
        self.resources.insert(key, true);
        Ok(())
    }

    /// Move a resource from an address (move_from)
    ///
    /// Fails if resource does not exist at that address
    pub fn move_from(&mut self, address: &[u8], type_name: &str) -> Result<(), ResourceError> {
        let key = (address.to_vec(), type_name.to_string());
        if !self.resources.get(&key).copied().unwrap_or(false) {
            return Err(ResourceError::NotFound {
                type_name: type_name.to_string(),
            });
        }
        self.resources.remove(&key);
        Ok(())
    }

    /// Borrow a resource (creates a reference)
    pub fn borrow(&self, address: &[u8], type_name: &str) -> Result<(), ResourceError> {
        if !self.exists(address, type_name) {
            return Err(ResourceError::NotFound {
                type_name: type_name.to_string(),
            });
        }
        Ok(())
    }
}

/// Resource operation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ResourceError {
    #[error("resource '{type_name}' already exists at address")]
    AlreadyExists { type_name: String },

    #[error("resource '{type_name}' not found at address")]
    NotFound { type_name: String },

    #[error("resource '{type_name}' cannot be copied")]
    CannotCopy { type_name: String },

    #[error("resource '{type_name}' cannot be dropped")]
    CannotDrop { type_name: String },
}

/// Generate NeoVM helper code for resource tracking
///
/// This would be injected into the compiled contract to track resources.
pub fn generate_resource_tracker_code() -> Vec<u8> {
    // Placeholder - would generate NeoVM bytecode for:
    // 1. Storage-based resource existence tracking
    // 2. Runtime checks before move_to/move_from
    // 3. Error handling for resource violations
    Vec::new()
}

/// Map Move signer type to Neo CheckWitness
///
/// In Move, `signer` represents the transaction sender.
/// In Neo, this maps to CheckWitness verification.
pub fn signer_to_checkwitness() -> &'static str {
    "System.Runtime.CheckWitness"
}

/// Prefix byte for resource storage keys
const RESOURCE_PREFIX: u8 = b'R';

/// Separator byte between address and type name in storage keys
const RESOURCE_SEPARATOR: u8 = b':';

/// Map Move global storage to Neo contract storage
///
/// Move: `borrow_global<T>`(address)
/// Neo: Storage.Get(prefix + address + type_hash)
pub fn global_storage_key(address: &[u8], type_name: &str) -> Vec<u8> {
    let mut key = Vec::with_capacity(address.len() + type_name.len() + 2);
    key.push(RESOURCE_PREFIX);
    key.extend_from_slice(address);
    key.push(RESOURCE_SEPARATOR);
    key.extend_from_slice(type_name.as_bytes());
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_lifecycle() {
        let mut tracker = ResourceTracker::new();
        let addr = b"test_address";
        let type_name = "0x1::Coin::Coin";

        // Initially doesn't exist
        assert!(!tracker.exists(addr, type_name));

        // Move to address
        tracker.move_to(addr, type_name).unwrap();
        assert!(tracker.exists(addr, type_name));

        // Can't move to same address again
        assert!(tracker.move_to(addr, type_name).is_err());

        // Move from address
        tracker.move_from(addr, type_name).unwrap();
        assert!(!tracker.exists(addr, type_name));

        // Can't move from non-existent
        assert!(tracker.move_from(addr, type_name).is_err());
    }
}
