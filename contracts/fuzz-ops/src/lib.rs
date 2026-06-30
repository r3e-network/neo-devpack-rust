// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Differential-fuzz target: a wide, single-op-per-method surface used to hunt
//! translator bugs by comparing the real NeoVM (via the conformance oracle)
//! against native Rust execution of the *same* `ops::*` functions.
//!
//! The op logic lives in [`ops`] as plain `pub fn`s. The `#[neo_method]`
//! wrappers (→ wasm → NeoVM) and the `refgen` binary (→ native ground truth)
//! both call the identical functions, so any oracle/native mismatch is a
//! translator lowering bug, not a reference-derivation artifact.
//!
//! Every op is trap-free and well-defined: divisions guard the zero divisor
//! (returning [`ops::GUARD`]), shifts/rotates mask the count, and arithmetic
//! wraps — matching WASM semantics on release builds.

use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FuzzOps" }"#);

/// Randomly-generated expression ops (regenerated each fuzz round by the
/// `gen_exprs` bin). Exposed via the `gen(idx, a, b)` export below and
/// evaluated natively by `refgen` — same code both ways.
pub mod generated_ops;

/// Trap-free op implementations shared by the exports and the native refgen.
pub mod ops {
    use neo_devpack::prelude::*;

    /// Sentinel returned by div/rem when the divisor is zero (keeps the op
    /// total and identical on both execution paths).
    pub const GUARD: i64 = i64::MIN;

    // ---- i64 arithmetic ----
    pub fn add(a: i64, b: i64) -> i64 {
        a.wrapping_add(b)
    }
    pub fn sub(a: i64, b: i64) -> i64 {
        a.wrapping_sub(b)
    }
    pub fn mul(a: i64, b: i64) -> i64 {
        a.wrapping_mul(b)
    }
    pub fn div_s(a: i64, b: i64) -> i64 {
        if b == 0 {
            GUARD
        } else {
            a.wrapping_div(b)
        }
    }
    pub fn rem_s(a: i64, b: i64) -> i64 {
        if b == 0 {
            GUARD
        } else {
            a.wrapping_rem(b)
        }
    }
    pub fn div_u(a: i64, b: i64) -> i64 {
        if b == 0 {
            GUARD
        } else {
            ((a as u64) / (b as u64)) as i64
        }
    }
    pub fn rem_u(a: i64, b: i64) -> i64 {
        if b == 0 {
            GUARD
        } else {
            ((a as u64) % (b as u64)) as i64
        }
    }
    pub fn neg(a: i64, _b: i64) -> i64 {
        a.wrapping_neg()
    }
    pub fn abs(a: i64, _b: i64) -> i64 {
        a.wrapping_abs()
    }

    // ---- i64 bitwise / shift / rotate ----
    pub fn and(a: i64, b: i64) -> i64 {
        a & b
    }
    pub fn or(a: i64, b: i64) -> i64 {
        a | b
    }
    pub fn xor(a: i64, b: i64) -> i64 {
        a ^ b
    }
    pub fn not(a: i64, _b: i64) -> i64 {
        !a
    }
    pub fn shl(a: i64, b: i64) -> i64 {
        a.wrapping_shl(b as u32)
    }
    pub fn shr_s(a: i64, b: i64) -> i64 {
        a.wrapping_shr(b as u32)
    }
    pub fn shr_u(a: i64, b: i64) -> i64 {
        (a as u64).wrapping_shr(b as u32) as i64
    }
    pub fn rotl(a: i64, b: i64) -> i64 {
        a.rotate_left((b & 63) as u32)
    }
    pub fn rotr(a: i64, b: i64) -> i64 {
        a.rotate_right((b & 63) as u32)
    }

    // ---- i64 bit-count / test ----
    pub fn clz(a: i64, _b: i64) -> i64 {
        a.leading_zeros() as i64
    }
    pub fn ctz(a: i64, _b: i64) -> i64 {
        a.trailing_zeros() as i64
    }
    pub fn popcnt(a: i64, _b: i64) -> i64 {
        a.count_ones() as i64
    }
    pub fn eqz(a: i64, _b: i64) -> i64 {
        (a == 0) as i64
    }

