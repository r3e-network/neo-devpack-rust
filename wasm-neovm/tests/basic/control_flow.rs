// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::translate_module;

#[test]
fn translate_block_with_branches() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "branch") (param i32)
                block
                  local.get 0
                  br_if 0
                  br 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Branch").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_loop_with_back_edge() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "loop")
                loop
                  br 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Loop").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_if_else_structure() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cond") (param i32)
                block
                  local.get 0
                  if
                    i32.const 1
                    drop
                  else
                    i32.const 2
                    drop
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Cond").expect("translation succeeds");
    let jmp_if_not_l = wasm_neovm::opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    let jmp_if_not_s = wasm_neovm::opcodes::lookup("JMPIFNOT").unwrap().byte;
    let jmp_l = wasm_neovm::opcodes::lookup("JMP_L").unwrap().byte;
    let jmp_s = wasm_neovm::opcodes::lookup("JMP").unwrap().byte;

    // Find the JMPIFNOT (long or short form) emitted by the if construct
    let jmp_if_not_pos = translation
        .script
        .iter()
        .position(|&b| b == jmp_if_not_l || b == jmp_if_not_s)
        .expect("if should emit JMPIFNOT");
    let is_long_jmpifnot = translation.script[jmp_if_not_pos] == jmp_if_not_l;

    // Find the JMP (long or short) for else branch
    let skip = if is_long_jmpifnot {
        jmp_if_not_pos + 5
    } else {
        jmp_if_not_pos + 2
    };
    let jmp_pos = translation
        .script
        .iter()
        .enumerate()
        .skip(skip)
        .find_map(|(idx, &byte)| {
            if byte == jmp_l || byte == jmp_s {
                Some(idx)
            } else {
                None
            }
        })
        .expect("then branch should emit JMP over else body");
    let is_long_jmp = translation.script[jmp_pos] == jmp_l;

    let if_not_offset = if is_long_jmpifnot {
        i32::from_le_bytes(
            translation.script[jmp_if_not_pos + 1..jmp_if_not_pos + 5]
                .try_into()
                .expect("valid JMPIFNOT_L operand"),
        )
    } else {
        translation.script[jmp_if_not_pos + 1] as i8 as i32
    };
    let if_not_target = (jmp_if_not_pos as i32 + if_not_offset) as usize;
    let jmp_instr_size = if is_long_jmp { 5 } else { 2 };
    assert_eq!(
        if_not_target,
        jmp_pos + jmp_instr_size,
        "if-false jump must land at ELSE body start"
    );
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_br_table_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dispatch") (param i32)
                block
                  block
                    local.get 0
                    br_table 1 0
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Dispatch").expect("translation succeeds");

    let dup = wasm_neovm::opcodes::lookup("DUP").unwrap().byte;
    let jmp_if_l = wasm_neovm::opcodes::lookup("JMPIF_L").unwrap().byte;
    let jmp_if_s = wasm_neovm::opcodes::lookup("JMPIF").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;

    assert!(translation.script.contains(&dup));
    assert!(translation.script.contains(&jmp_if_l) || translation.script.contains(&jmp_if_s));

    // Ensure there is at least one DROP to clear the index before branching.
    assert!(translation.script.iter().filter(|&&b| b == drop).count() >= 2);
}

#[test]
fn translate_br_table_constant_index() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dispatch_const")
                block
                  block
                    i32.const 2
                    br_table 1 0
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DispatchConst").expect("translation succeeds");

    let dup = wasm_neovm::opcodes::lookup("DUP").unwrap().byte;
    assert!(
        !translation.script.contains(&dup),
        "constant br_table should not emit DUP comparisons"
    );
}
