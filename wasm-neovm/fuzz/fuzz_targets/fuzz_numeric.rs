// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: numeric encoding via `push_biginteger()` and `push_bytevec()`.
//! Verifies these low-level encoding routines never panic on any input.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wasm_neovm::numeric::{push_biginteger, push_bytevec};

#[derive(Debug, Arbitrary)]
struct FuzzNumericInput<'a> {
    integer_value: i64,
    byte_data: &'a [u8],
}

fuzz_target!(|input: FuzzNumericInput| {
    let mut script = Vec::with_capacity(128);

    push_biginteger(&mut script, input.integer_value);
    assert!(!script.is_empty(), "push_biginteger must produce output");

    script.clear();
    push_bytevec(&mut script, input.byte_data);
    assert!(!script.is_empty(), "push_bytevec must produce output");
});
