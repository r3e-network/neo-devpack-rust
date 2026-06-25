// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Stack items modelled for the opcode subset we execute.

use num_bigint::BigInt;
use num_traits::Zero;
use std::cell::RefCell;
use std::rc::Rc;

/// A shared mutable byte buffer. NeoVM `Buffer`/`ByteString` are reference
/// types: a `LDSFLD` followed by `SETITEM` mutates the shared buffer so the
/// change is visible to later loads. We model that with `Rc<RefCell<..>>`.
pub type SharedBuffer = Rc<RefCell<Vec<u8>>>;

fn shared_buf(values: Vec<u8>) -> SharedBuffer {
    Rc::new(RefCell::new(values))
}

/// A NeoVM stack item for the executor's supported subset.
#[derive(Debug, Clone)]
pub enum NeoItem {
    /// A 256-bit signed integer (NeoVM's Integer type).
    Integer(BigInt),
    /// A byte buffer (NeoVM Buffer / ByteString — treated interchangeably for
    /// the subset of ops we execute: SUBSTR/CAT/CONVERT/SETITEM/PICKITEM).
    Buffer(SharedBuffer),
    /// A NeoVM array (used for the memory backing store and PACK/unPACK).
    Array(Vec<NeoItem>),
    /// A struct (same memory layout as Array for our purposes).
    Struct(Vec<NeoItem>),
    /// Boolean.
    Boolean(bool),
    /// Null.
    Null,
}

impl NeoItem {
    pub fn int<T>(v: T) -> Self
    where
        BigInt: From<T>,
    {
        NeoItem::Integer(BigInt::from(v))
    }

    pub fn buffer(values: Vec<u8>) -> Self {
        NeoItem::Buffer(shared_buf(values))
    }

    pub fn as_int(&self) -> Option<&BigInt> {
        match self {
            NeoItem::Integer(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_int_owned(self) -> Option<BigInt> {
        match self {
            NeoItem::Integer(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_buffer(&self) -> Option<Vec<u8>> {
        match self {
            NeoItem::Buffer(b) => Some(b.borrow().clone()),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<NeoItem>> {
        match self {
            NeoItem::Array(a) | NeoItem::Struct(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[NeoItem]> {
        match self {
            NeoItem::Array(a) | NeoItem::Struct(a) => Some(a),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            NeoItem::Integer(i) => !i.is_zero(),
            NeoItem::Boolean(b) => *b,
            NeoItem::Null => false,
            NeoItem::Buffer(b) => !b.borrow().is_empty(),
            NeoItem::Array(a) | NeoItem::Struct(a) => !a.is_empty(),
        }
    }

    /// Convert to an integer following NeoVM CONVERT semantics.
    pub fn convert_to_int(&self) -> Option<BigInt> {
        match self {
            NeoItem::Integer(i) => Some(i.clone()),
            NeoItem::Boolean(b) => Some(BigInt::from(*b as i32)),
            NeoItem::Buffer(b) => {
                let b = b.borrow();
                if b.is_empty() {
                    Some(BigInt::from(0))
                } else {
                    Some(BigInt::from_signed_bytes_le(&b))
                }
            }
            NeoItem::Null => Some(BigInt::from(0)),
            NeoItem::Array(a) | NeoItem::Struct(a) => Some(BigInt::from(a.len())),
        }
    }

    /// Size of the item (NeoVM SIZE semantics).
    pub fn size(&self) -> usize {
        match self {
            NeoItem::Buffer(b) => b.borrow().len(),
            NeoItem::Array(a) | NeoItem::Struct(a) => a.len(),
            NeoItem::Integer(i) => i.to_bytes_le().1.len(),
            _ => 0,
        }
    }
}

/// Convert a signed integer into a fixed-width little-endian two's-complement
/// buffer of `bytes` length (matching how NeoVM represents integer→buffer).
pub fn int_to_fixed_bytes(value: &BigInt, bytes: usize) -> Vec<u8> {
    let (sign, mag_le) = value.to_bytes_le();
    let mut out = vec![0u8; bytes];
    let n = mag_le.len().min(bytes);
    out[..n].copy_from_slice(&mag_le[..n]);
    if sign == num_bigint::Sign::Minus {
        // two's complement: invert and add 1
        for b in out.iter_mut() {
            *b = !*b;
        }
        let mut carry = 1u8;
        for b in out.iter_mut() {
            let (v, c) = b.overflowing_add(carry);
            *b = v;
            carry = if c { 1 } else { 0 };
        }
    }
    out
}

/// Re-interpret a `&[u8]` buffer as a two's-complement little-endian integer.
pub fn bytes_to_int(buf: &[u8]) -> BigInt {
    if buf.is_empty() {
        return BigInt::from(0);
    }
    BigInt::from_signed_bytes_le(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_roundtrip_i32_max() {
        let v = BigInt::from(0x7FFFFFFFi64);
        let bytes = int_to_fixed_bytes(&v, 4);
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0x7F]);
        assert_eq!(bytes_to_int(&bytes), v);
    }

    #[test]
    fn int_roundtrip_negative() {
        let v = BigInt::from(-1);
        let bytes = int_to_fixed_bytes(&v, 4);
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(bytes_to_int(&bytes), v);
    }

    #[test]
    fn int_roundtrip_i32_min() {
        let v = BigInt::from(i32::MIN as i64);
        let bytes = int_to_fixed_bytes(&v, 4);
        assert_eq!(bytes, [0x00, 0x00, 0x00, 0x80]);
        assert_eq!(bytes_to_int(&bytes), v);
    }
}
