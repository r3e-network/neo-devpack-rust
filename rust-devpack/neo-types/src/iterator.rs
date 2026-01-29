use std::vec::Vec;

/// Neo N3 Iterator type
#[derive(Debug, Clone)]
pub struct NeoIterator<T> {
    data: Vec<T>,
}

impl<T> NeoIterator<T> {
    /// Creates a new iterator from a vector.
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    /// Returns the next element and advances the iterator.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.data.remove(0))
        }
    }

    /// Returns true if there are more elements to iterate.
    pub fn has_next(&self) -> bool {
        !self.data.is_empty()
    }

    /// Returns the number of remaining elements.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if no more elements are available.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
