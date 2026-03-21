// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};

use num_bigint::BigInt;
use num_traits::{One, ToPrimitive, Zero};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 Integer type (arbitrary precision)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoInteger(BigInt);

impl NeoInteger {
    pub fn new<T: Into<BigInt>>(value: T) -> Self {
        Self(value.into())
    }

    pub fn zero() -> Self {
        Self(BigInt::zero())
    }

    pub fn one() -> Self {
        Self(BigInt::one())
    }

    pub fn min_i32() -> Self {
        Self(BigInt::from(i32::MIN))
    }

    pub fn max_i32() -> Self {
        Self(BigInt::from(i32::MAX))
    }

    pub fn as_bigint(&self) -> &BigInt {
        &self.0
    }

    /// Convert to i32, returning None if the value is out of range.
    /// This is the safe alternative to `as_i32()` that doesn't panic.
    pub fn try_as_i32(&self) -> Option<i32> {
        self.0.to_i32()
    }

    /// Convert to u32, returning None if the value is out of range.
    /// This is the safe alternative to `as_u32()` that doesn't panic.
    pub fn try_as_u32(&self) -> Option<u32> {
        self.0.to_u32()
    }

    /// Convert to i64, returning None if the value is out of range.
    pub fn try_as_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }

    /// Convert to u64, returning None if the value is out of range.
    pub fn try_as_u64(&self) -> Option<u64> {
        self.0.to_u64()
    }

    /// Convert to i32, returning `Result` for ergonomic `?` usage.
    pub fn try_into_i32(&self) -> crate::NeoResult<i32> {
        self.0
            .to_i32()
            .ok_or(crate::NeoError::Overflow)
    }

    /// Convert to u32, returning `Result` for ergonomic `?` usage.
    pub fn try_into_u32(&self) -> crate::NeoResult<u32> {
        self.0
            .to_u32()
            .ok_or(crate::NeoError::Overflow)
    }

    /// Convert to i64, returning `Result` for ergonomic `?` usage.
    pub fn try_into_i64(&self) -> crate::NeoResult<i64> {
        self.0
            .to_i64()
            .ok_or(crate::NeoError::Overflow)
    }

    /// Convert to u64, returning `Result` for ergonomic `?` usage.
    pub fn try_into_u64(&self) -> crate::NeoResult<u64> {
        self.0
            .to_u64()
            .ok_or(crate::NeoError::Overflow)
    }

    /// Convert to i32, saturating at the boundaries if the value is out of range.
    /// This never panics.
    pub fn as_i32_saturating(&self) -> i32 {
        self.0.to_i32().unwrap_or_else(|| {
            if self.0.sign() == num_bigint::Sign::Minus {
                i32::MIN
            } else {
                i32::MAX
            }
        })
    }

    /// Convert to u32, saturating at the boundaries if the value is out of range.
    /// This never panics.
    pub fn as_u32_saturating(&self) -> u32 {
        self.0.to_u32().unwrap_or_else(|| {
            if self.0.sign() == num_bigint::Sign::Minus {
                0
            } else {
                u32::MAX
            }
        })
    }

    /// Convert to i64, saturating at the boundaries if the value is out of range.
    /// This never panics.
    pub fn as_i64_saturating(&self) -> i64 {
        self.0.to_i64().unwrap_or_else(|| {
            if self.0.sign() == num_bigint::Sign::Minus {
                i64::MIN
            } else {
                i64::MAX
            }
        })
    }

    /// Deprecated compatibility helper that converts to `i32` using saturating semantics.
    #[deprecated(
        since = "0.4.1",
        note = "Use try_as_i32() or as_i32_saturating() explicitly"
    )]
    pub fn as_i32(&self) -> i32 {
        self.as_i32_saturating()
    }

    /// Deprecated compatibility helper that converts to `u32` using saturating semantics.
    #[deprecated(
        since = "0.4.1",
        note = "Use try_as_u32() or as_u32_saturating() explicitly"
    )]
    pub fn as_u32(&self) -> u32 {
        self.as_u32_saturating()
    }

    pub fn to_i32(&self) -> Option<i32> {
        self.0.to_i32()
    }

    pub fn to_u32(&self) -> Option<u32> {
        self.0.to_u32()
    }

    pub fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }
}

