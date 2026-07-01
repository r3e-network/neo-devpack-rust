// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT
//
//! Random expression-tree generator for the differential fuzzer. Emits
//! `generated_ops.rs`: `GEN_COUNT` trap-free, deterministic `i64`/`NeoInteger`
//! op compositions over inputs `a`,`b`, plus an `eval_idx` dispatcher. Both the
//! contract's `gen(idx,a,b)` export and the native `refgen` evaluate the *same*
//! generated code, so any oracle/native mismatch is a translator bug.
//!
//! Coverage spans arithmetic / bitwise / shift / rotate / compare / multi-width
//! casts / select, plus three control-flow + bignum surfaces that exercise the
//! riskiest parts of the translator:
//!   * `let` bindings — local-slot allocation and reuse,
//!   * bounded `while` loops — loop + branch lowering and local mutation,
//!   * fixed arrays with a dynamic index — linear-memory store/load lowering,
//!   * `NeoInteger` +-*/%&|^ >> — arbitrary-precision (DIV/MOD/etc.) lowering.
//! Divisors are forced non-zero (`| 1`) and loops are bounded (`& 7`) so every
//! generated program is trap-free and terminates.
//!
//! Regenerate with a fresh seed each round (`FUZZ_SEED`) to explore new op
//! compositions — this is what makes the continuous differential cover the
//! composition space, not just the input space. Knobs: `FUZZ_GEN_COUNT`
//! (functions, default 80), `FUZZ_GEN_DEPTH` (max tree depth, default 8),
//! `FUZZ_GEN_NODES` (per-function node budget bounding contract size, default
//! 26). The node budget keeps the translated NEF under NeoVM's script-size
//! limit regardless of depth — without it a deep tree expands to ~2^depth
//! leaves and faults the whole contract on load.

