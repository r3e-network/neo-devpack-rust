use std::vec::Vec;

/// Neo N3 Iterator type
#[derive(Debug, Clone)]
pub struct NeoIterator<T> {
    data: Vec<T>,
    index: usize,
}

impl<T> NeoIterator<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data, index: 0 }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        if self.index < self.data.len() {
            let item = self.data.remove(self.index);
            Some(item)
        } else {
            None
        }
    }

    pub fn has_next(&self) -> bool {
        self.index < self.data.len()
    }
}
