// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: integer arithmetic, bitwise, shift, rotate,
//! bit-count, comparison and conversion WASM ops, plus the `NeoInteger`
//! (256-bit BigInteger) operator and conversion surface.
//!
//! Every exported method takes/returns scalar `i64`/`bool`/`i32`/`u32`
//! (the wasm32->NeoVM export ABI); the richer `NeoInteger` work happens
//! inside method bodies. Release builds wrap on overflow, matching WASM.

use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureArithmetic" }"#);

#[neo_contract]
pub struct ArithmeticContract;

#[neo_contract]
impl ArithmeticContract {
    pub fn new() -> Self {
        Self
    }

    /// i64 add / sub / mul / div_s / rem_s (constant, non-zero divisors).
    #[neo_method(safe)]
    pub fn calc(a: i64, b: i64) -> i64 {
        a.wrapping_add(b).wrapping_mul(3) - b / 2 + a % 7
    }

    /// i64 unsigned div_u / rem_u.
    #[neo_method(safe)]
    pub fn calc_u(a: i64, b: i64) -> i64 {
        if b == 0 {
            return 0;
        }
        let (au, bu) = (a as u64, b as u64);
        (au / bu).wrapping_add(au % bu) as i64
    }

    /// Bitwise AND / OR / XOR / NOT.
    #[neo_method(safe)]
    pub fn bits(a: i64, b: i64) -> i64 {
        !((a & b) | (a ^ b))
    }

    /// shl / shr_u / shr_s (shift counts masked mod 64 by WASM semantics).
    #[neo_method(safe)]
    pub fn shifts(a: i64, n: u32) -> i64 {
        let s = n & 63;
        (a.wrapping_shl(s))
            .wrapping_add(((a as u64) >> s) as i64)
            .wrapping_add(a >> s)
    }

    /// i32 rotate_left / rotate_right.
    #[neo_method(safe)]
    pub fn rot(a: i32, n: u32) -> i32 {
        a.rotate_left(n) ^ a.rotate_right(n)
    }

    /// clz / ctz / popcnt.
    #[neo_method(safe)]
    pub fn bitcount(a: i64) -> i64 {
        (a.leading_zeros() + a.trailing_zeros() + a.count_ones()) as i64
    }

    /// eq / ne / lt_s / gt_u / eqz comparisons.
    #[neo_method(safe)]
    pub fn cmp(a: i64, b: i64) -> bool {
        a == b || a < b || (a as u64) > (b as u64) || a != 0
    }

    /// i32.wrap_i64.
    #[neo_method(safe)]
    pub fn narrow(a: i64) -> i32 {
        a as i32
    }

    /// i64.extend_i32_s + extend_i32_u.
    #[neo_method(safe)]
    pub fn widen(a: i32) -> i64 {
        (a as i64).wrapping_add((a as u32) as i64)
    }

    /// extend8_s / extend16_s (sign-extension of narrow values).
    #[neo_method(safe)]
    pub fn sx(a: i32) -> i64 {
        (((a as i8) as i32) + ((a as i16) as i32)) as i64
    }

    /// NeoInteger (256-bit) operators + checked div/rem + range guard.
    #[neo_method(safe)]
    pub fn bigmath(a: i64, b: i64) -> i64 {
        let x = NeoInteger::new(a);
        let y = NeoInteger::new(b);
        let mut acc = (x.clone() + y.clone()) - NeoInteger::new(1);
        acc = acc * NeoInteger::new(2);
        acc = (acc << 1) >> 1;
        acc = (acc.clone() & y.clone()) | (x.clone() ^ y.clone());
        // Checked division/remainder (fault-safe on zero divisor).
        if let Ok(q) = acc.try_div(&NeoInteger::new(3)) {
            acc = q;
        }
        if let Ok(r) = acc.try_rem(&NeoInteger::new(7)) {
            acc = acc + r;
        }
        // 256-bit bound guard, then narrow back to the scalar boundary.
        if !acc.fits_in_neovm() {
            return 0;
        }
        acc.as_i64_saturating()
    }

    /// NeoInteger try_as_* / try_into_* / saturating conversions.
    #[neo_method(safe)]
    pub fn narrowing(a: i64) -> i64 {
        let n = NeoInteger::new(a);
        let i32v = n.try_as_i32().unwrap_or(-1) as i64;
        let i64v = n.try_into_i64().unwrap_or(-1);
        let u32v = n.as_u32_saturating() as i64;
        i32v.wrapping_add(i64v).wrapping_add(u32v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_methods_compute() {
        assert_eq!(ArithmeticContract::calc(10, 4), 10i64.wrapping_add(4) * 3 - 2 + 3);
        assert_eq!(ArithmeticContract::calc_u(10, 3), (3 + 1) as i64);
        assert_eq!(ArithmeticContract::calc_u(10, 0), 0);
        assert_eq!(ArithmeticContract::bitcount(0), 64);
        assert!(ArithmeticContract::cmp(0, 0));
        assert_eq!(ArithmeticContract::narrow(0x1_0000_0001), 1);
        assert_eq!(ArithmeticContract::sx(0xFF), -1 + -1);
    }
}