    // ---- i64 comparisons (return 0/1) ----
    pub fn eq(a: i64, b: i64) -> i64 {
        (a == b) as i64
    }
    pub fn ne(a: i64, b: i64) -> i64 {
        (a != b) as i64
    }
    pub fn lt_s(a: i64, b: i64) -> i64 {
        (a < b) as i64
    }
    pub fn le_s(a: i64, b: i64) -> i64 {
        (a <= b) as i64
    }
    pub fn gt_s(a: i64, b: i64) -> i64 {
        (a > b) as i64
    }
    pub fn ge_s(a: i64, b: i64) -> i64 {
        (a >= b) as i64
    }
    pub fn lt_u(a: i64, b: i64) -> i64 {
        ((a as u64) < (b as u64)) as i64
    }
    pub fn le_u(a: i64, b: i64) -> i64 {
        ((a as u64) <= (b as u64)) as i64
    }
    pub fn gt_u(a: i64, b: i64) -> i64 {
        ((a as u64) > (b as u64)) as i64
    }
    pub fn ge_u(a: i64, b: i64) -> i64 {
        ((a as u64) >= (b as u64)) as i64
    }
    pub fn min_s(a: i64, b: i64) -> i64 {
        a.min(b)
    }
    pub fn max_s(a: i64, b: i64) -> i64 {
        a.max(b)
    }

    // ---- i32 ops (operate on the low 32 bits, return sign-extended i64) ----
    pub fn i32_add(a: i64, b: i64) -> i64 {
        (a as i32).wrapping_add(b as i32) as i64
    }
    pub fn i32_sub(a: i64, b: i64) -> i64 {
        (a as i32).wrapping_sub(b as i32) as i64
    }
    pub fn i32_mul(a: i64, b: i64) -> i64 {
        (a as i32).wrapping_mul(b as i32) as i64
    }
    pub fn i32_div_s(a: i64, b: i64) -> i64 {
        let b = b as i32;
        if b == 0 {
            GUARD
        } else {
            (a as i32).wrapping_div(b) as i64
        }
    }
    pub fn i32_rem_s(a: i64, b: i64) -> i64 {
        let b = b as i32;
        if b == 0 {
            GUARD
        } else {
            (a as i32).wrapping_rem(b) as i64
        }
    }
    pub fn i32_div_u(a: i64, b: i64) -> i64 {
        let b = b as u32;
        if b == 0 {
            GUARD
        } else {
            ((a as u32) / b) as i32 as i64
        }
    }
    pub fn i32_rem_u(a: i64, b: i64) -> i64 {
        let b = b as u32;
        if b == 0 {
            GUARD
        } else {
            ((a as u32) % b) as i32 as i64
        }
    }
    pub fn i32_shl(a: i64, b: i64) -> i64 {
        (a as i32).wrapping_shl(b as u32) as i64
    }
    pub fn i32_shr_s(a: i64, b: i64) -> i64 {
        (a as i32).wrapping_shr(b as u32) as i64
    }
    pub fn i32_shr_u(a: i64, b: i64) -> i64 {
        (a as u32).wrapping_shr(b as u32) as i32 as i64
    }
    pub fn i32_rotl(a: i64, b: i64) -> i64 {
        (a as i32).rotate_left((b & 31) as u32) as i64
    }
    pub fn i32_rotr(a: i64, b: i64) -> i64 {
        (a as i32).rotate_right((b & 31) as u32) as i64
    }
    pub fn i32_clz(a: i64, _b: i64) -> i64 {
        (a as i32).leading_zeros() as i64
    }
    pub fn i32_ctz(a: i64, _b: i64) -> i64 {
        (a as i32).trailing_zeros() as i64
    }
    pub fn i32_popcnt(a: i64, _b: i64) -> i64 {
        (a as i32).count_ones() as i64
    }

