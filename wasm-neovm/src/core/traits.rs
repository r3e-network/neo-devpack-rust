// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Core traits for the WASM to NeoVM translator
//!
//! This module consolidates related traits and provides a unified
//! interface for common operations.

use anyhow::Result;

use crate::types::{BytecodeOffset, ContractName, WasmValueType};

/// Trait for types that can be serialized to NeoVM bytecode
///
/// This trait replaces multiple similar serialization traits
/// that were previously scattered throughout the codebase.
pub trait ToBytecode {
    /// Serialize this value to NeoVM bytecode
    fn to_bytecode(&self) -> Vec<u8>;

    /// Get the size in bytes of the serialized form
    fn bytecode_size(&self) -> usize {
        self.to_bytecode().len()
    }
}

/// Trait for types that represent a translatable entity
///
/// This consolidates the various translation-related traits.
pub trait Translatable {
    /// The output type of translation
    type Output;

    /// Translate this entity
    fn translate(&self) -> Result<Self::Output>;

    /// Check if this entity can be translated
    fn can_translate(&self) -> bool {
        true
    }
}

/// Trait for bytecode emitters
///
/// Unified interface for emitting NeoVM opcodes and operands.
pub trait BytecodeEmitter {
    /// Emit a single byte
    fn emit_byte(&mut self, byte: u8);

    /// Emit multiple bytes
    fn emit_bytes(&mut self, bytes: &[u8]);

    /// Emit a 16-bit little-endian value
    fn emit_u16_le(&mut self, value: u16) {
        self.emit_bytes(&value.to_le_bytes());
    }

    /// Emit a 32-bit little-endian value
    fn emit_u32_le(&mut self, value: u32) {
        self.emit_bytes(&value.to_le_bytes());
    }

    /// Get the current offset in the bytecode
    fn current_offset(&self) -> BytecodeOffset;

    /// Get the total size of emitted bytecode
    fn emitted_size(&self) -> usize;
}

impl BytecodeEmitter for Vec<u8> {
    fn emit_byte(&mut self, byte: u8) {
        self.push(byte);
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn current_offset(&self) -> BytecodeOffset {
        BytecodeOffset::new(self.len())
    }

    fn emitted_size(&self) -> usize {
        self.len()
    }
}

/// Trait for named entities
///
/// Provides a consistent way to get names from various types.
pub trait Named {
    /// Get the name of this entity
    fn name(&self) -> &str;

    /// Get the fully qualified name if applicable
    fn fully_qualified_name(&self) -> String {
        self.name().to_string()
    }
}

/// Trait for entities with a contract context
///
/// Unified interface for working with contract-scoped entities.
pub trait ContractScoped {
    /// Get the contract name
    fn contract_name(&self) -> &ContractName;

    /// Check if this entity belongs to a specific contract
    fn belongs_to(&self, contract: &ContractName) -> bool {
        self.contract_name() == contract
    }
}

/// Trait for typed entities
///
/// Provides consistent type information access.
pub trait Typed {
    /// Get the value type
    fn value_type(&self) -> WasmValueType;

    /// Check if this is an integer type
    fn is_integer(&self) -> bool {
        self.value_type().is_integer()
    }

    /// Check if this is a reference type
    fn is_reference(&self) -> bool {
        self.value_type().is_reference()
    }
}

/// Trait for validating entities
///
/// Consolidates validation logic across types.
pub trait Validatable {
    /// The error type for validation failures
    type Error;

    /// Validate this entity
    fn validate(&self) -> Result<(), Self::Error>;

    /// Check if this entity is valid without returning details
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Trait for entities that can be optimized
///
/// Unified optimization interface.
pub trait Optimizable {
    /// Optimize this entity, returning true if changes were made
    fn optimize(&mut self) -> bool;

    /// Check if this entity can be optimized
    fn can_optimize(&self) -> bool {
        true
    }
}

/// Trait for size-limited entities
///
/// Provides consistent size limit checking.
pub trait SizeLimited {
    /// Get the current size
    fn current_size(&self) -> usize;

