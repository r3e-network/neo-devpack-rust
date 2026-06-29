// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use std::fmt;
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

    /// Maximum byte length of an integer the NeoVM will hold
    /// (`ExecutionEngineLimits.MaxIntegerSize` in C# `neo-project/neo` —
    /// 32 bytes / 256 bits, two's-complement). The VM FAULTs when an
    /// operation produces an integer wider than this.
    pub const MAX_BYTE_LENGTH: usize = 32;

    /// Whether this value fits within the NeoVM's 256-bit integer bound.
    ///
    /// Host-mode arithmetic on `NeoInteger` is arbitrary-precision, so a
    /// computation that overflows 256 bits succeeds off-chain but FAULTs
    /// on-chain. Call this on values derived from untrusted input or
    /// unbounded accumulation to detect the divergence before it reaches
    /// the VM. Zero is one byte in `num-bigint`'s representation but is
    /// trivially within bounds.
    pub fn fits_in_neovm(&self) -> bool {
        if self.0.is_zero() {
            return true;
        }
        self.0.to_signed_bytes_le().len() <= Self::MAX_BYTE_LENGTH
    }

    /// Checked division: returns `Err(DivisionByZero)` instead of panicking
    /// (the `Div` operator faults on a zero divisor — on-chain that becomes a
    /// VM FAULT reverting the whole transaction). Use this in any contract
    /// path that cannot guarantee a non-zero divisor (D5).
    pub fn try_div(&self, rhs: &NeoInteger) -> crate::NeoResult<NeoInteger> {
        if rhs.0.is_zero() {
            return Err(crate::NeoError::DivisionByZero);
        }
        Ok(NeoInteger::new(&self.0 / &rhs.0))
    }

    /// Checked remainder: returns `Err(DivisionByZero)` instead of panicking
    /// (see `try_div`). Matches the `Rem` operator's fault-on-zero behaviour.
    pub fn try_rem(&self, rhs: &NeoInteger) -> crate::NeoResult<NeoInteger> {
        if rhs.0.is_zero() {
            return Err(crate::NeoError::DivisionByZero);
        }
        Ok(NeoInteger::new(&self.0 % &rhs.0))
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

    /// Owned `BigInt` (clones the inner `BigInt`). Use this when you
    /// need to keep the value past the `NeoInteger`'s lifetime
    /// (e.g. pass to an API that expects `BigInt` by value).
    pub fn to_bigint(&self) -> BigInt {
        self.0.clone()
    }

    /// Construct a `NeoInteger` from a `BigInt`. The `From<BigInt>`
    /// impl already provides this, but spelled as a method for
    /// uniformity with `to_bigint` and for code that has the
    /// `BigInt` behind a reference.
    pub fn from_bigint(value: &BigInt) -> Self {
        Self(value.clone())
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
        self.0.to_i32().ok_or(crate::NeoError::Overflow)
    }

    /// Convert to u32, returning `Result` for ergonomic `?` usage.
    pub fn try_into_u32(&self) -> crate::NeoResult<u32> {
        self.0.to_u32().ok_or(crate::NeoError::Overflow)
    }

    /// Convert to i64, returning `Result` for ergonomic `?` usage.
    pub fn try_into_i64(&self) -> crate::NeoResult<i64> {
        self.0.to_i64().ok_or(crate::NeoError::Overflow)
    }

    /// Convert to u64, returning `Result` for ergonomic `?` usage.
    pub fn try_into_u64(&self) -> crate::NeoResult<u64> {
        self.0.to_u64().ok_or(crate::NeoError::Overflow)
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
        since = "0.1.0",
        note = "Use try_as_i32() or as_i32_saturating() explicitly"
    )]
    pub fn as_i32(&self) -> i32 {
        self.as_i32_saturating()
    }

    /// Deprecated compatibility helper that converts to `u32` using saturating semantics.
    #[deprecated(
        since = "0.1.0",
        note = "Use try_as_u32() or as_u32_saturating() explicitly"
    )]
    pub fn as_u32(&self) -> u32 {
        self.as_u32_saturating()
    }

    /// Deprecated: use `try_as_i32()` instead.
    #[deprecated(since = "0.1.0", note = "Use try_as_i32() instead")]
    pub fn to_i32(&self) -> Option<i32> {
        self.0.to_i32()
    }

    /// Deprecated: use `try_as_u32()` instead.
    #[deprecated(since = "0.1.0", note = "Use try_as_u32() instead")]
    pub fn to_u32(&self) -> Option<u32> {
        self.0.to_u32()
    }

    /// Deprecated: use `try_as_i64()` instead.
    #[deprecated(since = "0.1.0", note = "Use try_as_i64() instead")]
    pub fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }
}

