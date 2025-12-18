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
fn translate_complex_if_else_chain() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "classify") (param i32) (result i32)
                local.get 0
                i32.const 10
                i32.lt_s
                if (result i32)
                  local.get 0
                  i32.const 5
                  i32.lt_s
                  if (result i32)
                    i32.const 1
                  else
                    i32.const 2
                  end
                else
                  local.get 0
                  i32.const 20
                  i32.lt_s
                  if (result i32)
                    i32.const 3
                  else
                    local.get 0
                    i32.const 30
                    i32.lt_s
                    if (result i32)
                      i32.const 4
                    else
                      i32.const 5
                    end
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "IfChain").expect("translation succeeds");

    // Multiple nested if/else should generate multiple jumps
    let jmpifnot = opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    let jmp = opcodes::lookup("JMP_L").unwrap().byte;

    assert!(translation.script.contains(&jmpifnot));
    assert!(translation.script.contains(&jmp));
}

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

#[test]
fn translate_recursive_structure() {
    let wasm = wat::parse_str(
        r#"(module
              (func $factorial (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.le_s
                if (result i32)
                  i32.const 1
                else
                  local.get 0
                  local.get 0
                  i32.const 1
                  i32.sub
                  call $factorial
                  i32.mul
                end)
              (func (export "main") (result i32)
                i32.const 5
                call $factorial)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Recursive").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    // Should have recursive call to self
    let call_count = translation.script.iter().filter(|&&b| b == call_l).count();
    assert!(call_count >= 2, "expected recursive calls");
}

#[test]
fn translate_mutual_recursion() {
    let wasm = wat::parse_str(
        r#"(module
              (func $is_even (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.eq
                if (result i32)
                  i32.const 1
                else
                  local.get 0
                  i32.const 1
                  i32.sub
                  call $is_odd
                end)

              (func $is_odd (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.eq
                if (result i32)
                  i32.const 0
                else
                  local.get 0
                  i32.const 1
                  i32.sub
                  call $is_even
                end)

              (func (export "check_even") (param i32) (result i32)
                local.get 0
                call $is_even)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MutualRecursion").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
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
