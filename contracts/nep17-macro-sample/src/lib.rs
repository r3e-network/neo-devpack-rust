// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Integration test for the `nep17!` standard-library macro (L5).
//!
//! This contract exists purely to exercise the macro: if it
//! compiles, builds to wasm32, and the emitted NEF is well-formed,
//! the macro expansion is correct. The actual transfer/balanceOf
//! logic returns the macro defaults (balance = 0, transfer = false)
//! because the test target is the macro, not a real token.

use neo_devpack::nep17;

nep17! {
    contract MacroNep17Sample {
        symbol: "MCR",
        decimals: 8,
        total_supply: 1_000_000_000_000_000,
        // No additional fields.
    }
}

// A trivial no-op method so the wasm module has a code section
// (the translator requires one). The actual NEP-17 surface comes
// from the macro; this is just to satisfy the wasm shape.
#[no_mangle]
pub extern "C" fn _init() {}