struct Rng {
    s: u64,
    uid: u32,
}
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.s;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.s = x;
        x
    }
    fn n(&mut self, m: u64) -> u64 {
        self.next() % m
    }
    /// Monotonic id for unique local-variable names (avoids nested-scope clashes).
    fn uid(&mut self) -> u32 {
        self.uid += 1;
        self.uid
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

/// A terminal: an in-scope variable (always includes `a`,`b`) or a constant.
fn leaf(r: &mut Rng, vars: &[String]) -> String {
    match r.n(2) {
        0 => vars[(r.next() as usize) % vars.len()].clone(),
        _ => format!("{}i64", r.konst()),
    }
}

/// Build a random trap-free i64 expression string over the in-scope `vars`.
///
/// `budget` bounds the total node count of the whole tree (shared, decremented
/// per node), so contract size stays predictable regardless of `depth`. Without
/// it a depth-N binary tree can expand to ~2^N leaves, blowing the translated
/// NEF past NeoVM's script-size limit and faulting the whole contract on load.
fn expr(r: &mut Rng, depth: u32, budget: &mut i32, vars: &[String]) -> String {
    *budget -= 1;
    if depth == 0 || *budget <= 0 || r.n(4) == 0 {
        return leaf(r, vars);
    }
    let d = depth - 1;
    macro_rules! e {
        () => {
            expr(r, d, budget, vars)
        };
    }
    match r.n(39) {
        0 => format!("({}).wrapping_add({})", e!(), e!()),
        1 => format!("({}).wrapping_sub({})", e!(), e!()),
        2 => format!("({}).wrapping_mul({})", e!(), e!()),
        3 => format!("gdiv({}, {})", e!(), e!()),
        4 => format!("grem({}, {})", e!(), e!()),
        5 => format!("gdivu({}, {})", e!(), e!()),
        6 => format!("(({}) & ({}))", e!(), e!()),
        7 => format!("(({}) | ({}))", e!(), e!()),
        8 => format!("(({}) ^ ({}))", e!(), e!()),
        9 => format!("({}).wrapping_shl(({}) as u32)", e!(), e!()),
        10 => format!("({}).wrapping_shr(({}) as u32)", e!(), e!()),
        11 => format!("((({}) as u64).wrapping_shr(({}) as u32) as i64)", e!(), e!()),
        12 => format!("(({}).rotate_left((({}) & 63) as u32))", e!(), e!()),
        13 => format!("(({}).min({}))", e!(), e!()),
        14 => format!("(({}).max({}))", e!(), e!()),
        15 => format!("((({}) < ({})) as i64)", e!(), e!()),
        16 => format!("((({}) == ({})) as i64)", e!(), e!()),
        17 => format!("(((({}) as u64) < (({}) as u64)) as i64)", e!(), e!()),
        18 => format!("(({}).wrapping_neg())", e!()),
        19 => format!("(!({}))", e!()),
        20 => format!("(({}).wrapping_abs())", e!()),
        21 => format!("((({}) as i32) as i64)", e!()),
        22 => format!("((({}) as i8) as i64)", e!()),
        23 => format!("((({}) as u32) as u64 as i64)", e!()),
        24 => format!("(({}).count_ones() as i64)", e!()),
        25 => format!("(if ({}) != 0 {{ {} }} else {{ {} }})", e!(), e!(), e!()),
        26 => format!(
            "((NeoInteger::new({}) {} NeoInteger::new({})).as_i64_saturating())",
            e!(),
            ["+", "-", "*", "&", "|", "^"][(r.n(6)) as usize],
            e!()
        ),
        27 => format!(
            "((NeoInteger::new({}) >> ((({}) & 63) as u32)).as_i64_saturating())",
            e!(),
            e!()
        ),
        // NeoInteger DIV / MOD — arbitrary-precision; divisor forced non-zero.
        28 => format!(
            "((NeoInteger::new({}) / NeoInteger::new((({}) | 1))).as_i64_saturating())",
            e!(),
            e!()
        ),
        29 => format!(
            "((NeoInteger::new({}) % NeoInteger::new((({}) | 1))).as_i64_saturating())",
            e!(),
            e!()
        ),
        // Extra cast widths — sign/zero extension lowering.
        30 => {
            let x = e!();
            match r.n(4) {
                0 => format!("((({}) as i16) as i64)", x),
                1 => format!("((({}) as u16) as u64 as i64)", x),
                2 => format!("((({}) as u8) as u64 as i64)", x),
                _ => format!("((({}) as u64) as i64)", x),
            }
        }
        // let-binding — exercises local-slot allocation + reuse.
        31 => {
            let val = e!();
            let name = format!("t{}", r.uid());
            let mut v2 = vars.to_vec();
            v2.push(name.clone());
            let body = expr(r, d, budget, &v2);
            format!("{{ let {name} = ({val}); {body} }}")
        }
        // bounded while-loop — exercises loop + branch lowering + local mutation.
        32 => {
            let init = e!();
            let bound = e!();
            let acc = format!("acc{}", r.uid());
            let i = format!("i{}", r.uid());
            let n = format!("n{}", r.uid());
            let mut v2 = vars.to_vec();
            v2.push(acc.clone());
            let step = expr(r, d, budget, &v2);
            format!(
                "{{ let mut {acc} = ({init}); let {n} = ((({bound}) & 7)); let mut {i} = 0i64; \
                 while {i} < {n} {{ {acc} = ({acc}).wrapping_add(({step})); {i} = {i} + 1; }} {acc} }}"
            )
        }
        // i32 rotate with a DYNAMIC (variable) count — emit_rotate_dynamic, 32-bit.
        33 => format!("(((({}) as i32).rotate_right((({}) & 31) as u32)) as i64)", e!(), e!()),
        // u32 logical shift-right then sign-extend i32->i64 (logical zero-mask +
        // sign_extend_via_helper(32,32) canonicalization).
        34 => format!("(((({}) as u32).wrapping_shr(({}) as u32) as i32) as i64)", e!(), e!()),
        // NeoInteger unary NOT composed in-tree (two's-complement NOT on a value
        // that may exceed i64 before saturation).
        35 => format!("((!NeoInteger::new({})).as_i64_saturating())", e!()),
        // comparison-driven select over full-width i64 operands.
        36 => format!("(if (({}) < ({})) {{ ({}) }} else {{ ({}) }})", e!(), e!(), e!(), e!()),
        // array build + DYNAMIC index — forces linear-memory stores + a runtime
        // load (the dynamic index defeats SSA promotion, so the array is
        // materialised in memory). Exercises i64 store/load lowering.
        37 => format!(
            "{{ let arr: [i64; 4] = [{}, {}, {}, {}]; arr[((({}) & 3) as usize)] }}",
            e!(), e!(), e!(), e!(), e!()
        ),
        // array build + loop-sum — 4 stores + 4 runtime-indexed loads through a
        // bounded loop (more memory traffic + branch interaction).
        _ => format!(
            "{{ let ma: [i64; 4] = [{}, {}, {}, {}]; let mut ms = 0i64; let mut mi = 0usize; \
             while mi < 4 {{ ms = ms.wrapping_add(ma[mi]); mi += 1; }} ms }}",
            e!(), e!(), e!(), e!()
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
    let mut r = Rng {
        s: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        uid: 0,
    };

    println!("// AUTO-GENERATED by gen_exprs (seed={seed}, count={count}, depth={depth}, nodes={nodes}). Do not edit.");
    println!("#![allow(clippy::all, unused_parens, unused_variables, unused_mut, dead_code)]");
    println!("use neo_devpack::prelude::*;");
    println!("#[inline] fn gdiv(x: i64, y: i64) -> i64 {{ x.checked_div(y).unwrap_or(0) }}");
    println!("#[inline] fn grem(x: i64, y: i64) -> i64 {{ x.checked_rem(y).unwrap_or(0) }}");
    println!("#[inline] fn gdivu(x: i64, y: i64) -> i64 {{ if y == 0 {{ 0 }} else {{ ((x as u64) / (y as u64)) as i64 }} }}");
    println!("pub const GEN_COUNT: i64 = {count};");
    let base = ["a".to_string(), "b".to_string()];
    for i in 0..count {
        let mut budget = nodes;
        println!(
            "#[inline(never)] pub fn g{i}(a: i64, b: i64) -> i64 {{ {} }}",
            expr(&mut r, depth, &mut budget, &base)
        );
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
