// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: compile the same generated Rust devpack contract twice and assert
//! deterministic Wasm, script, manifest, method token, and NEF output.

#![no_main]

use libfuzzer_sys::fuzz_target;
use wasm_neovm_fuzz::{
    assert_rust_contract_translation_parity, compile_generated_rust_contract,
    render_structured_rust_contract, translate_generated_rust_contract, RustContractCompileOutcome,
    RustContractFuzzInput,
};

fuzz_target!(|input: RustContractFuzzInput<'_>| {
    let contract = render_structured_rust_contract(&input);
    let first = compile_generated_rust_contract("fuzz_rust_contract_differential_a", &contract);
    let second = compile_generated_rust_contract("fuzz_rust_contract_differential_b", &contract);

    match (first, second) {
        (RustContractCompileOutcome::Rejected, RustContractCompileOutcome::Rejected) => {}
        (
            RustContractCompileOutcome::Compiled(first),
            RustContractCompileOutcome::Compiled(second),
        ) => {
            assert_eq!(
                first.wasm, second.wasm,
                "Rust contract compiler/devpack emitted non-deterministic Wasm"
            );

            let first_translation = translate_generated_rust_contract(&first, &contract);
            let second_translation = translate_generated_rust_contract(&second, &contract);
            assert_rust_contract_translation_parity(&first_translation, &second_translation);
        }
        _ => {
            panic!("Rust contract compiler/devpack accepted the same input inconsistently");
        }
    }
});
