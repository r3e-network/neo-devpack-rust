// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: translate with varied configuration fields.
//! Uses `Arbitrary` to derive both WASM bytes and config parameters.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wasm_neovm::{BehaviorConfig, TranslationConfig};

#[derive(Debug, Arbitrary)]
struct FuzzInput<'a> {
    wasm: &'a [u8],
    max_memory_pages: u32,
    max_table_size: u32,
    stack_size_limit: u32,
    aggressive_optimization: bool,
    strict_validation: bool,
    allow_float: bool,
    enable_bulk_memory: bool,
}

fuzz_target!(|input: FuzzInput| {
    let behavior = BehaviorConfig {
        max_memory_pages: input.max_memory_pages.min(4096),
        max_table_size: input.max_table_size.min(50_000),
        stack_size_limit: input.stack_size_limit.min(8192),
        aggressive_optimization: input.aggressive_optimization,
        strict_validation: input.strict_validation,
        allow_float: input.allow_float,
        enable_bulk_memory: input.enable_bulk_memory,
        ..BehaviorConfig::default()
    };

    let config = TranslationConfig::new("FuzzConfigContract").with_behavior(behavior);

    let _ = wasm_neovm::translate_with_config(input.wasm, config);
});
