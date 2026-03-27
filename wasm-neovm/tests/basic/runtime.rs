// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::translate_module;

#[test]
fn translate_memory_size_uses_runtime_helper() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "size") (result i32)
                memory.size)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemSize").expect("translation succeeds");

    let guard_ldsfld = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;
    let guard_jump_l = wasm_neovm::opcodes::lookup("JMPIF_L").unwrap().byte;
    let guard_jump_s = wasm_neovm::opcodes::lookup("JMPIF").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let call_s = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    assert_eq!(translation.script[0], guard_ldsfld);
    assert!(translation.script[1] == guard_jump_l || translation.script[1] == guard_jump_s);
    assert!(translation.script.contains(&call_l) || translation.script.contains(&call_s));

    let ldsfld2 = wasm_neovm::opcodes::lookup("LDSFLD2").unwrap().byte;
    assert!(translation.script.contains(&ldsfld2));

    let manifest = translation
        .manifest
        .to_json_string()
        .expect("manifest serialises");
    assert!(manifest.contains("\"returntype\": \"Integer\""));
}

#[test]
fn translate_i32_load_uses_helper() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (export "load" (func $load))
              (func $load (result i32)
                i32.const 0
                i32.load)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemLoad").expect("translation succeeds");

    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    let guard_ldsfld = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;
    let guard_jump_l = wasm_neovm::opcodes::lookup("JMPIF_L").unwrap().byte;
    let guard_jump_s = wasm_neovm::opcodes::lookup("JMPIF").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let call_s = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let ret = wasm_neovm::opcodes::lookup("RET").unwrap().byte;

    assert_eq!(translation.script[0], push0);
    assert_eq!(translation.script[1], guard_ldsfld);
    assert!(translation.script[2] == guard_jump_l || translation.script[2] == guard_jump_s);

    let call_sites: Vec<_> = translation
        .script
        .iter()
        .enumerate()
        .filter(|(_, &byte)| byte == call_l || byte == call_s)
        .map(|(idx, _)| idx)
        .collect();
    assert!(call_sites.len() >= 2, "expected helper calls to be emitted");

    assert!(translation
        .script
        .iter()
        .any(|&b| b == wasm_neovm::opcodes::lookup("SUBSTR").unwrap().byte));

    assert!(translation.script.contains(&ret));
}

#[test]
fn translate_i32_store_uses_helper() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store")
                i32.const 0
                i32.const 0xAB
                i32.store8)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemStore").expect("translation succeeds");

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let call_s = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let setitem = wasm_neovm::opcodes::lookup("SETITEM").unwrap().byte;

    assert!(
        translation
            .script
            .iter()
            .filter(|&&b| b == call_l || b == call_s)
            .count()
            >= 2
    );

    assert!(translation.script.contains(&setitem));
}

#[test]
fn translate_memory_grow_returns_prev_or_fail() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "grow_zero") (result i32)
                i32.const 0
                memory.grow)
              (func (export "grow_fail") (result i32)
                i32.const 1
                memory.grow)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemGrow").expect("translation succeeds");

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let call_s = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;

    assert!(translation.script.contains(&call_l) || translation.script.contains(&call_s));
    assert!(translation.script.contains(&pushm1));
}

#[test]
fn translate_memory_fill_and_copy_helpers() {
    let fill_wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "fill") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.fill))"#,
    )
    .expect("valid wat");

    let fill_translation = translate_module(&fill_wasm, "MemFill").expect("translate fill");
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let call_s = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    assert!(
        fill_translation
            .script
            .iter()
            .filter(|&&b| b == call_l || b == call_s)
            .count()
            >= 2
    );

    let initslot = wasm_neovm::opcodes::lookup("INITSLOT").unwrap().byte;
    let setitem = wasm_neovm::opcodes::lookup("SETITEM").unwrap().byte;
    assert!(fill_translation.script.contains(&initslot));
    assert!(fill_translation.script.contains(&setitem));

    let copy_wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "copy") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.copy))"#,
    )
    .expect("valid wat");

    let copy_translation = translate_module(&copy_wasm, "MemCopy").expect("translate copy");
    let memcpy = wasm_neovm::opcodes::lookup("MEMCPY").unwrap().byte;
    assert!(copy_translation.script.contains(&memcpy));
    assert!(copy_translation.script.contains(&initslot));
}

#[test]
fn translate_memory_init_uses_passive_segment() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data "Hi")
              (func (export "init") (result i32)
                i32.const 0
                i32.const 0
                i32.const 2
                memory.init 0 0
                i32.const 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemInit").expect("translate memory.init");

    let pushdata1 = wasm_neovm::opcodes::lookup("PUSHDATA1").unwrap().byte;
    let memcpy = wasm_neovm::opcodes::lookup("MEMCPY").unwrap().byte;
    let stsfld4 = wasm_neovm::opcodes::lookup("STSFLD4").unwrap().byte;

    let hi_literal = b"\x02Hi";
    let has_inline_literal = translation
        .script
        .windows(hi_literal.len())
        .any(|window| window == hi_literal);

    assert!(
        translation.script.contains(&pushdata1) || has_inline_literal,
        "expected memory.init helper to embed segment literal via PUSHDATA1 or inline push; script did not contain {:?}",
        hi_literal
    );
    assert!(translation.script.contains(&memcpy));
    assert!(translation.script.contains(&stsfld4));
}

#[test]
fn translate_data_drop_emits_helper() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data "OK")
              (func (export "drop")
                data.drop 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemDrop").expect("translate data.drop");

    let stsfld5 = wasm_neovm::opcodes::lookup("STSFLD5").unwrap().byte;

    assert!(translation.script.contains(&stsfld5));
}

#[test]
fn translate_active_data_segment_initialises_memory() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data (i32.const 2) "AB")
              (func (export "load") (result i32)
                i32.const 2
                i32.load8_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ActiveData").expect("translate active data");

    let memcpy = wasm_neovm::opcodes::lookup("MEMCPY").unwrap().byte;
    let pushdata1 = wasm_neovm::opcodes::lookup("PUSHDATA1").unwrap().byte;

    assert!(translation.script.contains(&memcpy));
    let ab_literal = b"\x02AB";
    let has_inline_literal = translation
        .script
        .windows(ab_literal.len())
        .any(|window| window == ab_literal);

    assert!(
        translation.script.contains(&pushdata1) || has_inline_literal,
        "expected active segment literal to be emitted via PUSHDATA1 or inline push; script did not contain {:?}",
        ab_literal
    );
}

#[test]
fn translate_global_get_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (global $g i32 (i32.const 42))
              (func (export "main") (result i32)
                global.get $g))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GlobalConst").expect("translate global const");

    let pushint8 = wasm_neovm::opcodes::lookup("PUSHINT8").unwrap().byte;
    assert!(translation.script.starts_with(&[pushint8, 42]));
}

#[test]
fn translate_global_set_mutable() {
    let wasm = wat::parse_str(
        r#"(module
              (global $g (mut i32) (i32.const 0))
              (func (export "set") (param i32)
                local.get 0
                global.set $g)
              (func (export "get") (result i32)
                global.get $g))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GlobalMutable").expect("translate global mutable");

    let stsfld4 = wasm_neovm::opcodes::lookup("STSFLD4").unwrap().byte;
    let ldsfld4 = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;

    assert!(translation.script.contains(&stsfld4));
    assert!(translation.script.contains(&ldsfld4));
}