    // ---- conversions ----
    pub fn wrap_i32(a: i64, _b: i64) -> i64 {
        (a as i32) as i64
    }
    pub fn extend_i32_u(a: i64, _b: i64) -> i64 {
        ((a as i32) as u32) as u64 as i64
    }
    pub fn extend8_s(a: i64, _b: i64) -> i64 {
        (a as i8) as i64
    }
    pub fn extend16_s(a: i64, _b: i64) -> i64 {
        (a as i16) as i64
    }
    pub fn extend32_s(a: i64, _b: i64) -> i64 {
        (a as i32) as i64
    }

    // ---- NeoInteger (256-bit BigInteger) ops, reduced via saturating i64 ----
    fn red(n: NeoInteger) -> i64 {
        n.as_i64_saturating()
    }
    pub fn big_add(a: i64, b: i64) -> i64 {
        red(NeoInteger::new(a) + NeoInteger::new(b))
    }
    pub fn big_sub(a: i64, b: i64) -> i64 {
        red(NeoInteger::new(a) - NeoInteger::new(b))
    }
    pub fn big_mul(a: i64, b: i64) -> i64 {
        red(NeoInteger::new(a) * NeoInteger::new(b))
    }
    pub fn big_div(a: i64, b: i64) -> i64 {
        match NeoInteger::new(a).try_div(&NeoInteger::new(b)) {
            Ok(q) => red(q),
            Err(_) => GUARD,
        }
    }
    pub fn big_rem(a: i64, b: i64) -> i64 {
        match NeoInteger::new(a).try_rem(&NeoInteger::new(b)) {
            Ok(r) => red(r),
            Err(_) => GUARD,
        }
    }
    pub fn big_and(a: i64, b: i64) -> i64 {
        red(NeoInteger::new(a) & NeoInteger::new(b))
    }
    pub fn big_or(a: i64, b: i64) -> i64 {
        red(NeoInteger::new(a) | NeoInteger::new(b))
    }
    pub fn big_xor(a: i64, b: i64) -> i64 {
        red(NeoInteger::new(a) ^ NeoInteger::new(b))
    }
    pub fn big_shl(a: i64, b: i64) -> i64 {
        red(NeoInteger::new(a) << ((b & 63) as u32))
    }
    pub fn big_shr(a: i64, b: i64) -> i64 {
        red(NeoInteger::new(a) >> ((b & 63) as u32))
    }
    pub fn big_neg(a: i64, _b: i64) -> i64 {
        red(NeoInteger::zero() - NeoInteger::new(a))
    }

    // ---- chained / boolean-operand comparisons ----
    // These feed a comparison RESULT (a NeoVM Boolean) into another
    // eqz/eq/ne/and/or/xor, exercising the type-strict-EQUAL bug class that
    // raw-integer comparisons cannot reach.
    pub fn not_lt(a: i64, b: i64) -> i64 {
        (!(a < b)) as i64 // eqz applied to the LT Boolean
    }
    pub fn double_neg_lt(a: i64, b: i64) -> i64 {
        (!!(a < b)) as i64 // eqz(eqz(Boolean))
    }
    pub fn eq_cmps(a: i64, b: i64) -> i64 {
        ((a < b) == (b < a)) as i64 // eq of two Booleans
    }
    pub fn ne_cmps(a: i64, b: i64) -> i64 {
        ((a < b) != (a == b)) as i64 // ne of two Booleans
    }
    pub fn and_cmps(a: i64, b: i64) -> i64 {
        ((a < b) & (a != 0)) as i64 // bitand of two Booleans
    }
    pub fn or_cmps(a: i64, b: i64) -> i64 {
        ((a > b) | (b == 0)) as i64 // bitor of two Booleans
    }
    pub fn xor_cmps(a: i64, b: i64) -> i64 {
        ((a < b) ^ (a > b)) as i64 // bitxor of two Booleans
    }
    pub fn cmp_chain(a: i64, b: i64) -> i64 {
        // a longer chain mixing eqz/eq through if/else (the original bug shape)
        if (a < b) == (b > a) {
            if a == 0 {
                1
            } else {
                2
            }
        } else {
            3
        }
    }

