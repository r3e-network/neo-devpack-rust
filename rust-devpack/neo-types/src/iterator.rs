// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use std::vec::Vec;

/// Neo N3 Iterator type
///
/// This iterator efficiently traverses elements using an internal cursor,
/// avoiding O(n) overhead of Vec::remove(0) that would occur with a naive
/// implementation. Implements `std::iter::Iterator` for use with `for` loops
/// and iterator combinators.
#[derive(Debug, Clone)]
pub struct NeoIterator<T> {
    data: Vec<T>,
    cursor: usize,
}

impl<T> NeoIterator<T> {
    /// Creates a new iterator from a vector.
    pub fn new(data: Vec<T>) -> Self {
        Self { data, cursor: 0 }
    }

    /// Returns true if there are more elements to iterate.
    pub fn has_next(&self) -> bool {
        self.cursor < self.data.len()
    }

    /// Returns the number of remaining elements.
    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    /// Returns true if no more elements are available.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resets the iterator to the beginning.
    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    /// Returns the total number of elements (including already consumed).
    pub fn total_len(&self) -> usize {
        self.data.len()
    }
}

impl<T: Clone> Iterator for NeoIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.cursor >= self.data.len() {
            None
        } else {
            let item = self.data[self.cursor].clone();
            self.cursor += 1;
            Some(item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len();
        (remaining, Some(remaining))
    }
}

impl<T: Clone> ExactSizeIterator for NeoIterator<T> {}
