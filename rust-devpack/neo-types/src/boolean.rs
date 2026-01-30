// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use std::ops::{BitAnd, BitOr, BitXor, Not};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 Boolean type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct NeoBoolean(pub bool);

impl NeoBoolean {
    pub const TRUE: Self = Self(true);
    pub const FALSE: Self = Self(false);

    pub fn new(value: bool) -> Self {
        Self(value)
    }

    pub fn as_bool(self) -> bool {
        self.0
    }
}

impl BitAnd for NeoBoolean {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for NeoBoolean {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for NeoBoolean {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Not for NeoBoolean {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Default for NeoBoolean {
    fn default() -> Self {
        Self::FALSE
    }
}
