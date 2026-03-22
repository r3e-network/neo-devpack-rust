// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: feed arbitrary bytes to `translate_module()`.
//! The translator must never panic on any input — only `Err(...)` is acceptable.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Silently discard — we only care about panics, not errors.
    let _ = wasm_neovm::translate_module(data, "FuzzContract");
});
