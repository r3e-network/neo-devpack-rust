// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use std::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 Map type
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "K: Serialize + Eq, V: Serialize",
        deserialize = "K: Deserialize<'de> + Eq, V: Deserialize<'de>"
    ))
)]
pub struct NeoMap<K, V> {
    data: Vec<(K, V)>,
}

impl<K, V> NeoMap<K, V> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: PartialEq,
    {
        for (k, v) in &mut self.data {
            if *k == key {
                return Some(core::mem::replace(v, value));
            }
        }
        self.data.push((key, value));
        None
    }

    /// Gets a reference to the value associated with the given key.
    ///
    /// # Performance
    /// This operation is O(n) as it performs a linear search.
    /// Consider using a HashMap for O(1) lookups if performance is critical.
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        self.data.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Gets a mutable reference to the value associated with the given key.
    ///
    /// # Performance
    /// This operation is O(n) as it performs a linear search.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: PartialEq,
    {
        self.data.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Removes the key-value pair associated with the given key.
    ///
    /// # Performance
    /// This operation is O(n) due to the element removal.
    ///
    /// # Order stability
    /// Uses `Vec::remove` (shift) so iteration order of remaining entries is
    /// preserved (D7: `swap_remove` reordered entries, diverging from on-chain
    /// NeoVM Map semantics and breaking deterministic serialization). Note that
    /// on-chain NeoVM `Map` is sorted by key; this `NeoMap` is a linear map
    /// (insertion-stable), not the on-chain sorted map.
    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: PartialEq,
    {
        self.data
            .iter()
            .position(|(k, _)| k == key)
            .map(|i| self.data.remove(i).1)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter().map(|(k, v)| (k, v))
    }

    /// Returns true if the map contains the given key.
    pub fn contains_key(&self, key: &K) -> bool
    where
        K: PartialEq,
    {
        self.data.iter().any(|(k, _)| k == key)
    }

    /// Returns an iterator over the keys of the map.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.data.iter().map(|(k, _)| k)
    }

    /// Returns an iterator over the values of the map.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().map(|(_, v)| v)
    }

    /// Strict removal: only removes if both `key` AND `expected_value`
    /// match. Returns `Ok(())` on success and `Err(Mismatch)` on value
    /// mismatch or missing key. Mirrors the C# NeoVM `MAPREMOVE`
    /// semantics (which FAULTs if value doesn't match), letting
    /// contracts handle the bound explicitly.
    ///
    /// `Mismatch::Found` distinguishes "missing key" (the standard
    /// `remove` would have returned `None`) from "wrong value" (a
    /// potential optimistic-concurrency conflict).
    pub fn remove_strict(&mut self, key: &K, expected_value: &V) -> Result<(), RemoveStrictError>
    where
        K: PartialEq,
        V: PartialEq,
    {
        match self.data.iter().position(|(k, _)| k == key) {
            Some(i) if &self.data[i].1 == expected_value => {
                self.data.remove(i);
                Ok(())
            }
            Some(_) => Err(RemoveStrictError::ValueMismatch),
            None => Err(RemoveStrictError::Missing),
        }
    }
}

/// Error returned by [`NeoMap::remove_strict`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveStrictError {
    /// The key was not present in the map.
    Missing,
    /// The key was present but the value did not match `expected_value`.
    ValueMismatch,
}

impl core::fmt::Display for RemoveStrictError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RemoveStrictError::Missing => write!(f, "key not present in map"),
            RemoveStrictError::ValueMismatch => {
                write!(f, "value at key did not match expected value")
            }
        }
    }
}

impl std::error::Error for RemoveStrictError {}

impl<K, V> Default for NeoMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> IntoIterator for NeoMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a NeoMap<K, V> {
    type Item = &'a (K, V);
    type IntoIter = std::slice::Iter<'a, (K, V)>;
    fn into_iter(self) -> std::slice::Iter<'a, (K, V)> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_strict_succeeds_on_match() {
        let mut m: NeoMap<u32, u32> = NeoMap::new();
        m.insert(1, 100);
        m.insert(2, 200);
        assert!(m.remove_strict(&1, &100).is_ok());
        assert!(!m.contains_key(&1));
        assert!(m.contains_key(&2));
    }

    #[test]
    fn remove_strict_returns_missing_on_absent_key() {
        let mut m: NeoMap<u32, u32> = NeoMap::new();
        m.insert(1, 100);
        assert_eq!(m.remove_strict(&99, &100), Err(RemoveStrictError::Missing));
    }

    #[test]
    fn remove_strict_returns_value_mismatch_on_wrong_value() {
        let mut m: NeoMap<u32, u32> = NeoMap::new();
        m.insert(1, 100);
        assert_eq!(
            m.remove_strict(&1, &999),
            Err(RemoveStrictError::ValueMismatch)
        );
        // Key still present.
        assert!(m.contains_key(&1));
        assert_eq!(m.get(&1), Some(&100));
    }
}
