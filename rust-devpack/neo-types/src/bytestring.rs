// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use std::fmt;
use std::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 ByteString type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoByteString {
    data: Vec<u8>,
}

impl NeoByteString {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            data: slice.to_vec(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn push(&mut self, byte: u8) {
        self.data.push(byte);
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }
}

impl fmt::Display for NeoByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.data {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl From<Vec<u8>> for NeoByteString {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl From<&[u8]> for NeoByteString {
    fn from(slice: &[u8]) -> Self {
        Self::from_slice(slice)
    }
}

impl AsRef<[u8]> for NeoByteString {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Extend<u8> for NeoByteString {
    fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        self.data.extend(iter);
    }
}

impl FromIterator<u8> for NeoByteString {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        Self {
            data: Vec::from_iter(iter),
        }
    }
}
