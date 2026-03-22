// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Branch control flow tests (br, br_if, br_table)

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_br_table_large() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dispatch") (param i32) (result i32)
                block $default
                  block $case7
                    block $case6
                      block $case5
                        block $case4
                          block $case3
                            block $case2
                              block $case1
                                block $case0
                                  local.get 0
                                  br_table $case0 $case1 $case2 $case3 $case4 $case5 $case6 $case7 $default
                                end
                                i32.const 0
                                return
                              end
                              i32.const 1
                              return
                            end
                            i32.const 2
                            return
                          end
                          i32.const 3
                          return
                        end
                        i32.const 4
                        return
                      end
                      i32.const 5
                      return
                    end
                    i32.const 6
                    return
                  end
                  i32.const 7
                  return
                end
                i32.const 99)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LargeBrTable").expect("translation succeeds");

    // Should have multiple conditional jumps for br_table dispatch
    let dup = opcodes::lookup("DUP").unwrap().byte;
    let jmp_if = opcodes::lookup("JMPIF_L").unwrap().byte;

    assert!(translation.script.iter().filter(|&&b| b == dup).count() >= 4);
    assert!(translation.script.iter().filter(|&&b| b == jmp_if).count() >= 4);
}

#[test]
fn br_table_loop_continue_with_extra_stack_fails() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "mix_bad") (param i32) (result i32)
                (local i32)
                block $outer (result i32)
                  loop $loop
                    local.get 0
                    br_table $loop $outer  ;; extra stack value when continuing
                  end
                  i32.const 99
                end)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "BrTableLoopBlockBad")
        .expect_err("unbalanced br_table to loop should fail");
    assert!(
        err.chain()
            .any(|cause| cause.to_string().contains("branch requires")),
        "unexpected error: {err}"
    );
}

#[test]
fn br_table_targets_across_loop_and_block_void_ok() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "mix_ok") (param i32) (result i32)
                block $outer
                  loop $loop
                    local.get 0
                    br_table $loop $outer
                  end
                end
                i32.const 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BrTableLoopBlockOk").expect("translation succeeds");
    let ret = opcodes::lookup("RET").unwrap().byte;
    assert_eq!(translation.script.last().copied(), Some(ret));
}

#[test]
fn br_table_mismatched_label_arities_fail() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "mix_mismatch") (param i32) (result i32)
                block $outer (result i32)
                  loop $loop
                    local.get 0
                    br_table $loop $outer   ;; loop label expects 0, outer expects 1
                  end
                  i32.const 7
                end)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "BrTableMismatch")
        .expect_err("br_table with mixed label arities should fail");
    assert!(
        err.chain()
            .any(|cause| cause.to_string().contains("branch requires")),
        "unexpected error: {err}"
    );
}

#[test]
fn translate_switch_like_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "switch") (param i32) (result i32)
                block $default
                  block $case2
                    block $case1
                      block $case0
                        local.get 0
                        i32.const 0
                        i32.eq
                        br_if $case0
                        local.get 0
                        i32.const 1
                        i32.eq
                        br_if $case1
                        local.get 0
                        i32.const 2
                        i32.eq
                        br_if $case2
                        br $default
                      end
                      i32.const 100
                      return
                    end
                    i32.const 200
                    return
                  end
                  i32.const 300
                  return
                end
                i32.const 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Switch").expect("translation succeeds");

    let ret = opcodes::lookup("RET").unwrap().byte;
    let ret_count = translation.script.iter().filter(|&&b| b == ret).count();
    assert!(ret_count >= 3, "expected multiple return paths");
}
