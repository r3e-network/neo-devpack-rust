// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use std::ops::Index;
use std::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 Array type
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))
)]
pub struct NeoArray<T> {
    data: Vec<T>,
}

impl<T> NeoArray<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }
}

impl<T> Default for NeoArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Vec<T>> for NeoArray<T> {
    fn from(data: Vec<T>) -> Self {
        Self { data }
    }
}

impl<T> FromIterator<T> for NeoArray<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            data: Vec::from_iter(iter),
        }
    }
}

impl<T> Extend<T> for NeoArray<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.data.extend(iter);
    }
}

impl<T> Index<usize> for NeoArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.data[index]
    }
}

impl<T> IntoIterator for NeoArray<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a NeoArray<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<T: PartialEq> PartialEq<Vec<T>> for NeoArray<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.data == *other
    }
}

impl<T: PartialEq> PartialEq<NeoArray<T>> for Vec<T> {
    fn eq(&self, other: &NeoArray<T>) -> bool {
        *self == other.data
    }
}

/// An error returned when a `NeoArray` would exceed `NeoArray::MAX_SIZE` items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayFullError {
    /// The current length of the array when the push was attempted.
    pub current_len: usize,
}

impl core::fmt::Display for ArrayFullError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "NeoArray is full (current_len = {}; MAX_SIZE = {})",
            self.current_len,
            crate::MAX_STACK_SIZE
        )
    }
}

impl std::error::Error for ArrayFullError {}

impl<T> NeoArray<T> {
    /// Maximum number of items in a `NeoArray` on-chain. Mirrors the single
    /// source of truth `MAX_STACK_SIZE` (the C# NeoVM `Limits.MaxStackSize`,
    /// 1024); exceeding it at runtime raises `FAULT`. Spelled as an
    /// associated const for parity with `NeoByteString::MAX_SIZE`.
    pub const MAX_SIZE: usize = crate::MAX_STACK_SIZE;

    /// Try to push an item, returning `Err(ArrayFullError)` if the array
    /// is already at `MAX_SIZE` (1024) items. Use this instead of `push`
    /// when the array size is data-dependent and you need to handle the
    /// bound explicitly (e.g. paginated reads).
    pub fn try_push(&mut self, item: T) -> Result<(), ArrayFullError> {
        if self.data.len() >= Self::MAX_SIZE {
            return Err(ArrayFullError {
                current_len: self.data.len(),
            });
        }
        self.data.push(item);
        Ok(())
    }

    /// Returns the remaining capacity before the on-chain `MAX_SIZE`
    /// limit is reached.
    pub fn remaining_capacity(&self) -> usize {
        Self::MAX_SIZE.saturating_sub(self.data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_SIZE: usize = NeoArray::<u32>::MAX_SIZE;

    #[test]
    fn max_size_tracks_single_source() {
        assert_eq!(NeoArray::<u32>::MAX_SIZE, crate::MAX_STACK_SIZE);
    }

    #[test]
    fn try_push_within_limit() {
        let mut arr: NeoArray<u32> = NeoArray::new();
        for i in 0..100 {
            arr.try_push(i).expect("well under MAX_SIZE");
        }
        assert_eq!(arr.len(), 100);
        assert_eq!(arr.remaining_capacity(), MAX_SIZE - 100);
    }

    #[test]
    fn try_push_at_limit_returns_error() {
        let mut arr: NeoArray<u32> = NeoArray::new();
        for i in 0..MAX_SIZE {
            arr.try_push(i as u32).expect("fits within MAX_SIZE");
        }
        assert_eq!(arr.len(), MAX_SIZE);
        assert_eq!(arr.remaining_capacity(), 0);
        // 1025th push must fail.
        let err = arr.try_push(MAX_SIZE as u32).unwrap_err();
        assert_eq!(err.current_len, MAX_SIZE);
        assert_eq!(
            err.to_string(),
            "NeoArray is full (current_len = 1024; MAX_SIZE = 1024)"
        );
    }
}
