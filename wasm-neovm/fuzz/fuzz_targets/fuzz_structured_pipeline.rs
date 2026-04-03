// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: exercise developer-facing translation paths with structured, valid-ish modules.
//! This complements raw-byte fuzzing by covering:
//! - WAT/WASM parsing for realistic contract shapes
//! - chain-specific import lowering
//! - memory/table helpers
//! - manifest + NEF metadata invariants after successful translation

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wasm_neovm::api::TranslationBuilder;
use wasm_neovm::{BehaviorConfig, SourceChain};
use wasm_neovm_fuzz::{
    assert_translation_invariants, choose_chain, sanitize_contract_name, sanitize_source_url,
    sanitize_symbol,
};

#[derive(Debug, Arbitrary)]
struct StructuredPipelineInput<'a> {
    template: u8,
    chain: u8,
    contract_name: &'a str,
    export_name: &'a str,
    helper_name: &'a str,
    import_name: &'a str,
    source_url: Option<&'a str>,
    lhs: i32,
    rhs: i32,
    byte_value: u8,
    len: u8,
    strict_validation: bool,
    aggressive_optimization: bool,
    enable_bulk_memory: bool,
    allow_float: bool,
    prefer_known_import: bool,
}

fuzz_target!(|input: StructuredPipelineInput<'_>| {
    let chain = choose_chain(input.chain);
    let contract_name = sanitize_contract_name(input.contract_name);
    let export_name = sanitize_symbol(input.export_name, "main");
    let helper_name = sanitize_symbol(input.helper_name, "helper");

    let wasm = build_structured_module(&input, chain, &export_name, &helper_name);

    let behavior = BehaviorConfig {
        max_memory_pages: 1 + u32::from(input.len % 8),
        max_table_size: 4 + u32::from(input.len % 16),
        stack_size_limit: 64 + u32::from(input.byte_value),
        aggressive_optimization: input.aggressive_optimization,
        strict_validation: input.strict_validation,
        allow_float: input.allow_float,
        enable_bulk_memory: input.enable_bulk_memory,
        ..BehaviorConfig::default()
    };

    let mut builder = TranslationBuilder::new(contract_name)
        .with_wasm(wasm)
        .from_chain(chain)
        .with_behavior(behavior);

    if let Some(source_url) = sanitize_source_url(input.source_url) {
        builder = builder.with_source_url(source_url);
    }

    if let Ok((translation, stats)) = builder.translate_with_stats() {
        assert!(
            stats.translation_time_ms.is_some(),
            "translate_with_stats must populate timing information"
        );
        assert!(
            stats.export_count >= 1,
            "structured pipeline modules always export at least one entry point"
        );
        assert_translation_invariants(&translation);
    }
});

fn build_structured_module(
    input: &StructuredPipelineInput<'_>,
    chain: SourceChain,
    export_name: &str,
    helper_name: &str,
) -> Vec<u8> {
    let wat = match input.template % 6 {
        0 => arithmetic_module(export_name, input.lhs, input.rhs),
        1 => descriptor_syscall_module(export_name),
        2 => chain_import_module(
            chain,
            export_name,
            input.import_name,
            input.prefer_known_import,
            input.lhs,
            input.rhs,
        ),
        3 => env_memory_module(export_name, input.byte_value, input.len),
        4 => table_dispatch_module(export_name, helper_name, input.lhs, input.rhs),
        _ => bulk_memory_module(export_name, input.byte_value, input.len),
    };

    wat::parse_str(&wat).expect("structured fuzz templates must always produce valid WAT")
}

fn arithmetic_module(export_name: &str, lhs: i32, rhs: i32) -> String {
    format!(
        r#"(module
            (func (export "{export_name}") (result i32)
                i32.const {lhs}
                i32.const {rhs}
                i32.xor
                i32.const 7
                i32.add
            )
        )"#
    )
}

fn descriptor_syscall_module(export_name: &str) -> String {
    format!(
        r#"(module
            (import "syscall" "System.Runtime.GetTime" (func $get_time (result i64)))
            (func (export "{export_name}") (result i64)
                call $get_time
            )
        )"#
    )
}

fn chain_import_module(
    chain: SourceChain,
    export_name: &str,
    raw_import_name: &str,
    prefer_known_import: bool,
    lhs: i32,
    rhs: i32,
) -> String {
    match chain {
        SourceChain::Neo => {
            let import_name = if prefer_known_import {
                "get_time".to_string()
            } else {
                sanitize_symbol(raw_import_name, "get_time")
            };
            format!(
                r#"(module
                    (import "neo" "{import_name}" (func $host (result i64)))
                    (func (export "{export_name}") (result i64)
                        call $host
                    )
                )"#
            )
        }
        SourceChain::Solana => {
            let import_name = if prefer_known_import {
                "sol_log".to_string()
            } else {
                sanitize_symbol(raw_import_name, "sol_log")
            };
            format!(
                r#"(module
                    (import "solana" "{import_name}" (func $host (param i32 i32)))
                    (func (export "{export_name}") (result i32)
                        i32.const {lhs}
                        i32.const {rhs}
                        call $host
                        i32.const 0
                    )
                )"#
            )
        }
        SourceChain::Move => {
            let import_name = if prefer_known_import {
                "debug_print".to_string()
            } else {
                sanitize_symbol(raw_import_name, "debug_print")
            };
            format!(
                r#"(module
                    (import "move_stdlib" "{import_name}" (func $host (param i32 i32)))
                    (func (export "{export_name}") (result i32)
                        i32.const {lhs}
                        i32.const {rhs}
                        call $host
                        i32.const 1
                    )
                )"#
            )
        }
    }
}

fn env_memory_module(export_name: &str, byte_value: u8, len: u8) -> String {
    let (import_name, args) = match byte_value % 3 {
        0 => ("memset", format!("i32.const 0\n                        i32.const {byte_value}\n                        i32.const {}", bounded_len(len))),
        1 => ("memcpy", format!("i32.const 0\n                        i32.const 16\n                        i32.const {}", bounded_len(len))),
        _ => ("memmove", format!("i32.const 0\n                        i32.const 16\n                        i32.const {}", bounded_len(len))),
    };

    format!(
        r#"(module
            (import "env" "{import_name}" (func $mem (param i32 i32 i32) (result i32)))
            (memory 1)
            (data (i32.const 16) "seed-seed-seed-seed")
            (func (export "{export_name}") (result i32)
                {args}
                call $mem
                drop
                i32.const 0
                i32.load
            )
        )"#
    )
}

fn table_dispatch_module(export_name: &str, helper_name: &str, lhs: i32, rhs: i32) -> String {
    format!(
        r#"(module
            (type $dispatch (func (result i32)))
            (func ${helper_name}_a (type $dispatch) (result i32)
                i32.const {lhs}
            )
            (func ${helper_name}_b (type $dispatch) (result i32)
                i32.const {rhs}
            )
            (table 2 funcref)
            (elem (i32.const 0) ${helper_name}_a ${helper_name}_b)
            (func (export "{export_name}") (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.and
                call_indirect (type $dispatch)
            )
        )"#
    )
}

fn bulk_memory_module(export_name: &str, byte_value: u8, len: u8) -> String {
    format!(
        r#"(module
            (memory 1)
            (func (export "{export_name}") (result i32)
                i32.const 0
                i32.const {byte_value}
                i32.const {}
                memory.fill
                i32.const 0
                i32.load
            )
        )"#,
        bounded_len(len)
    )
}

fn bounded_len(len: u8) -> u8 {
    1 + (len % 32)
}
