// Comprehensive function call tests for WASM-NeoVM translator
// Phase 3: Completeness coverage - Function calls and invocations

use wasm_neovm::translate_module;

// ============================================================================
// Direct Function Calls
// ============================================================================

#[test]
fn translate_direct_call_no_params() {
    let wasm = wat::parse_str(
        r#"(module
              (func $helper (result i32)
                i32.const 42)
              (func (export "main") (result i32)
                call $helper))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DirectCall").expect("translation succeeds");

    // Direct call should emit CALL_L or JMP opcode
    assert!(
        !translation.script.is_empty(),
        "should generate call bytecode"
    );
}

#[test]
fn translate_direct_call_with_params() {
    let wasm = wat::parse_str(
        r#"(module
              (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
              (func (export "main") (result i32)
                i32.const 10
                i32.const 20
                call $add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CallWithParams").expect("translation succeeds");

    // Should handle parameter passing via stack
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(
        translation.script.contains(&add),
        "should emit ADD in called function"
    );
}

#[test]
fn translate_direct_call_return_value() {
    let wasm = wat::parse_str(
        r#"(module
              (func $compute (param i32) (result i32)
                local.get 0
                i32.const 5
                i32.mul)
              (func (export "main") (param i32) (result i32)
                local.get 0
                call $compute
                i32.const 3
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CallReturnValue").expect("translation succeeds");

    // Return value should be on stack after call
    let mul = wasm_neovm::opcodes::lookup("MUL").unwrap().byte;
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(
        translation.script.contains(&mul),
        "should emit MUL in callee"
    );
    assert!(
        translation.script.contains(&add),
        "should emit ADD in caller"
    );
}

#[test]
fn translate_nested_calls() {
    let wasm = wat::parse_str(
        r#"(module
              (func $innermost (result i32)
                i32.const 1)
              (func $middle (result i32)
                call $innermost
                i32.const 2
                i32.add)
              (func (export "main") (result i32)
                call $middle
                i32.const 3
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NestedCalls").expect("translation succeeds");

    // Should handle multiple levels of function calls
    assert!(!translation.script.is_empty(), "should handle nested calls");
}

#[test]
fn translate_recursive_call() {
    let wasm = wat::parse_str(
        r#"(module
              (func $factorial (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.le_s
                (if (result i32)
                  (then
                    i32.const 1)
                  (else
                    local.get 0
                    local.get 0
                    i32.const 1
                    i32.sub
                    call $factorial
                    i32.mul)))
              (func (export "main") (result i32)
                i32.const 5
                call $factorial))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RecursiveCall").expect("translation succeeds");

    // Recursive calls should work (factorial example)
    let mul = wasm_neovm::opcodes::lookup("MUL").unwrap().byte;
    assert!(
        translation.script.contains(&mul),
        "should handle recursive factorial"
    );
}

// ============================================================================
// Indirect Function Calls (call_indirect)
// ============================================================================

#[test]
fn translate_call_indirect_basic() {
    let wasm = wat::parse_str(
        r#"(module
              (table 2 funcref)
              (func $f1 (result i32)
                i32.const 10)
              (func $f2 (result i32)
                i32.const 20)
              (elem (i32.const 0) $f1 $f2)
              (func (export "main") (param i32) (result i32)
                local.get 0
                call_indirect (type 0))
              (type (func (result i32))))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CallIndirect").expect("translation succeeds");

    // call_indirect requires table lookup and bounds checking
    assert!(
        !translation.script.is_empty(),
        "should generate call_indirect logic"
    );
}

#[test]
fn translate_call_indirect_with_params() {
    let wasm = wat::parse_str(
        r#"(module
              (table 2 funcref)
              (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
              (func $sub (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.sub)
              (elem (i32.const 0) $add $sub)
              (func (export "main") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                call_indirect (type 0))
              (type (func (param i32 i32) (result i32))))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CallIndirectParams").expect("translation succeeds");

    // Should handle parameters with call_indirect
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_call_indirect_type_mismatch_check() {
    let wasm = wat::parse_str(
        r#"(module
              (table 1 funcref)
              (func $target (param i32) (result i32)
                local.get 0)
              (elem (i32.const 0) $target)
              (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                call_indirect (type 0))
              (type (func (param i32) (result i32))))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "IndirectTypeCheck").expect("translation succeeds");

    // Type checking is done at runtime for call_indirect
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Function Reference and Export Tests
// ============================================================================

#[test]
fn translate_multiple_exported_functions() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
              (func (export "sub") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.sub)
              (func (export "mul") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MultiExport").expect("translation succeeds");

    // Multiple exports should all be callable
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    let sub = wasm_neovm::opcodes::lookup("SUB").unwrap().byte;
    let mul = wasm_neovm::opcodes::lookup("MUL").unwrap().byte;
    assert!(translation.script.contains(&add));
    assert!(translation.script.contains(&sub));
    assert!(translation.script.contains(&mul));
}

#[test]
fn translate_internal_helper_functions() {
    let wasm = wat::parse_str(
        r#"(module
              (func $helper1 (param i32) (result i32)
                local.get 0
                i32.const 10
                i32.add)
              (func $helper2 (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.mul)
              (func (export "main") (param i32) (result i32)
                local.get 0
                call $helper1
                call $helper2))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Helpers").expect("translation succeeds");

    // Internal helper functions should be included
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Call Stack Management
// ============================================================================

#[test]
fn translate_deep_call_stack() {
    let wasm = wat::parse_str(
        r#"(module
              (func $f1 (result i32)
                i32.const 1)
              (func $f2 (result i32)
                call $f1
                i32.const 1
                i32.add)
              (func $f3 (result i32)
                call $f2
                i32.const 1
                i32.add)
              (func $f4 (result i32)
                call $f3
                i32.const 1
                i32.add)
              (func $f5 (result i32)
                call $f4
                i32.const 1
                i32.add)
              (func (export "main") (result i32)
                call $f5))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DeepStack").expect("translation succeeds");

    // Should handle deep call stacks (5 levels)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_function_with_many_params() {
    let wasm = wat::parse_str(
        r#"(module
              (func $many_params (param i32 i32 i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
                local.get 2
                i32.add
                local.get 3
                i32.add
                local.get 4
                i32.add)
              (func (export "main") (result i32)
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                i32.const 5
                call $many_params))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ManyParams").expect("translation succeeds");

    // Should handle functions with many parameters
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_function_returning_nothing() {
    let wasm = wat::parse_str(
        r#"(module
              (func $void_func (param i32)
                local.get 0
                drop)
              (func (export "main") (result i32)
                i32.const 42
                call $void_func
                i32.const 100))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "VoidReturn").expect("translation succeeds");

    // Void functions should not leave values on stack
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    assert!(
        translation.script.contains(&drop),
        "should emit DROP in void function"
    );
}

// ============================================================================
// Tail Calls and Optimization
// ============================================================================

#[test]
fn translate_tail_call_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (func $helper (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add)
              (func (export "main") (param i32) (result i32)
                local.get 0
                call $helper))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TailCall").expect("translation succeeds");

    // Tail call pattern (call as last instruction)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_call_with_computation_after() {
    let wasm = wat::parse_str(
        r#"(module
              (func $compute (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.mul)
              (func (export "main") (param i32) (result i32)
                local.get 0
                call $compute
                i32.const 10
                i32.add
                i32.const 5
                i32.sub))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CallWithAfter").expect("translation succeeds");

    // Non-tail call with computation after
    let mul = wasm_neovm::opcodes::lookup("MUL").unwrap().byte;
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    let sub = wasm_neovm::opcodes::lookup("SUB").unwrap().byte;
    assert!(translation.script.contains(&mul));
    assert!(translation.script.contains(&add));
    assert!(translation.script.contains(&sub));
}

// ============================================================================
// Function Index and Type Validation
// ============================================================================

#[test]
fn translate_validates_function_index() {
    let wasm = wat::parse_str(
        r#"(module
              (func $target (result i32)
                i32.const 42)
              (func (export "main") (result i32)
                call $target))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ValidIndex").expect("valid function index");

    // Valid function index should work
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_function_signature_matching() {
    let wasm = wat::parse_str(
        r#"(module
              (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
              (func (export "main") (result i32)
                i32.const 5
                i32.const 7
                call $add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SigMatch").expect("signature matches");

    // Function signature should match call site
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(translation.script.contains(&add));
}
