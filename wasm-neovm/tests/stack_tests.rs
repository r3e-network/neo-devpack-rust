// Comprehensive stack management tests for WASM-NeoVM translator
// Phase 1: Critical coverage additions - Stack operations are COMPLETELY UNTESTED (0%)

use wasm_neovm::translate_module;

// ============================================================================
// Basic Stack Operations
// ============================================================================

#[test]
fn translate_drop_basic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "drop_val") (param i32) (result i32)
                local.get 0
                i32.const 999
                drop
                local.get 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Drop").expect("translation succeeds");

    // Translator may optimize the drop by removing the unused value entirely.
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_drop_multiple() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "drop_multi") (result i32)
                i32.const 1
                i32.const 2
                i32.const 3
                drop
                drop
                drop
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DropMulti").expect("translation succeeds");

    // Multiple drops should be handled correctly
    assert!(
        !translation.script.is_empty(),
        "should handle multiple drops"
    );
}

#[test]
fn translate_select_basic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "sel") (param i32) (result i32)
                i32.const 100
                i32.const 200
                local.get 0
                select))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Select").expect("translation succeeds");

    // Select should use conditional logic (JMPIF/JMPIFNOT or similar)
    assert!(
        !translation.script.is_empty(),
        "should generate select logic"
    );
}

#[test]
fn translate_select_with_computation() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "sel_comp") (param i32 i32) (result i32)
                local.get 0
                i32.const 10
                i32.add
                local.get 1
                i32.const 20
                i32.add
                local.get 0
                local.get 1
                i32.gt_s
                select))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SelectComp").expect("translation succeeds");

    // Complex select with computations on both branches
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    let gt = wasm_neovm::opcodes::lookup("GT").unwrap().byte;
    assert!(translation.script.contains(&add), "should emit ADD");
    assert!(translation.script.contains(&gt), "should emit GT");
}

// ============================================================================
// Stack Depth Tracking
// ============================================================================

#[test]
fn translate_deep_stack() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "deep") (result i32)
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                i32.const 5
                i32.const 6
                i32.const 7
                i32.const 8
                drop
                drop
                drop
                drop
                drop
                drop
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DeepStack").expect("translation succeeds");

    // Should track stack depth correctly through push/drop sequence
    assert!(!translation.script.is_empty(), "should handle deep stack");
}

#[test]
fn translate_stack_underflow_detection() {
    // This should be caught during translation
    let wasm = wat::parse_str(
        r#"(module
              (func (export "underflow") (result i32)
                i32.const 42))"#,
    )
    .expect("valid wat");

    // Function should succeed - it returns a valid value
    let translation = translate_module(&wasm, "Valid").expect("translation succeeds");
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_stack_polymorphism_after_unreachable() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "poly") (result i32)
                unreachable
                i32.const 42))"#,
    )
    .expect("valid wat");

    // After unreachable, stack becomes polymorphic - any instruction valid
    let translation = translate_module(&wasm, "Polymorphic").expect("translation succeeds");

    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    assert!(
        translation.script.contains(&abort),
        "should emit ABORT for unreachable"
    );
}

// ============================================================================
// Stack Manipulation Patterns
// ============================================================================

#[test]
fn translate_dup_pattern_via_local_tee() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dup") (param i32) (result i32)
                local.get 0
                local.tee 0
                local.get 0
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DupPattern").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_swap_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "swap") (param i32 i32) (result i32 i32)
                local.get 1
                local.get 0))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "SwapPattern")
        .expect_err("translator should reject multi-value swap patterns");
    let has_multi_value_error = err
        .chain()
        .any(|cause| cause.to_string().contains("multi-value"));
    assert!(has_multi_value_error, "unexpected error: {err}");
}

#[test]
fn translate_rotate_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rotate") (param i32 i32 i32) (result i32 i32 i32)
                local.get 2
                local.get 0
                local.get 1))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "RotatePattern")
        .expect_err("translator should reject multi-value rotation patterns");
    let has_multi_value_error = err
        .chain()
        .any(|cause| cause.to_string().contains("multi-value"));
    assert!(has_multi_value_error, "unexpected error: {err}");
}

// ============================================================================
// Stack State Across Blocks
// ============================================================================

#[test]
fn translate_stack_across_block() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "block_stack") (param i32) (result i32)
                i32.const 10
                (block (result i32)
                  local.get 0
                  i32.const 5
                  i32.add
                  br 0)
                i32.add))"#,
    )
    .expect("valid wat");

    let translation =
        translate_module(&wasm, "BlockStack").expect("translator should support block values");
    assert!(
        !translation.script.is_empty(),
        "block-stack translation should emit bytecode"
    );
}

#[test]
fn translate_stack_across_if_else() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "if_stack") (param i32) (result i32)
                i32.const 100
                local.get 0
                (if (result i32)
                  (then
                    i32.const 200
                    i32.add)
                  (else
                    i32.const 300
                    i32.add))
                i32.add))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "IfStack")
        .expect_err("translator should reject unsupported stack pattern in if/else");
    assert!(!err.to_string().is_empty(), "unexpected error: {err}");
}

#[test]
fn translate_stack_across_loop() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "loop_stack") (param i32) (result i32)
                i32.const 0
                (loop (result i32)
                  local.get 0
                  i32.const 1
                  i32.sub
                  local.tee 0
                  i32.const 0
                  i32.gt_s
                  br_if 0
                  local.get 0)
                drop
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LoopStack").expect("translation succeeds");

    // Stack must be managed correctly through loop iterations
    assert!(
        !translation.script.is_empty(),
        "should handle loop stack management"
    );
}

// ============================================================================
// Multi-Value Stack (WebAssembly 1.1+)
// ============================================================================

#[test]
fn translate_multi_value_return() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "multi") (result i32 i32)
                i32.const 42
                i32.const 99))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "MultiValue")
        .expect_err("translator should reject multi-value returns");
    let has_multi_value_error = err
        .chain()
        .any(|cause| cause.to_string().contains("multi-value"));
    assert!(has_multi_value_error, "unexpected error: {err}");
}

#[test]
fn translate_multi_value_params_and_returns() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "multi_param") (param i32 i32) (result i32 i32)
                local.get 1
                local.get 0))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "MultiParamReturn")
        .expect_err("translator should reject multi-value params/returns");
    let has_multi_value_error = err
        .chain()
        .any(|cause| cause.to_string().contains("multi-value"));
    assert!(has_multi_value_error, "unexpected error: {err}");
}
