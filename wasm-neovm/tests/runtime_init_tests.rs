// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::{opcodes, translate_module};

#[test]
fn runtime_initialization_runs_once() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "main") (result i32)
                i32.const 0
                i32.load)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "InitGuard").expect("translation succeeds");

    let init_slot = opcodes::lookup("INITSSLOT").unwrap().byte;
    let count = translation
        .script
        .iter()
        .filter(|&&byte| byte == init_slot)
        .count();

    assert_eq!(count, 1, "expected a single INITSSLOT invocation");
}

#[test]
fn start_only_export_uses_init_stub() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func $start
                i32.const 0
                i32.const 1
                i32.store)
              (start $start)
              (export "start" (func $start))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StartOnly").expect("translation succeeds");

    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let export = methods
        .iter()
        .find(|m| m["name"].as_str() == Some("start"))
        .expect("start export present");
    let offset = export["offset"].as_u64().expect("offset present") as usize;

    let script = &translation.script;
    assert!(offset + 2 < script.len(), "export offset within script");

    let ldsfld_op = opcodes::lookup("LDSFLD").unwrap().byte;
    let ldsfld_short: Vec<u8> = (0..=6)
        .map(|i| opcodes::lookup(&format!("LDSFLD{i}")).unwrap().byte)
        .collect();

    let first = script[offset];
    let jump_pos = if first == ldsfld_op {
        offset + 2 // opcode + slot operand
    } else {
        assert!(
            ldsfld_short.contains(&first),
            "export stub should begin with LDSFLD*_ load"
        );
        offset + 1
    };

    let jmpif_l = opcodes::lookup("JMPIF_L").unwrap().byte;
    assert_eq!(
        script.get(jump_pos),
        Some(&jmpif_l),
        "stub should gate init/start on INIT_FLAG"
    );
}