    // ---- unsigned min/max (unsigned compare + select) and i32 eqz ----
    pub fn min_u(a: i64, b: i64) -> i64 {
        (a as u64).min(b as u64) as i64
    }
    pub fn max_u(a: i64, b: i64) -> i64 {
        (a as u64).max(b as u64) as i64
    }
    pub fn i32_eqz(a: i64, _b: i64) -> i64 {
        ((a as i32) == 0) as i64
    }
    pub fn i32_eq(a: i64, b: i64) -> i64 {
        ((a as i32) == (b as i32)) as i64
    }
    pub fn i32_ne(a: i64, b: i64) -> i64 {
        ((a as i32) != (b as i32)) as i64
    }

    // ---- control-flow patterns (loops / br_table / recursion / select) ----
    pub fn nested_loop(a: i64, b: i64) -> i64 {
        let n = a.rem_euclid(40);
        let m = b.rem_euclid(40);
        let mut s: i64 = 0;
        for i in 0..n {
            for j in 0..m {
                s = s.wrapping_add(i.wrapping_mul(j));
            }
        }
        s
    }
    pub fn loop_continue(a: i64, b: i64) -> i64 {
        let n = a.rem_euclid(200);
        let mut s: i64 = 0;
        let mut i = 0i64;
        while i < n {
            i += 1;
            if i % (b.rem_euclid(5) + 1) == 0 {
                continue;
            }
            s = s.wrapping_add(i);
        }
        s
    }
    pub fn multi_break(a: i64, b: i64) -> i64 {
        let mut s: i64 = 0;
        let mut i = 0i64;
        loop {
            if i >= a.rem_euclid(1000) {
                break;
            }
            s = s.wrapping_add(i);
            if s > b.rem_euclid(10000) {
                break;
            }
            i += 1;
        }
        s
    }
    pub fn br_wide(a: i64, _b: i64) -> i64 {
        match a.rem_euclid(20) {
            0 => 1000,
            1 => 1001,
            2 => 1002,
            3 => 1003,
            4 => 1004,
            5 => 1005,
            6 => 1006,
            7 => 1007,
            8 => 1008,
            9 => 1009,
            10 => 1010,
            11 => 1011,
            12 => 1012,
            13 => 1013,
            14 => 1014,
            15 => 1015,
            16 => 1016,
            17 => 1017,
            18 => 1018,
            _ => 1019,
        }
    }
    pub fn recurse_sum(a: i64, _b: i64) -> i64 {
        fn go(n: i64) -> i64 {
            if n <= 0 {
                0
            } else {
                n.wrapping_add(go(n - 1))
            }
        }
        go(a.rem_euclid(500))
    }
    pub fn cond_nest(a: i64, b: i64) -> i64 {
        let r = if a > b {
            if a > 0 {
                a.wrapping_sub(b)
            } else {
                b.wrapping_sub(a)
            }
        } else if a == b {
            0
        } else {
            -(b.wrapping_sub(a))
        };
        r.wrapping_add(if (a ^ b) < 0 { 1 } else { 2 })
    }

    // ---- conversion chains ----
    pub fn sx_chain(a: i64, _b: i64) -> i64 {
        // chained sign-extends of narrowing widths
        let x = (a as i8) as i64;
        let y = (a as i16) as i64;
        let z = (a as i32) as i64;
        x.wrapping_add(y).wrapping_add(z)
    }
    pub fn wrap_roundtrip(a: i64, _b: i64) -> i64 {
        let lo = (a as i32) as i64; // sign-extended low 32
        let u = ((a as u32) as u64) as i64; // zero-extended low 32
        lo ^ u
    }
    pub fn narrow_widen(a: i64, b: i64) -> i64 {
        // mix wrap + extend + arithmetic
        let x = (a as i32).wrapping_add(b as i32);
        ((x as i64) << 1).wrapping_sub((x as u32) as u64 as i64)
    }

