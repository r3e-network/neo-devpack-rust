// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

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
