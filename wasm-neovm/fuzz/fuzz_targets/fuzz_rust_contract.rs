// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: generate valid-ish Rust neo-devpack contracts, compile them to Wasm,
//! then translate them through the Neo pipeline and assert manifest/NEF invariants.

#![no_main]

use libfuzzer_sys::fuzz_target;
use wasm_neovm_fuzz::{
    compile_generated_rust_contract, render_structured_rust_contract,
    translate_generated_rust_contract, RustContractCompileOutcome, RustContractFuzzInput,
};

fuzz_target!(|input: RustContractFuzzInput<'_>| {
    let contract = render_structured_rust_contract(&input);
    if let RustContractCompileOutcome::Compiled(compiled) =
        compile_generated_rust_contract("fuzz_rust_contract", &contract)
    {
        let _ = translate_generated_rust_contract(&compiled, &contract);
    }
});
