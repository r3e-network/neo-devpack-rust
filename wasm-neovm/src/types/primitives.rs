// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Primitive type wrappers for WASM/NeoVM translation
//!
//! These types provide semantic meaning to primitive values used
//! during the translation process.

use std::fmt;

/// A WASM value type (i32, i64, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WasmValueType {
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// 32-bit floating point (not fully supported)
    F32,
    /// 64-bit floating point (not fully supported)
    F64,
    /// Function reference
    FuncRef,
    /// External reference
    ExternRef,
}

impl WasmValueType {
    /// Check if this type is an integer type
    pub const fn is_integer(&self) -> bool {
        matches!(self, Self::I32 | Self::I64)
    }

    /// Check if this type is a floating point type
    pub const fn is_float(&self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }

    /// Check if this type is a reference type
    pub const fn is_reference(&self) -> bool {
        matches!(self, Self::FuncRef | Self::ExternRef)
    }

    /// Get the size in bits
    pub const fn bit_width(&self) -> u32 {
        match self {
            Self::I32 | Self::F32 => 32,
            Self::I64 | Self::F64 => 64,
            Self::FuncRef | Self::ExternRef => 32, // Pointers are 32-bit in WASM
        }
    }

    /// Get the size in bytes
    pub const fn byte_size(&self) -> u32 {
        self.bit_width() / 8
    }
}

impl fmt::Display for WasmValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::FuncRef => write!(f, "funcref"),
            Self::ExternRef => write!(f, "externref"),
        }
    }
}

impl From<wasmparser::ValType> for WasmValueType {
    fn from(ty: wasmparser::ValType) -> Self {
        use wasmparser::ValType;
        match ty {
            ValType::I32 => Self::I32,
            ValType::I64 => Self::I64,
            ValType::F32 => Self::F32,
            ValType::F64 => Self::F64,
            ValType::Ref(ref_type) => {
                if ref_type.is_func_ref() {
                    Self::FuncRef
                } else {
                    Self::ExternRef
                }
            }
            _ => Self::ExternRef, // Handle any new reference types
        }
    }
}

/// A NeoVM stack item type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeoStackType {
    /// Boolean value
    Boolean,
    /// Integer (BigInteger)
    Integer,
    /// Byte array
    ByteArray,
    /// UTF-8 string
    String,
    /// Array
    Array,
    /// Map
    Map,
    /// Contract reference
    Contract,
    /// Any type (untyped)
    Any,
}

impl fmt::Display for NeoStackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "Boolean"),
            Self::Integer => write!(f, "Integer"),
            Self::ByteArray => write!(f, "ByteArray"),
            Self::String => write!(f, "String"),
            Self::Array => write!(f, "Array"),
            Self::Map => write!(f, "Map"),
            Self::Contract => write!(f, "Contract"),
            Self::Any => write!(f, "Any"),
        }
    }
}

/// Alignment for memory operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Alignment(u32);

impl Alignment {
    /// Create a new alignment (must be a power of 2)
    ///
    /// # Panics
    /// Panics if alignment is not a power of 2
    pub fn new(align: u32) -> Self {
        assert!(align.is_power_of_two(), "Alignment must be a power of 2");
        Self(align)
    }

    /// Try to create a new alignment, returning `None` when the value is not a power of 2.
    pub const fn try_new(align: u32) -> Option<Self> {
        if align.is_power_of_two() {
            Some(Self(align))
        } else {
            None
        }
    }

    /// Create alignment without validation (use with caution)
    pub const fn new_unchecked(align: u32) -> Self {
        Self(align)
    }

    /// Get the alignment value
    pub const fn value(&self) -> u32 {
        self.0
    }

    /// Check if an offset is aligned
    pub const fn is_aligned(&self, offset: u32) -> bool {
        offset % self.0 == 0
    }

    /// Align an offset up to this alignment
    pub const fn align_up(&self, offset: u32) -> u32 {
        (offset + self.0 - 1) & !(self.0 - 1)
    }
}

impl fmt::Display for Alignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} bytes", self.0)
    }
}

impl Default for Alignment {
    fn default() -> Self {
        Self(1)
    }
}

/// Memory access size for load/store operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessSize {
    /// 8-bit access
    Byte,
    /// 16-bit access
    HalfWord,
    /// 32-bit access
    Word,
    /// 64-bit access
    DoubleWord,
}

impl AccessSize {
    /// Get the size in bytes
    pub const fn bytes(&self) -> u32 {
        match self {
            Self::Byte => 1,
            Self::HalfWord => 2,
            Self::Word => 4,
            Self::DoubleWord => 8,
        }
    }

    /// Get the size in bits
    pub const fn bits(&self) -> u32 {
        self.bytes() * 8
    }
}

impl fmt::Display for AccessSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Byte => write!(f, "byte"),
            Self::HalfWord => write!(f, "halfword"),
            Self::Word => write!(f, "word"),
            Self::DoubleWord => write!(f, "doubleword"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_value_type() {
        assert!(WasmValueType::I32.is_integer());
        assert!(WasmValueType::I64.is_integer());
        assert!(!WasmValueType::F32.is_integer());
        assert!(WasmValueType::F32.is_float());
        assert_eq!(WasmValueType::I32.byte_size(), 4);
        assert_eq!(WasmValueType::I64.byte_size(), 8);
    }

    #[test]
    fn test_alignment() {
        let align = Alignment::new(4);
        assert_eq!(align.value(), 4);
        assert!(align.is_aligned(8));
        assert!(!align.is_aligned(7));
        assert_eq!(align.align_up(5), 8);
        assert_eq!(align.align_up(4), 4);
        assert_eq!(Alignment::try_new(8).map(|v| v.value()), Some(8));
        assert!(Alignment::try_new(3).is_none());
        assert!(Alignment::try_new(0).is_none());
    }

    #[test]
    fn test_access_size() {
        assert_eq!(AccessSize::Byte.bytes(), 1);
        assert_eq!(AccessSize::HalfWord.bytes(), 2);
        assert_eq!(AccessSize::Word.bytes(), 4);
        assert_eq!(AccessSize::DoubleWord.bytes(), 8);
    }
}