impl fmt::Display for NeoInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

impl From<i8> for NeoInteger {
    fn from(value: i8) -> Self {
        NeoInteger::new(value)
    }
}

impl From<u8> for NeoInteger {
    fn from(value: u8) -> Self {
        NeoInteger::new(value)
    }
}

impl From<i16> for NeoInteger {
    fn from(value: i16) -> Self {
        NeoInteger::new(value)
    }
}

impl From<u16> for NeoInteger {
    fn from(value: u16) -> Self {
        NeoInteger::new(value)
    }
}

impl From<i32> for NeoInteger {
    fn from(value: i32) -> Self {
        NeoInteger::new(value)
    }
}

impl From<u32> for NeoInteger {
    fn from(value: u32) -> Self {
        NeoInteger::new(value)
    }
}

impl From<i64> for NeoInteger {
    fn from(value: i64) -> Self {
        NeoInteger::new(value)
    }
}

impl From<u64> for NeoInteger {
    fn from(value: u64) -> Self {
        NeoInteger::new(value)
    }
}

impl From<i128> for NeoInteger {
    fn from(value: i128) -> Self {
        NeoInteger::new(value)
    }
}

impl From<u128> for NeoInteger {
    fn from(value: u128) -> Self {
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

#[cfg(test)]
mod d5_tests {
    use super::*;

    #[test]
    fn try_div_returns_division_by_zero_instead_of_panicking() {
        let a = NeoInteger::new(10);
        let zero = NeoInteger::zero();
        assert_eq!(a.try_div(&zero), Err(crate::NeoError::DivisionByZero));
        let two = NeoInteger::new(2);
        assert_eq!(a.try_div(&two).unwrap(), NeoInteger::new(5));
    }

    #[test]
    fn try_rem_returns_division_by_zero_instead_of_panicking() {
        let a = NeoInteger::new(10);
        let zero = NeoInteger::zero();
        assert_eq!(a.try_rem(&zero), Err(crate::NeoError::DivisionByZero));
        let three = NeoInteger::new(3);
        assert_eq!(a.try_rem(&three).unwrap(), NeoInteger::new(1));
    }

    #[test]
    fn fits_in_neovm_tracks_the_256_bit_bound() {
        assert!(NeoInteger::zero().fits_in_neovm());
        assert!(NeoInteger::new(i64::MAX).fits_in_neovm());
        assert!(NeoInteger::new(i64::MIN).fits_in_neovm());

        // 2^255 - 1 is the largest positive value that fits in 32 bytes.
        let max_pos = (BigInt::one() << 255) - BigInt::one();
        assert!(NeoInteger::from_bigint(&max_pos).fits_in_neovm());
        assert_eq!(
            max_pos.to_signed_bytes_le().len(),
            NeoInteger::MAX_BYTE_LENGTH
        );

        // 2^255 needs 33 bytes (a sign byte), so it overflows the bound.
        let over = BigInt::one() << 255;
        assert!(!NeoInteger::from_bigint(&over).fits_in_neovm());

        // A clearly oversized value (2^256) also fails.
        assert!(!NeoInteger::from_bigint(&(BigInt::one() << 256)).fits_in_neovm());
    }
}