    // ---- NeoInteger: operators + conversions + large-value paths ----
    pub fn big_not(a: i64, _b: i64) -> i64 {
        red(!NeoInteger::new(a))
    }
    pub fn big_div_op(a: i64, b: i64) -> i64 {
        if b == 0 {
            GUARD
        } else {
            red(NeoInteger::new(a) / NeoInteger::new(b))
        }
    }
    pub fn big_rem_op(a: i64, b: i64) -> i64 {
        if b == 0 {
            GUARD
        } else {
            red(NeoInteger::new(a) % NeoInteger::new(b))
        }
    }
    pub fn big_lt(a: i64, b: i64) -> i64 {
        (NeoInteger::new(a) < NeoInteger::new(b)) as i64
    }
    pub fn big_eq(a: i64, b: i64) -> i64 {
        (NeoInteger::new(a) == NeoInteger::new(b)) as i64
    }
    pub fn big_as_i32_sat(a: i64, _b: i64) -> i64 {
        NeoInteger::new(a).as_i32_saturating() as i64
    }
    pub fn big_as_u32_sat(a: i64, _b: i64) -> i64 {
        NeoInteger::new(a).as_u32_saturating() as i64
    }
    pub fn big_try_i32(a: i64, _b: i64) -> i64 {
        match NeoInteger::new(a).try_as_i32() {
            Some(x) => x as i64,
            None => -999_999,
        }
    }
    pub fn big_fits(a: i64, _b: i64) -> i64 {
        NeoInteger::new(a).fits_in_neovm() as i64
    }
    pub fn big_large_mul(a: i64, b: i64) -> i64 {
        // push operands into 128-bit+ territory before reducing (tests the
        // wide-BigInteger multiply + saturating narrowing)
        let x = NeoInteger::new(a) << 40u32;
        let y = NeoInteger::new(b) << 40u32;
        red(x * y)
    }
    pub fn big_chain(a: i64, b: i64) -> i64 {
        // mixed operator chain exercising sign + bit ops on BigInteger
        let x = NeoInteger::new(a);
        let y = NeoInteger::new(b);
        let t = (x.clone() + y.clone()) - NeoInteger::new(1);
        let t = (t & y.clone()) | (x ^ y);
        red(!t)
    }
    // --- control flow: conditional return + br_if that discards buried stack
    // values (exercises handle_branch's br_if -> Function-frame return-arity
    // path; same family as the handle_branch operand-drop bug). ---
    pub fn cond_ret(a: i64, b: i64) -> i64 {
        if a > b {
            return 42;
        }
        if a == 0 {
            return b.wrapping_neg();
        }
        99
    }
    pub fn early_ret_loop(a: i64, b: i64) -> i64 {
        // early return out of a bounded loop (br_if to the function frame across
        // a still-live loop body)
        let mut s = 0i64;
        let mut i = 0i64;
        let n = a.rem_euclid(50);
        let cap = b.rem_euclid(1000);
        while i < n {
            s = s.wrapping_add(i);
            if s > cap {
                return s;
            }
            i += 1;
        }
        s
    }
    pub fn switch_ret(a: i64, b: i64) -> i64 {
        // br_table whose arms mix early-return and fallthrough at differing
        // arities (stresses handle_br_table per-case XDROP)
        match a.rem_euclid(8) {
            0 => return b,
            1 | 2 => b.wrapping_add(1),
            3 => {
                if b < 0 {
                    return -1;
                }
                7
            }
            4..=6 => b.wrapping_mul(2),
            _ => return 0,
        }
    }
    // --- NeoInteger comparisons beyond < and == (LT+NOT lowering of >/<=/>=) ---
    pub fn big_le(a: i64, b: i64) -> i64 {
        (NeoInteger::new(a) <= NeoInteger::new(b)) as i64
    }
    pub fn big_gt(a: i64, b: i64) -> i64 {
        (NeoInteger::new(a) > NeoInteger::new(b)) as i64
    }
    pub fn big_ge(a: i64, b: i64) -> i64 {
        (NeoInteger::new(a) >= NeoInteger::new(b)) as i64
    }
    pub fn big_ne(a: i64, b: i64) -> i64 {
        (NeoInteger::new(a) != NeoInteger::new(b)) as i64
    }
}

