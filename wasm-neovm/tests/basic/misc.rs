// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use std::collections::HashSet;
use wasm_neovm::translate_module;

#[test]
fn translate_drop_preserves_semantics() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                i32.const 42
                drop
                i32.const 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Drop").expect("translation succeeds");
    // Keep explicit DROP to avoid backtracking truncations that can invalidate
    // pending control-flow fixup positions in complex functions.
    assert_eq!(translation.script, vec![0x00, 0x2A, 0x45, 0x11, 0x40]);
}

#[test]
fn translate_duplicate_exports_preserve_all_aliases() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "foo") (export "bar"))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MultiExport").expect("translation succeeds");
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let names: HashSet<_> = methods
        .iter()
        .map(|entry| entry["name"].as_str().unwrap())
        .collect();
    assert!(names.contains("foo"));
    assert!(names.contains("bar"));
    assert_eq!(names.len(), 2, "expected both export aliases to remain");

    let offsets: HashSet<_> = methods
        .iter()
        .map(|entry| entry["offset"].as_u64().unwrap())
        .collect();
    assert_eq!(
        offsets.len(),
        1,
        "all aliases should point at the same function offset"
    );
}

#[test]
fn translate_unreachable_emits_abort() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                unreachable)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Trap").expect("translation succeeds");
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    assert_eq!(translation.script.first().copied(), Some(abort));
}

#[test]
fn translate_reports_float_unsupported() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result f32)
                f32.const 0
                f32.const 1
                f32.add)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "FloatOp").expect_err("float should be unsupported");
    let msg = format!("{:#}", err);
    assert!(msg.contains("floating point operation"), "message: {msg}");
    assert!(
        msg.contains("docs/wasm-neovm-status.md"),
        "hint missing: {msg}"
    );
}

#[test]
fn translate_reports_simd_unsupported() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "main")
                v128.const i32x4 1 2 3 4
                drop)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "SimdOp").expect_err("simd should be unsupported");
    let msg = format!("{:#}", err);
    let lower = msg.to_lowercase();
    assert!(lower.contains("simd"), "message: {msg}");
    assert!(
        msg.contains("docs/wasm-neovm-status.md"),
        "hint missing: {msg}"
    );
}
