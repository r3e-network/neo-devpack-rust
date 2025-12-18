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

    pub fn as_i32(&self) -> i32 {
        self.0.to_i32().expect("NeoInteger value exceeds i32 range")
    }

    pub fn as_u32(&self) -> u32 {
        self.0.to_u32().expect("NeoInteger value exceeds u32 range")
    }

    pub fn to_i32(&self) -> Option<i32> {
        self.0.to_i32()
    }

    pub fn to_u32(&self) -> Option<u32> {
        self.0.to_u32()
    }
}

impl Add for NeoInteger {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<'a> Add<&'a NeoInteger> for NeoInteger {
    type Output = Self;
    fn add(self, rhs: &'a NeoInteger) -> Self::Output {
        Self(self.0 + rhs.0.clone())
    }
}

impl<'a> Add<NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn add(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() + rhs.0)
    }
}

impl<'a, 'b> Add<&'b NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn add(self, rhs: &'b NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() + rhs.0.clone())
    }
}

impl Sub for NeoInteger {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<'a> Sub<&'a NeoInteger> for NeoInteger {
    type Output = Self;
    fn sub(self, rhs: &'a NeoInteger) -> Self::Output {
        Self(self.0 - rhs.0.clone())
    }
}

impl<'a> Sub<NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn sub(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() - rhs.0)
    }
}

impl<'a, 'b> Sub<&'b NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn sub(self, rhs: &'b NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() - rhs.0.clone())
    }
}

impl Mul for NeoInteger {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl<'a> Mul<&'a NeoInteger> for NeoInteger {
    type Output = Self;
    fn mul(self, rhs: &'a NeoInteger) -> Self::Output {
        Self(self.0 * rhs.0.clone())
    }
}

impl<'a> Mul<NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn mul(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() * rhs.0)
    }
}

impl<'a, 'b> Mul<&'b NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn mul(self, rhs: &'b NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() * rhs.0.clone())
    }
}

impl Div for NeoInteger {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl<'a> Div<&'a NeoInteger> for NeoInteger {
    type Output = Self;
    fn div(self, rhs: &'a NeoInteger) -> Self::Output {
        Self(self.0 / rhs.0.clone())
    }
}

impl<'a> Div<NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn div(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() / rhs.0)
    }
}

impl<'a, 'b> Div<&'b NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn div(self, rhs: &'b NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() / rhs.0.clone())
    }
}

impl Rem for NeoInteger {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl<'a> Rem<&'a NeoInteger> for NeoInteger {
    type Output = Self;
    fn rem(self, rhs: &'a NeoInteger) -> Self::Output {
        Self(self.0 % rhs.0.clone())
    }
}

impl<'a> Rem<NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn rem(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() % rhs.0)
    }
}

impl<'a, 'b> Rem<&'b NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn rem(self, rhs: &'b NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() % rhs.0.clone())
    }
}

impl BitAnd for NeoInteger {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl<'a> BitAnd<&'a NeoInteger> for NeoInteger {
    type Output = Self;
    fn bitand(self, rhs: &'a NeoInteger) -> Self::Output {
        Self(self.0 & rhs.0.clone())
    }
}

impl<'a> BitAnd<NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn bitand(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() & rhs.0)
    }
}

impl<'a, 'b> BitAnd<&'b NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn bitand(self, rhs: &'b NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() & rhs.0.clone())
    }
}

impl BitOr for NeoInteger {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl<'a> BitOr<&'a NeoInteger> for NeoInteger {
    type Output = Self;
    fn bitor(self, rhs: &'a NeoInteger) -> Self::Output {
        Self(self.0 | rhs.0.clone())
    }
}

impl<'a> BitOr<NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn bitor(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() | rhs.0)
    }
}

impl<'a, 'b> BitOr<&'b NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn bitor(self, rhs: &'b NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() | rhs.0.clone())
    }
}

impl BitXor for NeoInteger {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl<'a> BitXor<&'a NeoInteger> for NeoInteger {
    type Output = Self;
    fn bitxor(self, rhs: &'a NeoInteger) -> Self::Output {
        Self(self.0 ^ rhs.0.clone())
    }
}

impl<'a> BitXor<NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn bitxor(self, rhs: NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() ^ rhs.0)
    }
}

impl<'a, 'b> BitXor<&'b NeoInteger> for &'a NeoInteger {
    type Output = NeoInteger;
    fn bitxor(self, rhs: &'b NeoInteger) -> Self::Output {
        NeoInteger::new(self.0.clone() ^ rhs.0.clone())
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

impl<'a> Shl<u32> for &'a NeoInteger {
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

impl<'a> Shr<u32> for &'a NeoInteger {
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

impl Default for NeoInteger {
    fn default() -> Self {
        NeoInteger::zero()
    }
}