/// Dispatch an op by name (shared by the contract test and the refgen binary).
pub fn eval(op: &str, a: i64, b: i64) -> Option<i64> {
    use ops::*;
    let f: fn(i64, i64) -> i64 = match op {
        "add" => add,
        "sub" => sub,
        "mul" => mul,
        "div_s" => div_s,
        "rem_s" => rem_s,
        "div_u" => div_u,
        "rem_u" => rem_u,
        "neg" => neg,
        "abs" => abs,
        "and" => and,
        "or" => or,
        "xor" => xor,
        "not" => not,
        "shl" => shl,
        "shr_s" => shr_s,
        "shr_u" => shr_u,
        "rotl" => rotl,
        "rotr" => rotr,
        "clz" => clz,
        "ctz" => ctz,
        "popcnt" => popcnt,
        "eqz" => eqz,
        "eq" => eq,
        "ne" => ne,
        "lt_s" => lt_s,
        "le_s" => le_s,
        "gt_s" => gt_s,
        "ge_s" => ge_s,
        "lt_u" => lt_u,
        "le_u" => le_u,
        "gt_u" => gt_u,
        "ge_u" => ge_u,
        "min_s" => min_s,
        "max_s" => max_s,
        "i32_add" => i32_add,
        "i32_sub" => i32_sub,
        "i32_mul" => i32_mul,
        "i32_div_s" => i32_div_s,
        "i32_rem_s" => i32_rem_s,
        "i32_div_u" => i32_div_u,
        "i32_rem_u" => i32_rem_u,
        "i32_shl" => i32_shl,
        "i32_shr_s" => i32_shr_s,
        "i32_shr_u" => i32_shr_u,
        "i32_rotl" => i32_rotl,
        "i32_rotr" => i32_rotr,
        "i32_clz" => i32_clz,
        "i32_ctz" => i32_ctz,
        "i32_popcnt" => i32_popcnt,
        "wrap_i32" => wrap_i32,
        "extend_i32_u" => extend_i32_u,
        "extend8_s" => extend8_s,
        "extend16_s" => extend16_s,
        "extend32_s" => extend32_s,
        "big_add" => big_add,
        "big_sub" => big_sub,
        "big_mul" => big_mul,
        "big_div" => big_div,
        "big_rem" => big_rem,
        "big_and" => big_and,
        "big_or" => big_or,
        "big_xor" => big_xor,
        "big_shl" => big_shl,
        "big_shr" => big_shr,
        "big_neg" => big_neg,
        "not_lt" => not_lt,
        "double_neg_lt" => double_neg_lt,
        "eq_cmps" => eq_cmps,
        "ne_cmps" => ne_cmps,
        "and_cmps" => and_cmps,
        "or_cmps" => or_cmps,
        "xor_cmps" => xor_cmps,
        "cmp_chain" => cmp_chain,
        "min_u" => min_u,
        "max_u" => max_u,
        "i32_eqz" => i32_eqz,
        "i32_eq" => i32_eq,
        "i32_ne" => i32_ne,
        "nested_loop" => nested_loop,
        "loop_continue" => loop_continue,
        "multi_break" => multi_break,
        "br_wide" => br_wide,
        "recurse_sum" => recurse_sum,
        "cond_nest" => cond_nest,
        "sx_chain" => sx_chain,
        "wrap_roundtrip" => wrap_roundtrip,
        "narrow_widen" => narrow_widen,
        "big_not" => big_not,
        "big_div_op" => big_div_op,
        "big_rem_op" => big_rem_op,
        "big_lt" => big_lt,
        "big_eq" => big_eq,
        "big_as_i32_sat" => big_as_i32_sat,
        "big_as_u32_sat" => big_as_u32_sat,
        "big_try_i32" => big_try_i32,
        "big_fits" => big_fits,
        "big_large_mul" => big_large_mul,
        "big_chain" => big_chain,
        "cond_ret" => cond_ret,
        "early_ret_loop" => early_ret_loop,
        "switch_ret" => switch_ret,
        "big_le" => big_le,
        "big_gt" => big_gt,
        "big_ge" => big_ge,
        "big_ne" => big_ne,
        _ => return None,
    };
    Some(f(a, b))
}

