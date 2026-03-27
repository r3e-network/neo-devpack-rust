// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::translate_module;

#[test]
fn translate_syscall_import() {
    let wasm = wat::parse_str(
        r#"(module
              (import "syscall" "System.Runtime.GetTime" (func $get_time (result i64)))
              (func (export "main") (result i64)
                call $get_time)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Clock").expect("translation succeeds");

    assert_eq!(translation.script.len(), 6);
    let syscall_opcode = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert_eq!(translation.script[0], syscall_opcode); // SYSCALL
    assert_eq!(&translation.script[1..5], &[0xB7, 0xC3, 0x88, 0x03]);
    assert_eq!(translation.script[5], 0x40); // RET

    let manifest = translation
        .manifest
        .to_json_string()
        .expect("manifest serialises");
    assert!(manifest.contains("\"name\": \"Clock\""));
    assert!(manifest.contains("\"returntype\": \"Integer\""));
}

#[test]
fn translate_call_indirect_dispatches() {
    let wasm = wat::parse_str(
        r#"(module
              (type $t (func (result i32)))
              (table funcref (elem $f0 $f1))
              (func $f0 (result i32)
                i32.const 1)
              (func $f1 (result i32)
                i32.const 2)
              (func (export "main") (param i32) (result i32)
                local.get 0
                call_indirect (type $t)))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CallIndirect").expect("translate call_indirect");

    let call = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;

    let call_count = translation
        .script
        .iter()
        .filter(|&&b| b == call || b == call_l)
        .count();
    assert!(call_count >= 2);
    assert!(translation.script.contains(&abort));
}

#[test]
fn translate_opcode_immediate_and_raw() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "PUSHINT32" (func $push32 (param i32)))
              (import "opcode" "RAW" (func $raw (param i32)))
              (func (export "emit")
                i32.const 1234
                call $push32
                ;; Build a valid PUSHDATA1 instruction via RAW: [0x0C, 0x01, 0xDE]
                i32.const 12
                call $raw
                i32.const 1
                call $raw
                i32.const 222
                call $raw)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Emit").expect("translation succeeds");

    // PUSHINT32 opcode (0x02) followed by little-endian immediate, then PUSHDATA1 payload, then RET.
    assert_eq!(translation.script.len(), 9);
    assert_eq!(translation.script[0], 0x02); // PUSHINT32
    assert_eq!(&translation.script[1..5], &1234i32.to_le_bytes());
    assert_eq!(translation.script[5], 0x0C); // PUSHDATA1
    assert_eq!(translation.script[6], 1u8); // length
    assert_eq!(translation.script[7], 222u8); // payload byte
    assert_eq!(translation.script[8], 0x40); // RET
}

#[test]
fn translate_opcode_pushint128_immediate() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "PUSHINT128" (func $push128 (param i64)))
              (func (export "emit")
                i64.const 123456789
                call $push128)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Emit128").expect("translation succeeds");

    assert_eq!(translation.script.len(), 18);
    assert_eq!(translation.script[0], 0x04); // PUSHINT128
    assert_eq!(&translation.script[1..17], &123456789i128.to_le_bytes());
    assert_eq!(translation.script[17], 0x40); // RET
}

#[test]
fn translate_opcode_pushint256_immediate_sign_extends() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "PUSHINT256" (func $push256 (param i64)))
              (func (export "emit")
                i64.const -1
                call $push256)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Emit256").expect("translation succeeds");

    let pushint256 = wasm_neovm::opcodes::lookup("PUSHINT256").unwrap().byte;
    assert_eq!(translation.script.len(), 34);
    assert_eq!(translation.script[0], pushint256);
    assert!(translation.script[1..33].iter().all(|&b| b == 0xFF));
    assert_eq!(translation.script[33], 0x40); // RET
}

#[test]
fn translate_internal_function_call() {
    let wasm = wat::parse_str(
        r#"(module
              (func $helper (result i32)
                i32.const 5
                i32.const 7
                i32.add)
              (func (export "main") (result i32)
                call $helper))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Call").expect("translate call");

    let call = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;

    let call_pos = translation
        .script
        .iter()
        .position(|&b| b == call || b == call_l)
        .expect("CALL/CALL_L emitted");
    if translation.script[call_pos] == call_l {
        let immediate = &translation.script[call_pos + 1..call_pos + 5];
        assert_ne!(immediate, &[0, 0, 0, 0], "CALL_L immediate patched");
    } else {
        assert_ne!(
            translation.script[call_pos + 1],
            0,
            "CALL immediate patched"
        );
    }
    assert!(translation.script.contains(&add));
}

#[test]
fn translate_nop_is_ignored() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                nop
                i32.const 9))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Nop").expect("translate nop");

    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    let const_byte = push0.wrapping_add(9);
    assert!(translation.script.contains(&const_byte));
}

#[test]
fn translate_native_contract_syscall() {
    let wasm = wat::parse_str(
        r#"(module
              (import "syscall" "System.Contract.Call" (func $call (param i32 i32 i32) (result i32)))
              (func (export "main") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                call $call)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NativeCall").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40)); // RET present
}

#[test]
fn translate_import_reexport_generates_stub() {
    let wasm = wat::parse_str(
        r#"(module
              (import "syscall" "System.Runtime.GetTime" (func (result i64)))
              (export "get_time" (func 0))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ReExport").expect("translation succeeds");
    assert!(
        !translation.script.is_empty(),
        "stub should emit executable script bytes"
    );
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods array");
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0]["name"].as_str().unwrap(), "get_time");
}
