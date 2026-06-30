// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT
//
//! Native ground-truth generator for the differential fuzzer. Runs each
//! `fuzz_ops::ops` function over an edge-case + seeded-random input set and
//! prints one JSONL record per (op, a, b) with the native (Rust) result. A
//! companion Python runner feeds the identical (op, a, b) to the NeoVM oracle
//! and flags any mismatch as a translator bug.

use fuzz_ops::{eval, OP_NAMES};

fn edges() -> Vec<i64> {
    vec![
        0,
        1,
        -1,
        2,
        -2,
        3,
        7,
        -7,
        8,
        255,
        256,
        -256,
        63,
        64,
        65,
        31,
        32,
        33,
        100,
        -100,
        1 << 31,
        1 << 32,
        1 << 62,
        -(1 << 62),
        0xFFFF_FFFF,
        0x1_0000_0000,
        0x7FFF_FFFF,
        -0x8000_0000,
        i32::MAX as i64,
        i32::MIN as i64,
        i64::MAX,
        i64::MIN,
        i64::MAX - 1,
        i64::MIN + 1,
        -0x0123_4567_89AB_CDEF,
        0x0123_4567_89AB_CDEF,
    ]
}

// Deterministic xorshift64 so runs are reproducible.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn i64(&mut self) -> i64 {
        self.next() as i64
    }
    // small magnitudes are common bug triggers (shift counts, small divisors)
    fn small(&mut self) -> i64 {
        (self.next() % 130) as i64 - 65
    }
}

fn pairs(seed: u64, extra_random: usize) -> Vec<(i64, i64)> {
    let e = edges();
    let mut out: Vec<(i64, i64)> = Vec::new();
    // structured: every edge paired with a few rotated edges (always present —
    // edge x edge combinations are the highest-yield regression cases).
    for (i, &a) in e.iter().enumerate() {
        for &k in &[1usize, 7, 13, 19] {
            let b = e[(i + k) % e.len()];
            out.push((a, b));
        }
    }
    // edge-vs-small (good for shifts / divisors / rotates)
    let mut rng = Rng(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed });
    for &a in &e {
        out.push((a, rng.small()));
        out.push((rng.small(), a));
    }
    // seeded random full-width pairs (+ extra for continuous fuzzing)
    for _ in 0..(120 + extra_random * 2 / 3) {
        out.push((rng.i64(), rng.i64()));
    }
    // random with small second operand (shift/div/rem stress)
    for _ in 0..(60 + extra_random / 3) {
        out.push((rng.i64(), rng.small()));
    }
    out
}

fn main() {
    // Continuous-fuzz knobs: FUZZ_SEED varies the random batch each run,
    // FUZZ_RANDOM adds that many extra random pairs per op. Defaults reproduce
    // the deterministic regression set.
    let seed = std::env::var("FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x9E37_79B9_7F4A_7C15u64);
    let extra = std::env::var("FUZZ_RANDOM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0usize);
    let pairs = pairs(seed, extra);
    for &op in OP_NAMES {
        for &(a, b) in &pairs {
            if let Some(expected) = eval(op, a, b) {
                // JSONL: op, a, b, expected (all decimal integers)
                println!("{{\"op\":\"{op}\",\"a\":{a},\"b\":{b},\"e\":{expected}}}");
            }
        }
    }
}
