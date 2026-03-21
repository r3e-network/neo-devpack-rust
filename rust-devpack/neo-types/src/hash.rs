// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use crate::error::{NeoError, NeoResult};
use crate::NeoByteString;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A 20-byte script hash used to identify accounts and contracts on Neo N3.
///
/// Hash160 is the standard identifier for Neo addresses and contract script hashes.
/// It wraps exactly 20 bytes and provides validated construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Hash160([u8; 20]);

/// A 32-byte hash used for transaction and block identifiers on Neo N3.
///
/// Hash256 wraps exactly 32 bytes and provides validated construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Hash256([u8; 32]);

impl Hash160 {
    /// The fixed byte length of a Hash160.
    pub const LENGTH: usize = 20;

    /// A zero-valued Hash160.
    pub const ZERO: Self = Self([0u8; 20]);

    /// Creates a Hash160 from a 20-byte array.
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Attempts to create a Hash160 from a byte slice.
    ///
    /// Returns `Err(NeoError::InvalidArgument)` if the slice length is not exactly 20.
    pub fn try_from_slice(slice: &[u8]) -> NeoResult<Self> {
        let bytes: [u8; 20] = slice.try_into().map_err(|_| {
            NeoError::Custom(format!(
                "Hash160 requires exactly 20 bytes, got {}",
                slice.len()
            ))
        })?;
        Ok(Self(bytes))
    }

    /// Returns the underlying 20 bytes as a slice.
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Returns the underlying 20 bytes as a generic slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns `true` if all bytes are zero.
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 20]
    }

    /// Converts this Hash160 into a `NeoByteString`.
    pub fn to_byte_string(&self) -> NeoByteString {
        NeoByteString::from_slice(&self.0)
    }
}

impl Hash256 {
    /// The fixed byte length of a Hash256.
    pub const LENGTH: usize = 32;

    /// A zero-valued Hash256.
    pub const ZERO: Self = Self([0u8; 32]);

    /// Creates a Hash256 from a 32-byte array.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Attempts to create a Hash256 from a byte slice.
    ///
    /// Returns `Err(NeoError::InvalidArgument)` if the slice length is not exactly 32.
    pub fn try_from_slice(slice: &[u8]) -> NeoResult<Self> {
        let bytes: [u8; 32] = slice.try_into().map_err(|_| {
            NeoError::Custom(format!(
                "Hash256 requires exactly 32 bytes, got {}",
                slice.len()
            ))
        })?;
        Ok(Self(bytes))
    }

    /// Returns the underlying 32 bytes as a slice.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns the underlying 32 bytes as a generic slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns `true` if all bytes are zero.
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }

    /// Converts this Hash256 into a `NeoByteString`.
    pub fn to_byte_string(&self) -> NeoByteString {
        NeoByteString::from_slice(&self.0)
    }
}

impl TryFrom<&[u8]> for Hash160 {
    type Error = NeoError;
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_slice(slice)
    }
}

impl TryFrom<Vec<u8>> for Hash160 {
    type Error = NeoError;
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_slice(&vec)
    }
}

impl TryFrom<NeoByteString> for Hash160 {
    type Error = NeoError;
    fn try_from(bs: NeoByteString) -> Result<Self, Self::Error> {
        Self::try_from_slice(bs.as_slice())
    }
}

impl From<Hash160> for NeoByteString {
    fn from(h: Hash160) -> Self {
        h.to_byte_string()
    }
}

impl TryFrom<&[u8]> for Hash256 {
    type Error = NeoError;
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_slice(slice)
    }
}

impl TryFrom<Vec<u8>> for Hash256 {
    type Error = NeoError;
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_slice(&vec)
    }
}

impl TryFrom<NeoByteString> for Hash256 {
    type Error = NeoError;
    fn try_from(bs: NeoByteString) -> Result<Self, Self::Error> {
        Self::try_from_slice(bs.as_slice())
    }
}

impl From<Hash256> for NeoByteString {
    fn from(h: Hash256) -> Self {
        h.to_byte_string()
    }
}

impl AsRef<[u8]> for Hash160 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Hash256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash160_from_valid_bytes() {
        let bytes = [0xABu8; 20];
        let h = Hash160::from_bytes(bytes);
        assert_eq!(h.as_bytes(), &[0xAB; 20]);
    }

    #[test]
    fn hash160_try_from_wrong_length_fails() {
        assert!(Hash160::try_from_slice(&[0u8; 19]).is_err());
        assert!(Hash160::try_from_slice(&[0u8; 21]).is_err());
        assert!(Hash160::try_from_slice(&[0u8; 0]).is_err());
    }

    #[test]
    fn hash160_try_from_correct_length_succeeds() {
        let h = Hash160::try_from_slice(&[0xFFu8; 20]).unwrap();
        assert_eq!(h.as_bytes(), &[0xFF; 20]);
    }

    #[test]
    fn hash160_zero() {
        assert!(Hash160::ZERO.is_zero());
        assert!(!Hash160::from_bytes([1; 20]).is_zero());
    }

    #[test]
    fn hash256_from_valid_bytes() {
        let bytes = [0xCDu8; 32];
        let h = Hash256::from_bytes(bytes);
        assert_eq!(h.as_bytes(), &[0xCD; 32]);
    }

    #[test]
    fn hash256_try_from_wrong_length_fails() {
        assert!(Hash256::try_from_slice(&[0u8; 31]).is_err());
        assert!(Hash256::try_from_slice(&[0u8; 33]).is_err());
    }

    #[test]
    fn hash256_try_from_correct_length_succeeds() {
        let h = Hash256::try_from_slice(&[0xEEu8; 32]).unwrap();
        assert_eq!(h.as_bytes(), &[0xEE; 32]);
    }

    #[test]
    fn hash256_zero() {
        assert!(Hash256::ZERO.is_zero());
        assert!(!Hash256::from_bytes([1; 32]).is_zero());
    }

    #[test]
    fn hash160_roundtrip_bytestring() {
        let h = Hash160::from_bytes([0x42; 20]);
        let bs: NeoByteString = h.clone().into();
        let h2 = Hash160::try_from(bs).unwrap();
        assert_eq!(h, h2);
    }

    #[test]
    fn hash256_roundtrip_bytestring() {
        let h = Hash256::from_bytes([0x42; 32]);
        let bs: NeoByteString = h.clone().into();
        let h2 = Hash256::try_from(bs).unwrap();
        assert_eq!(h, h2);
    }
}
