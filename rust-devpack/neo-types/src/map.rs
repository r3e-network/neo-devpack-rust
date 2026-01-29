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

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        self.data
            .iter()
            .find_map(|(k, v)| if k == key { Some(v) } else { None })
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: PartialEq,
    {
        self.data
            .iter_mut()
            .find_map(|(k, v)| if k == key { Some(v) } else { None })
    }

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

    /// Returns an iterator over the keys of the map
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.data.iter().map(|(k, _)| k)
    }

    /// Returns an iterator over the values of the map
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().map(|(_, v)| v)
    }
}

impl<K, V> Default for NeoMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
