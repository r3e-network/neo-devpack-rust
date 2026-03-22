// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Block control flow tests

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_deeply_nested_blocks() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "nested") (param i32) (result i32)
                (local i32)
                block $outer
                  local.get 0
                  i32.const 0
                  i32.eq
                  br_if $outer
                  block $mid1
                    local.get 0
                    i32.const 1
                    i32.eq
                    br_if $mid1
                    block $mid2
                      local.get 0
                      i32.const 2
                      i32.eq
                      br_if $mid2
                      block $inner
                        local.get 0
                        i32.const 3
                        i32.eq
                        br_if $inner
                        i32.const 100
                        local.set 1
                      end
                      local.get 1
                      i32.const 3
                      i32.add
                      local.set 1
                    end
                    local.get 1
                    i32.const 2
                    i32.add
                    local.set 1
                  end
                  local.get 1
                  i32.const 1
                  i32.add
                  local.set 1
                end
                local.get 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NestedBlocks").expect("translation succeeds");

    let jmp_if = opcodes::lookup("JMPIF_L").unwrap().byte;
    let _jmp = opcodes::lookup("JMP_L").unwrap().byte;

    // Should have multiple conditional jumps for br_if
    let jmp_if_count = translation.script.iter().filter(|&&b| b == jmp_if).count();
    assert!(jmp_if_count >= 4, "expected at least 4 conditional jumps");

    // Verify final RET
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn block_results_are_preserved_and_tracked() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "preserve") (result i32)
                block (result i32)
                  i32.const 8
                end
                i32.const 1
                i32.add)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BlockResults").expect("translation succeeds");

    let add = opcodes::lookup("ADD").unwrap().byte;
    let ret = opcodes::lookup("RET").unwrap().byte;
    assert!(
        translation.script.contains(&add),
        "expected ADD after using block result"
    );
    assert_eq!(translation.script.last().copied(), Some(ret));
}

#[test]
fn branch_to_block_with_result_is_allowed() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "br_result") (result i32)
                block (result i32)
                  i32.const 42
                  br 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BranchResult").expect("translation succeeds");

    let ret = opcodes::lookup("RET").unwrap().byte;
    assert_eq!(translation.script.last().copied(), Some(ret));
}

#[test]
fn translate_early_return_in_blocks() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "early_return") (param i32) (result i32)
                block $exit (result i32)
                  local.get 0
                  i32.const 0
                  i32.eq
                  if
                    i32.const 42
                    return
                  end

                  local.get 0
                  i32.const 1
                  i32.eq
                  if
                    i32.const 43
                    return
                  end

                  local.get 0
                  i32.const 2
                  i32.eq
                  if
                    i32.const 44
                    return
                  end

                  i32.const 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "EarlyReturn").expect("translation succeeds");

    let ret = opcodes::lookup("RET").unwrap().byte;
    // Should have RET instructions for early returns
    assert!(translation.script.iter().filter(|&&b| b == ret).count() >= 3);
}

#[test]
fn translate_unreachable_after_branch() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dead_code") (result i32)
                i32.const 1
                br 0
                unreachable
                i32.const 2
                i32.add)
            )"#,
    )
    .expect("valid wat");

    translate_module(&wasm, "DeadCode").expect("unreachable code after br is valid wasm");
}

#[test]
fn translate_unreachable_after_return() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dead_code") (result i32)
                i32.const 5
                return
                i32.const 2
                i32.add)
            )"#,
    )
    .expect("valid wat");

    translate_module(&wasm, "DeadAfterReturn").expect("unreachable code after return is valid wasm");
}

#[test]
fn br_to_function_patches_jump_targets() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "exit") (result i32)
                i32.const 7
                br 0
                i32.const 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BrToFunction").expect("translation succeeds");
    let jmp_l = opcodes::lookup("JMP_L").unwrap().byte;
    let unpatched = translation
        .script
        .windows(5)
        .any(|window| window[0] == jmp_l && window[1..] == [0xFF, 0xFF, 0xFF, 0xFF]);
    assert!(!unpatched, "found unpatched JMP_L placeholder in script");
}
