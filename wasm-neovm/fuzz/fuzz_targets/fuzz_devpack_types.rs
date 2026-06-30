// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: property-check the neo-devpack value TYPES against independent
//! references — NeoInteger arithmetic/bitwise/conversions vs i128/std, and
//! NeoByteString/NeoArray operations vs Vec. Complements `fuzz_devpack_codec`
//! (which covers serde roundtrips) by checking the *semantics* of the types
//! the SDK exposes to contract authors.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use neo_devpack::{NeoArray, NeoByteString, NeoInteger};

#[derive(Debug, Arbitrary)]
struct TypesInput {
    a: i64,
    b: i64,
    shift: u8,
    bytes: Vec<u8>,
    push_byte: u8,
}

fn sat_i64(v: i128) -> i64 {
    v.clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

fuzz_target!(|input: TypesInput| {
    let TypesInput {
        a,
        b,
        shift,
        bytes,
        push_byte,
    } = input;
    let na = NeoInteger::new(a);
    let nb = NeoInteger::new(b);
    let red = |n: NeoInteger| n.as_i64_saturating();

    // ---- arithmetic vs an i128 reference (NeoInteger is arbitrary-precision,
    // then saturates back to i64) ----
    assert_eq!(red(na.clone() + nb.clone()), sat_i64(a as i128 + b as i128), "add {a} {b}");
    assert_eq!(red(na.clone() - nb.clone()), sat_i64(a as i128 - b as i128), "sub {a} {b}");
    assert_eq!(red(na.clone() * nb.clone()), sat_i64(a as i128 * b as i128), "mul {a} {b}");
    assert_eq!(red(NeoInteger::zero() - na.clone()), sat_i64(-(a as i128)), "neg {a}");

    if b != 0 {
        // try_div / try_rem use truncating BigInt division, like i128.
        assert_eq!(
            na.try_div(&nb).map(red).unwrap_or(i64::MIN),
            sat_i64(a as i128 / b as i128),
            "div {a} {b}"
        );
        assert_eq!(
            na.try_rem(&nb).map(red).unwrap_or(i64::MIN),
            sat_i64(a as i128 % b as i128),
            "rem {a} {b}"
        );
    }

    // ---- bitwise: exact in i64 (two's complement) ----
    assert_eq!(red(na.clone() & nb.clone()), a & b, "and {a} {b}");
    assert_eq!(red(na.clone() | nb.clone()), a | b, "or {a} {b}");
    assert_eq!(red(na.clone() ^ nb.clone()), a ^ b, "xor {a} {b}");
    assert_eq!(red(!na.clone()), !a, "not {a}");

    // ---- shifts (bounded count) ----
    let s = (shift % 63) as u32;
    assert_eq!(red(na.clone() << s), sat_i64((a as i128) << s), "shl {a} {s}");
    // BigInt >> is an arithmetic (floor) shift, like i64 >>.
    assert_eq!(red(na.clone() >> s), a >> s, "shr {a} {s}");

    // ---- conversions vs std ----
    assert_eq!(na.try_as_i32(), i32::try_from(a).ok(), "try_as_i32 {a}");
    assert_eq!(na.try_as_u32(), u32::try_from(a).ok(), "try_as_u32 {a}");
    let exp_i32_sat = a.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    assert_eq!(na.as_i32_saturating(), exp_i32_sat, "as_i32_saturating {a}");
    let exp_u32_sat = a.clamp(0, u32::MAX as i64) as u32;
    assert_eq!(na.as_u32_saturating(), exp_u32_sat, "as_u32_saturating {a}");
    // i64 values always fit NeoVM's 256-bit integer bound.
    assert!(na.fits_in_neovm(), "fits_in_neovm {a}");

    // ---- NeoByteString vs Vec ----
    let mut bs = NeoByteString::from_slice(&bytes);
    assert_eq!(bs.as_slice(), bytes.as_slice(), "bs from_slice");
    assert_eq!(bs.len(), bytes.len());
    assert_eq!(bs.is_empty(), bytes.is_empty());
    let mut vref = bytes.clone();
    bs.push(push_byte);
    vref.push(push_byte);
    assert_eq!(bs.as_slice(), vref.as_slice(), "bs push");
    bs.extend_from_slice(&bytes);
    vref.extend_from_slice(&bytes);
    assert_eq!(bs.as_slice(), vref.as_slice(), "bs extend");

    // ---- NeoArray vs Vec ----
    let mut arr: NeoArray<i64> = NeoArray::new();
    let mut aref: Vec<i64> = Vec::new();
    for (i, &byte) in bytes.iter().enumerate().take(64) {
        let v = (byte as i64).wrapping_mul(i as i64 + 1);
        arr.push(v);
        aref.push(v);
    }
    assert_eq!(arr.len(), aref.len(), "arr len");
    for i in 0..aref.len() {
        assert_eq!(arr.get(i).copied(), Some(aref[i]), "arr get {i}");
    }
    assert_eq!(arr.pop(), aref.pop(), "arr pop");
    assert_eq!(arr.len(), aref.len(), "arr len after pop");
});
