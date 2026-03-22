// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: NEF serialization with arbitrary script bytes and metadata.
//! `write_nef_with_metadata()` must never panic regardless of input.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wasm_neovm::MethodToken;

#[derive(Debug, Arbitrary)]
struct FuzzNefInput<'a> {
    script: &'a [u8],
    source_url: Option<&'a str>,
    tokens: Vec<FuzzMethodToken<'a>>,
}

#[derive(Debug, Arbitrary)]
struct FuzzMethodToken<'a> {
    contract_hash: [u8; 20],
    method: &'a str,
    parameters_count: u16,
    has_return_value: bool,
    call_flags: u8,
}

fuzz_target!(|input: FuzzNefInput| {
    let tokens: Vec<MethodToken> = input
        .tokens
        .iter()
        .take(16) // cap token count to avoid OOM
        .map(|t| MethodToken {
            contract_hash: t.contract_hash,
            method: t.method.chars().take(32).collect(),
            parameters_count: t.parameters_count,
            has_return_value: t.has_return_value,
            call_flags: t.call_flags & 0x0F,
        })
        .collect();

    let dir = tempfile::tempdir().expect("tempdir");
    let nef_path = dir.path().join("fuzz.nef");

    let _ = wasm_neovm::write_nef_with_metadata(
        input.script,
        input.source_url,
        &tokens,
        &nef_path,
    );
});