/// The full op name list (snake_case == inherent method name; the export name
/// is the camelCase form).
pub const OP_NAMES: &[&str] = &[
    "add", "sub", "mul", "div_s", "rem_s", "div_u", "rem_u", "neg", "abs", "and", "or", "xor",
    "not", "shl", "shr_s", "shr_u", "rotl", "rotr", "clz", "ctz", "popcnt", "eqz", "eq", "ne",
    "lt_s", "le_s", "gt_s", "ge_s", "lt_u", "le_u", "gt_u", "ge_u", "min_s", "max_s", "i32_add",
    "i32_sub", "i32_mul", "i32_div_s", "i32_rem_s", "i32_div_u", "i32_rem_u", "i32_shl",
    "i32_shr_s", "i32_shr_u", "i32_rotl", "i32_rotr", "i32_clz", "i32_ctz", "i32_popcnt",
    "wrap_i32", "extend_i32_u", "extend8_s", "extend16_s", "extend32_s", "big_add", "big_sub",
    "big_mul", "big_div", "big_rem", "big_and", "big_or", "big_xor", "big_shl", "big_shr",
    "big_neg", "not_lt", "double_neg_lt", "eq_cmps", "ne_cmps", "and_cmps", "or_cmps", "xor_cmps",
    "cmp_chain", "min_u", "max_u", "i32_eqz", "i32_eq", "i32_ne",
    "nested_loop", "loop_continue", "multi_break", "br_wide", "recurse_sum", "cond_nest", "sx_chain", "wrap_roundtrip", "narrow_widen", "big_not", "big_div_op", "big_rem_op", "big_lt", "big_eq", "big_as_i32_sat", "big_as_u32_sat", "big_try_i32", "big_fits", "big_large_mul", "big_chain",
    "cond_ret", "early_ret_loop", "switch_ret", "big_le", "big_gt", "big_ge", "big_ne",
];

#[neo_contract]
pub struct FuzzOps;

// The exported surface: one #[neo_method] per op, each forwarding to `ops::*`.
// Method name == op name so the camelCase export maps cleanly back.
macro_rules! export_ops {
    ($($name:ident),* $(,)?) => {
        #[neo_contract]
        impl FuzzOps {
            pub fn new() -> Self { Self }
            $(
                #[neo_method(safe)]
                pub fn $name(a: i64, b: i64) -> i64 { ops::$name(a, b) }
            )*
        }
    };
}

export_ops!(
    add, sub, mul, div_s, rem_s, div_u, rem_u, neg, abs, and, or, xor, not, shl, shr_s, shr_u,
    rotl, rotr, clz, ctz, popcnt, eqz, eq, ne, lt_s, le_s, gt_s, ge_s, lt_u, le_u, gt_u, ge_u,
    min_s, max_s, i32_add, i32_sub, i32_mul, i32_div_s, i32_rem_s, i32_div_u, i32_rem_u, i32_shl,
    i32_shr_s, i32_shr_u, i32_rotl, i32_rotr, i32_clz, i32_ctz, i32_popcnt, wrap_i32, extend_i32_u,
    extend8_s, extend16_s, extend32_s, big_add, big_sub, big_mul, big_div, big_rem, big_and,
    big_or, big_xor, big_shl, big_shr, big_neg, not_lt, double_neg_lt, eq_cmps, ne_cmps, and_cmps,
    or_cmps, xor_cmps, cmp_chain, min_u, max_u, i32_eqz, i32_eq, i32_ne,
    nested_loop, loop_continue, multi_break, br_wide, recurse_sum, cond_nest, sx_chain, wrap_roundtrip, narrow_widen, big_not, big_div_op, big_rem_op, big_lt, big_eq, big_as_i32_sat, big_as_u32_sat, big_try_i32, big_fits, big_large_mul, big_chain,
    cond_ret, early_ret_loop, switch_ret, big_le, big_gt, big_ge, big_ne,
);

#[neo_contract]
impl FuzzOps {
    /// Evaluate the randomly-generated expression `idx` (0..GEN_COUNT) on a real
    /// NeoVM; `refgen` evaluates the same `generated_ops::eval_idx` natively.
    #[neo_method(safe)]
    pub fn gen(idx: i64, a: i64, b: i64) -> i64 {
        generated_ops::eval_idx(idx, a, b)
    }
}
