// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT
//
//! Random expression-tree generator for the differential fuzzer. Emits
//! `generated_ops.rs`: `GEN_COUNT` trap-free, deterministic `i64`/`NeoInteger`
//! op compositions over inputs `a`,`b`, plus an `eval_idx` dispatcher. Both the
//! contract's `gen(idx,a,b)` export and the native `refgen` evaluate the *same*
//! generated code, so any oracle/native mismatch is a translator bug.
//!
//! Regenerate with a fresh seed each round (`FUZZ_SEED`) to explore new op
//! compositions — this is what makes the continuous differential cover the
//! composition space, not just the input space. Knobs: `FUZZ_GEN_COUNT`
//! (functions, default 80), `FUZZ_GEN_DEPTH` (max tree depth, default 8),
//! `FUZZ_GEN_NODES` (per-function node budget bounding contract size, default
//! 26). The node budget keeps the translated NEF under NeoVM's script-size
//! limit regardless of depth — without it a deep tree expands to ~2^depth
//! leaves and faults the whole contract on load.

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
    fn n(&mut self, m: u64) -> u64 {
        self.next() % m
    }
    fn konst(&mut self) -> i64 {
        // bias toward edge/small values that trigger boundary behaviour
        const E: &[i64] = &[
            0, 1, -1, 2, 7, 8, 31, 32, 63, 64, 255, 256, -256, i64::MIN, i64::MAX,
            i32::MIN as i64, i32::MAX as i64, 0xFFFF_FFFF, 1 << 31, 1 << 62,
        ];
        match self.n(3) {
            0 => E[(self.next() % E.len() as u64) as usize],
            _ => self.next() as i64,
        }
    }
}

/// Build a random trap-free i64 expression string.
///
/// `budget` bounds the total node count of the whole tree (shared, decremented
/// per node), so contract size stays predictable regardless of `depth`. Without
/// it a depth-N binary tree can expand to ~2^N leaves, blowing the translated
/// NEF past NeoVM's script-size limit and faulting the whole contract on load.
fn expr(r: &mut Rng, depth: u32, budget: &mut i32) -> String {
    *budget -= 1;
    if depth == 0 || *budget <= 0 || r.n(4) == 0 {
        return match r.n(3) {
            0 => "a".to_string(),
            1 => "b".to_string(),
            _ => format!("{}i64", r.konst()),
        };
    }
    let d = depth - 1;
    // 0..=24 binary/unary/ternary/bigint nodes
    match r.n(28) {
        0 => format!("({}).wrapping_add({})", expr(r, d, budget), expr(r, d, budget)),
        1 => format!("({}).wrapping_sub({})", expr(r, d, budget), expr(r, d, budget)),
        2 => format!("({}).wrapping_mul({})", expr(r, d, budget), expr(r, d, budget)),
        3 => format!("gdiv({}, {})", expr(r, d, budget), expr(r, d, budget)),
        4 => format!("grem({}, {})", expr(r, d, budget), expr(r, d, budget)),
        5 => format!("gdivu({}, {})", expr(r, d, budget), expr(r, d, budget)),
        6 => format!("(({}) & ({}))", expr(r, d, budget), expr(r, d, budget)),
        7 => format!("(({}) | ({}))", expr(r, d, budget), expr(r, d, budget)),
        8 => format!("(({}) ^ ({}))", expr(r, d, budget), expr(r, d, budget)),
        9 => format!("({}).wrapping_shl(({}) as u32)", expr(r, d, budget), expr(r, d, budget)),
        10 => format!("({}).wrapping_shr(({}) as u32)", expr(r, d, budget), expr(r, d, budget)),
        11 => format!("((({}) as u64).wrapping_shr(({}) as u32) as i64)", expr(r, d, budget), expr(r, d, budget)),
        12 => format!("(({}).rotate_left((({}) & 63) as u32))", expr(r, d, budget), expr(r, d, budget)),
        13 => format!("(({}).min({}))", expr(r, d, budget), expr(r, d, budget)),
        14 => format!("(({}).max({}))", expr(r, d, budget), expr(r, d, budget)),
        15 => format!("((({}) < ({})) as i64)", expr(r, d, budget), expr(r, d, budget)),
        16 => format!("((({}) == ({})) as i64)", expr(r, d, budget), expr(r, d, budget)),
        17 => format!("(((({}) as u64) < (({}) as u64)) as i64)", expr(r, d, budget), expr(r, d, budget)),
        18 => format!("(({}).wrapping_neg())", expr(r, d, budget)),
        19 => format!("(!({}))", expr(r, d, budget)),
        20 => format!("(({}).wrapping_abs())", expr(r, d, budget)),
        21 => format!("((({}) as i32) as i64)", expr(r, d, budget)),
        22 => format!("((({}) as i8) as i64)", expr(r, d, budget)),
        23 => format!("((({}) as u32) as u64 as i64)", expr(r, d, budget)),
        24 => format!("(({}).count_ones() as i64)", expr(r, d, budget)),
        25 => format!("(if ({}) != 0 {{ {} }} else {{ {} }})", expr(r, d, budget), expr(r, d, budget), expr(r, d, budget)),
        26 => format!(
            "((NeoInteger::new({}) {} NeoInteger::new({})).as_i64_saturating())",
            expr(r, d, budget),
            ["+", "-", "*", "&", "|", "^"][(r.n(6)) as usize],
            expr(r, d, budget)
        ),
        _ => format!(
            "((NeoInteger::new({}) >> ((({}) & 63) as u32)).as_i64_saturating())",
            expr(r, d, budget),
            expr(r, d, budget)
        ),
    }
}

