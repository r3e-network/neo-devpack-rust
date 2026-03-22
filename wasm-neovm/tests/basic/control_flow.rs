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
    let jmp_if_not = wasm_neovm::opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    let jmp = wasm_neovm::opcodes::lookup("JMP_L").unwrap().byte;

    let jmp_if_not_pos = translation
        .script
        .iter()
        .position(|&b| b == jmp_if_not)
        .expect("if should emit JMPIFNOT_L");
    let jmp_pos = translation
        .script
        .iter()
        .enumerate()
        .skip(jmp_if_not_pos + 1)
        .find_map(|(idx, &byte)| if byte == jmp { Some(idx) } else { None })
        .expect("then branch should emit JMP_L over else body");

    let if_not_offset = i32::from_le_bytes(
        translation.script[jmp_if_not_pos + 1..jmp_if_not_pos + 5]
            .try_into()
            .expect("valid JMPIFNOT_L operand"),
    );
    let if_not_target = (jmp_if_not_pos as i32 + if_not_offset) as usize;
    assert_eq!(
        if_not_target,
        jmp_pos + 5,
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
    let jmp_if = wasm_neovm::opcodes::lookup("JMPIF_L").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;

    assert!(translation.script.contains(&dup));
    assert!(translation.script.contains(&jmp_if));

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
