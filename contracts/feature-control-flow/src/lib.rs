// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: control-flow lowering (block/loop/if/else,
//! br/br_if, br_table), function calls (direct + call_indirect via fn
//! pointers), recursion, early return, and select/drop. Heap-free.

use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureControlFlow" }"#);

#[neo_contract]
pub struct ControlFlowContract;

#[inline(never)]
fn add_a(a: i64, b: i64) -> i64 {
    let _ = b;
    a + 1
}

#[inline(never)]
fn add_b(a: i64, b: i64) -> i64 {
    let _ = a;
    b + 2
}

#[inline(never)]
fn fib_rec(n: i64) -> i64 {
    if n < 2 {
        n
    } else {
        fib_rec(n - 1).wrapping_add(fib_rec(n - 2))
    }
}

#[neo_contract]
impl ControlFlowContract {
    pub fn new() -> Self {
        Self
    }

    /// Nested for/while loops with `if`, `continue`, and an early `return`
    /// (block / loop / br / br_if lowering).
    #[neo_method(safe)]
    pub fn run(n: i64) -> i64 {
        let mut acc: i64 = 0;
        let mut i: i64 = 0;
        while i < n {
            i += 1;
            if i % 3 == 0 {
                continue;
            }
            for j in 0..i {
                acc = acc.wrapping_add(j);
                if acc > 1_000_000 {
                    return acc; // early return out of nested loops
                }
            }
        }
        acc
    }

    /// Dense `match` -> br_table jump table (kept well under 256 arms).
    #[neo_method(safe)]
    pub fn sel(k: i64) -> i64 {
        match k {
            0 => 100,
            1 => 101,
            2 => 102,
            3 => 103,
            4 => 104,
            5 => 105,
            6 => 106,
            7 => 107,
            _ => -1,
        }
    }

    /// Recursion / repeated direct calls.
    #[neo_method(safe)]
    pub fn fib(n: i64) -> i64 {
        if n < 0 || n > 90 {
            return -1;
        }
        fib_rec(n)
    }

    /// Direct function call with two args (verifies argument order).
    #[neo_method(safe)]
    pub fn direct(x: i64) -> i64 {
        add_a(x, x + 100)
    }

    /// Indirect call through a function pointer (call_indirect / funcref).
    #[neo_method(safe)]
    pub fn indirect(sel: i64, x: i64) -> i64 {
        let f: fn(i64, i64) -> i64 = if sel == 0 { add_a } else { add_b };
        f(x, x)
    }

    /// `select` / `drop` / local.get/set/tee via a branchless pick.
    #[neo_method(safe)]
    pub fn pick(c: bool, a: i64, b: i64) -> i64 {
        let mut r = if c { a } else { b };
        r += 1;
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_flow_methods() {
        assert_eq!(ControlFlowContract::sel(3), 103);
        assert_eq!(ControlFlowContract::sel(99), -1);
        assert_eq!(ControlFlowContract::fib(10), 55);
        assert_eq!(ControlFlowContract::direct(5), 6);
        assert_eq!(ControlFlowContract::indirect(0, 5), 6);
        assert_eq!(ControlFlowContract::indirect(1, 5), 7);
        assert_eq!(ControlFlowContract::pick(true, 10, 20), 11);
    }
}