impl Not for NeoInteger {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Shl<u32> for NeoInteger {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl Shl<u32> for &NeoInteger {
    type Output = NeoInteger;
    fn shl(self, rhs: u32) -> Self::Output {
        NeoInteger::new(self.0.clone() << rhs)
    }
}

impl Shr<u32> for NeoInteger {
    type Output = Self;
    fn shr(self, rhs: u32) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl Shr<u32> for &NeoInteger {
    type Output = NeoInteger;
    fn shr(self, rhs: u32) -> Self::Output {
        NeoInteger::new(self.0.clone() >> rhs)
    }
}

impl From<i32> for NeoInteger {
    fn from(value: i32) -> Self {
        NeoInteger::new(value)
    }
}

impl From<i64> for NeoInteger {
    fn from(value: i64) -> Self {
        NeoInteger::new(value)
    }
}

impl From<u32> for NeoInteger {
    fn from(value: u32) -> Self {
        NeoInteger::new(value)
    }
}

impl From<BigInt> for NeoInteger {
    fn from(value: BigInt) -> Self {
        NeoInteger::new(value)
    }
}

impl From<&BigInt> for NeoInteger {
    fn from(value: &BigInt) -> Self {
        NeoInteger::new(value.clone())
    }
}

impl Add for NeoInteger {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<&NeoInteger> for NeoInteger {
    type Output = Self;
    fn add(self, rhs: &NeoInteger) -> Self::Output {
        Self(self.0 + &rhs.0)
    }
}

impl Add<NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn add(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 + rhs.0)
    }
}

impl Add<&NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn add(self, rhs: &NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 + &rhs.0)
    }
}

impl Sub for NeoInteger {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<&NeoInteger> for NeoInteger {
    type Output = Self;
    fn sub(self, rhs: &NeoInteger) -> Self::Output {
        Self(self.0 - &rhs.0)
    }
}

impl Sub<NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn sub(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 - rhs.0)
    }
}

impl Sub<&NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn sub(self, rhs: &NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 - &rhs.0)
    }
}

impl Mul for NeoInteger {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Mul<&NeoInteger> for NeoInteger {
    type Output = Self;
    fn mul(self, rhs: &NeoInteger) -> Self::Output {
        Self(self.0 * &rhs.0)
    }
}

impl Mul<NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn mul(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 * rhs.0)
    }
}

impl Mul<&NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn mul(self, rhs: &NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 * &rhs.0)
    }
}

impl Div for NeoInteger {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Div<&NeoInteger> for NeoInteger {
    type Output = Self;
    fn div(self, rhs: &NeoInteger) -> Self::Output {
        Self(self.0 / &rhs.0)
    }
}

impl Div<NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn div(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 / rhs.0)
    }
}

impl Div<&NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn div(self, rhs: &NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 / &rhs.0)
    }
}

impl Rem for NeoInteger {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl Rem<&NeoInteger> for NeoInteger {
    type Output = Self;
    fn rem(self, rhs: &NeoInteger) -> Self::Output {
        Self(self.0 % &rhs.0)
    }
}

impl Rem<NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn rem(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 % rhs.0)
    }
}

impl Rem<&NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn rem(self, rhs: &NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 % &rhs.0)
    }
}

impl BitAnd for NeoInteger {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAnd<&NeoInteger> for NeoInteger {
    type Output = Self;
    fn bitand(self, rhs: &NeoInteger) -> Self::Output {
        Self(self.0 & &rhs.0)
    }
}

impl BitAnd<NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn bitand(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 & rhs.0)
    }
}

impl BitAnd<&NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn bitand(self, rhs: &NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 & &rhs.0)
    }
}

impl BitOr for NeoInteger {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOr<&NeoInteger> for NeoInteger {
    type Output = Self;
    fn bitor(self, rhs: &NeoInteger) -> Self::Output {
        Self(self.0 | &rhs.0)
    }
}

impl BitOr<NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn bitor(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 | rhs.0)
    }
}

impl BitOr<&NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn bitor(self, rhs: &NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 | &rhs.0)
    }
}

impl BitXor for NeoInteger {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXor<&NeoInteger> for NeoInteger {
    type Output = Self;
    fn bitxor(self, rhs: &NeoInteger) -> Self::Output {
        Self(self.0 ^ &rhs.0)
    }
}

impl BitXor<NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn bitxor(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 ^ rhs.0)
    }
}

impl BitXor<&NeoInteger> for &NeoInteger {
    type Output = NeoInteger;
    fn bitxor(self, rhs: &NeoInteger) -> Self::Output {
        NeoInteger::new(&self.0 ^ &rhs.0)
    }
}

impl Default for NeoInteger {
    fn default() -> Self {
        NeoInteger::zero()
    }
}
