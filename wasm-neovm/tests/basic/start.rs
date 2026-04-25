// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use std::convert::TryInto;
use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_emits_start_call() {
    let wasm = wat::parse_str(
        r#"(module
              (func $start (export "start")
                i32.const 0
                drop)
              (func (export "main") (result i32)
                i32.const 7)
              (start $start))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Starty").expect("translation succeeds");

    let methods = translation
        .manifest
        .value
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(|methods| methods.as_array())
        .expect("manifest methods present");

    let start_method = methods
        .iter()
        .find(|method| method.get("name").and_then(|v| v.as_str()) == Some("start"))
        .expect("start method present");
    let start_offset = start_method
        .get("offset")
        .and_then(|offset| offset.as_u64())
        .expect("start offset present") as isize;

    let call_l_byte = opcodes::lookup("CALL_L")
        .expect("CALL_L opcode available")
        .byte;
    let call_s_byte = opcodes::lookup("CALL").expect("CALL opcode available").byte;
    assert!(
        translation.script[start_offset as usize] == call_l_byte
            || translation.script[start_offset as usize] == call_s_byte,
        "exported start method should be an init stub"
    );

    let mut found_call = false;
    let script = &translation.script;
    let mut i = 0usize;
    while i < script.len() {
        if script[i] == call_l_byte && i + 4 < script.len() {
            let delta = i32::from_le_bytes(script[i + 1..i + 5].try_into().unwrap());
            let target = i as isize + delta as isize;
            if target == 0 {
                found_call = true;
                break;
            }
            i += 5;
        } else if script[i] == call_s_byte && i + 1 < script.len() {
            let delta = script[i + 1] as i8 as isize;
            let target = i as isize + delta;
            if target == 0 {
                found_call = true;
                break;
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    assert!(
        found_call,
        "expected runtime init helper to call the original start body"
    );
}

#[test]
fn translate_rejects_start_with_result() {
    let wasm = wat::parse_str(
        r#"(module
              (func $start (result i32)
                i32.const 0)
              (func (export "main") (result i32)
                i32.const 1)
              (start $start))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "HasResult").expect_err("translation should fail");
    let message = err.to_string();
    assert!(
        message.contains("start function must not return values"),
        "unexpected start-function error message: {message}"
    );
}

#[test]
fn translate_calls_imported_start_opcode() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "NOP" (func $start))
              (func (export "main") (result i32)
                i32.const 2)
              (start $start))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ImportStart").expect("translation succeeds");
    let nop = opcodes::lookup("NOP").expect("NOP opcode available").byte;
    assert!(
        translation.script.contains(&nop),
        "expected emitted script to contain imported NOP start call"
    );
}