    /// Get the maximum allowed size
    fn max_size(&self) -> usize;

    /// Check if the current size is within limits
    fn is_within_limits(&self) -> bool {
        self.current_size() <= self.max_size()
    }

    /// Get remaining capacity
    fn remaining_capacity(&self) -> usize {
        self.max_size().saturating_sub(self.current_size())
    }
}

/// Trait for resolvable references
///
/// Unified interface for resolving symbolic references.
pub trait Resolvable {
    /// The target type after resolution
    type Target;

    /// Resolve the reference
    fn resolve(&self) -> Option<&Self::Target>;

    /// Check if this reference is resolved
    fn is_resolved(&self) -> bool {
        self.resolve().is_some()
    }
}

/// Trait for mutable resolvable references
pub trait ResolvableMut: Resolvable {
    /// Resolve the reference mutably
    fn resolve_mut(&mut self) -> Option<&mut Self::Target>;
}

/// Trait for indexable collections
///
/// Provides safe indexed access with bounds checking.
pub trait Indexable<Index> {
    /// The element type
    type Element;

    /// Get an element by index
    fn get_at(&self, index: Index) -> Option<&Self::Element>;

    /// Get an element mutably by index
    fn get_at_mut(&mut self, index: Index) -> Option<&mut Self::Element>;

    /// Check if an index is valid
    fn is_valid_index(&self, index: Index) -> bool;
}

/// Trait for countable collections
///
/// Provides consistent count/size access.
pub trait Countable {
    /// Get the count of items
    fn count(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

/// Extension trait for BytecodeEmitter providing convenience methods
pub trait BytecodeEmitterExt: BytecodeEmitter {
    /// Emit a placeholder for a value that will be patched later
    fn emit_placeholder(&mut self, size: usize) -> BytecodeOffset {
        let offset = self.current_offset();
        for _ in 0..size {
            self.emit_byte(0);
        }
        offset
    }

    /// Patch a previously emitted value
    fn patch_u16_le(&mut self, offset: BytecodeOffset, value: u16) {
        let bytes = value.to_le_bytes();
        if let Some(slice) = self.as_mut_slice() {
            let start = offset.value();
            slice[start] = bytes[0];
            slice[start + 1] = bytes[1];
        }
    }

    /// Get a mutable slice of the bytecode if available
    fn as_mut_slice(&mut self) -> Option<&mut [u8]>;
}

impl BytecodeEmitterExt for Vec<u8> {
    fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        Some(self.as_mut_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_emitter_vec() {
        let mut bytecode = Vec::new();
        bytecode.emit_byte(0x01);
        bytecode.emit_bytes(&[0x02, 0x03]);
        bytecode.emit_u16_le(0x1234);

        assert_eq!(bytecode, vec![0x01, 0x02, 0x03, 0x34, 0x12]);
        assert_eq!(bytecode.current_offset().value(), 5);
    }

    #[test]
    fn test_countable() {
        struct TestCollection(Vec<i32>);
        impl Countable for TestCollection {
            fn count(&self) -> usize {
                self.0.len()
            }
        }

        let empty = TestCollection(vec![]);
        assert!(empty.is_empty());

        let items = TestCollection(vec![1, 2, 3]);
        assert_eq!(items.count(), 3);
        assert!(!items.is_empty());
    }

    #[test]
    fn test_size_limited() {
        struct TestBuffer {
            data: Vec<u8>,
            max: usize,
        }
        impl SizeLimited for TestBuffer {
            fn current_size(&self) -> usize {
                self.data.len()
            }
            fn max_size(&self) -> usize {
                self.max
            }
        }

        let buffer = TestBuffer {
            data: vec![1, 2, 3],
            max: 10,
        };
        assert!(buffer.is_within_limits());
        assert_eq!(buffer.remaining_capacity(), 7);
    }
}
