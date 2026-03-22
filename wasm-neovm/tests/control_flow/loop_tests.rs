// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Loop control flow tests

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_nested_loops_with_breaks() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "nested_loop") (param i32) (result i32)
                (local i32 i32)
                loop $outer
                  local.get 1
                  i32.const 10
                  i32.ge_s
                  br_if 1

                  i32.const 0
                  local.set 2

                  loop $inner
                    local.get 2
                    i32.const 5
                    i32.ge_s
                    br_if 1

                    local.get 1
                    i32.const 1
                    i32.add
                    local.set 1

                    local.get 2
                    i32.const 1
                    i32.add
                    local.set 2

                    br $inner
                  end

                  br $outer
                end
                local.get 1)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "NestedLoops").expect_err("invalid branch should fail");
    let message = err.to_string();
    let branch_issue = err
        .chain()
        .any(|cause| cause.to_string().contains("branch requires"));
    assert!(
        branch_issue,
        "unexpected nested-loop branch error: {message}"
    );
}

#[test]
fn translate_loop_with_multiple_exits() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "multi_exit") (param i32) (result i32)
                (local i32)
                loop $continue
                  local.get 0
                  i32.const 100
                  i32.ge_s
                  br_if 1

                  local.get 0
                  i32.const 50
                  i32.eq
                  if
                    local.get 1
                    return
                  end

                  local.get 1
                  local.get 0
                  i32.add
                  local.set 1

                  local.get 0
                  i32.const 1
                  i32.add
                  local.set 0

                  br $continue
                end
                local.get 1)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "MultiExit").expect_err("invalid branch should fail");
    let message = err.to_string();
    let branch_issue = err
        .chain()
        .any(|cause| cause.to_string().contains("branch requires"));
    assert!(
        branch_issue,
        "unexpected multi-exit branch error: {message}"
    );
}

#[test]
fn loop_result_break_requires_value() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "bad_break") (result i32)
                loop (result i32)
                  br 1
                end))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "BadBreak")
        .expect_err("break without value should fail translation");
    let message = err.to_string();
    assert!(
        err.chain()
            .any(|cause| cause.to_string().contains("branch requires")),
        "unexpected error for missing loop result: {message}"
    );
}

#[test]
fn loop_result_break_with_value_succeeds() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "good_break") (result i32)
                loop (result i32)
                  i32.const 7
                  br 1
                end))"#,
    )
    .expect("valid wat");

    let translation =
        translate_module(&wasm, "GoodBreak").expect("break supplying result should succeed");
    let ret = opcodes::lookup("RET").unwrap().byte;
    assert_eq!(translation.script.last().copied(), Some(ret));
}

#[test]
fn loop_continue_requires_entry_stack_height() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "bad_continue")
                loop
                  i32.const 1
                  br 0   ;; extra stack item: should be rejected
                end))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "BadContinue")
        .expect_err("continue with mismatched stack should fail");
    let message = err.to_string();
    assert!(
        err.chain()
            .any(|cause| cause.to_string().contains("branch requires")),
        "unexpected continue stack error: {message}"
    );
}

#[test]
fn loop_continue_with_matching_stack_succeeds() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "good_continue")
                (local i32)
                loop
                  local.get 0
                  i32.const 1
                  i32.add
                  local.set 0
                  local.get 0
                  i32.const 3
                  i32.lt_s
                  br_if 0
                end))"#,
    )
    .expect("valid wat");

    let translation =
        translate_module(&wasm, "GoodContinue").expect("continue with balanced stack should pass");
    let ret = opcodes::lookup("RET").unwrap().byte;
    assert_eq!(translation.script.last().copied(), Some(ret));
}