fn main() {
    let seed = std::env::var("FUZZ_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(1u64);
    let count: u64 = std::env::var("FUZZ_GEN_COUNT").ok().and_then(|s| s.parse().ok()).unwrap_or(80);
    let depth: u32 = std::env::var("FUZZ_GEN_DEPTH").ok().and_then(|s| s.parse().ok()).unwrap_or(8);
    // Per-function node budget bounds contract size; keep count*nodes modest so
    // the translated NEF stays under NeoVM's script-size limit (depth 8 / 80
    // fns / 26 nodes ~= proven-safe; verified 0 faults on the real VM).
    let nodes: i32 = std::env::var("FUZZ_GEN_NODES").ok().and_then(|s| s.parse().ok()).unwrap_or(26);
    let mut r = Rng(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed });

    println!("// AUTO-GENERATED by gen_exprs (seed={seed}, count={count}, depth={depth}, nodes={nodes}). Do not edit.");
    println!("#![allow(clippy::all, unused_parens, unused_variables, dead_code)]");
    println!("use neo_devpack::prelude::*;");
    println!("#[inline] fn gdiv(x: i64, y: i64) -> i64 {{ x.checked_div(y).unwrap_or(0) }}");
    println!("#[inline] fn grem(x: i64, y: i64) -> i64 {{ x.checked_rem(y).unwrap_or(0) }}");
    println!("#[inline] fn gdivu(x: i64, y: i64) -> i64 {{ if y == 0 {{ 0 }} else {{ ((x as u64) / (y as u64)) as i64 }} }}");
    println!("pub const GEN_COUNT: i64 = {count};");
    for i in 0..count {
        let mut budget = nodes;
        println!("#[inline(never)] pub fn g{i}(a: i64, b: i64) -> i64 {{ {} }}", expr(&mut r, depth, &mut budget));
    }
    println!("pub fn eval_idx(idx: i64, a: i64, b: i64) -> i64 {{");
    println!("    match idx {{");
    for i in 0..count {
        println!("        {i} => g{i}(a, b),");
    }
    println!("        _ => 0,");
    println!("    }}");
    println!("}}");
}
