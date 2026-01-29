#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 Storage Context type
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoStorageContext {
    id: u32,
    read_only: bool,
}

impl NeoStorageContext {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            read_only: false,
        }
    }

    pub fn with_read_only(id: u32, read_only: bool) -> Self {
        Self { id, read_only }
    }

    pub fn read_only(id: u32) -> Self {
        Self {
            id,
            read_only: true,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn as_read_only(&self) -> Self {
        Self {
            id: self.id,
            read_only: true,
        }
    }
}
