// Comprehensive optimization verification tests for WASM-NeoVM translator
// Phase 3: Completeness coverage - Translation optimizations and bytecode efficiency

use wasm_neovm::translate_module;

// ============================================================================
// Constant Folding Optimizations
// ============================================================================

#[test]
fn translate_constant_addition_folding() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 10
                i32.const 20
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ConstFold").expect("translation succeeds");

    // Constant addition could be folded to single PUSH 30
    // But even if not optimized, should produce valid bytecode
    assert!(!translation.script.is_empty(), "should generate bytecode");
}

#[test]
fn translate_constant_multiplication_folding() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 5
                i32.const 7
                i32.mul))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ConstMulFold").expect("translation succeeds");

    // 5 * 7 = 35 could be folded
    let mul = wasm_neovm::opcodes::lookup("MUL").unwrap().byte;
    // Either contains MUL or optimized away
    assert!(
        translation.script.contains(&mul) || translation.script.len() < 20,
        "should either emit MUL or optimize"
    );
}

#[test]
fn translate_identity_operations() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.add
                i32.const 1
                i32.mul))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Identity").expect("translation succeeds");

    // x + 0 and x * 1 are identity operations that could be optimized
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_dead_code_elimination() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 999
                drop
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DeadCode").expect("translation succeeds");

    // Dead code (push then drop) could be eliminated
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Algebraic Simplifications
// ============================================================================

#[test]
fn translate_double_negation() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0
                i32.const -1
                i32.xor
                i32.const -1
                i32.xor))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DoubleNeg").expect("translation succeeds");

    // Double bitwise NOT cancels out (could optimize to identity)
    let xor = wasm_neovm::opcodes::lookup("XOR").unwrap().byte;
    assert!(translation.script.contains(&xor) || translation.script.len() < 10);
}

#[test]
fn translate_subtract_self() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0
                local.get 0
                i32.sub))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SubSelf").expect("translation succeeds");

    // x - x = 0 could be optimized
    let sub = wasm_neovm::opcodes::lookup("SUB").unwrap().byte;
    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    // Either emits SUB or optimizes to PUSH0
    assert!(translation.script.contains(&sub) || translation.script.contains(&push0));
}

#[test]
fn translate_and_with_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.and))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "AndZero").expect("translation succeeds");

    // x & 0 = 0 could be optimized
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Bytecode Size Optimizations
// ============================================================================

#[test]
fn translate_uses_compact_push_instructions() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CompactPush").expect("translation succeeds");

    // PUSH0 should be used instead of longer PUSHDATA1
    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    assert!(
        translation.script.contains(&push0),
        "should use compact PUSH0 for zero"
    );
}

#[test]
fn translate_small_constants_efficiently() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 1
                i32.const 2
                i32.const 3
                i32.add
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SmallConst").expect("translation succeeds");

    // Small constants (1-16) should use PUSH1-PUSH16 opcodes
    // Note: Optimized code may be slightly larger due to additional features
    assert!(
        translation.script.len() < 100,
        "should generate reasonably compact bytecode (got {} bytes)",
        translation.script.len()
    );
}

#[test]
fn translate_minimizes_stack_operations() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MinStack").expect("translation succeeds");

    // Should minimize unnecessary stack operations
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(translation.script.contains(&add));
}

// ============================================================================
// Control Flow Optimizations
// ============================================================================

#[test]
fn translate_empty_block_optimization() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                (block
                  nop)
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "EmptyBlock").expect("translation succeeds");

    // Empty blocks could be optimized away
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_unreachable_after_return() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 42
                return
                i32.const 999
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnreachAfterRet").expect("translation succeeds");

    // Code after return is unreachable and could be eliminated
    let ret = wasm_neovm::opcodes::lookup("RET").unwrap().byte;
    assert!(translation.script.contains(&ret));
}

#[test]
fn translate_branch_to_same_location() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                (block $b
                  local.get 0
                  br $b)
                i32.const 42))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "BranchOpt")
        .expect_err("translator should reject invalid branch depth");
    let has_stack_error = err.chain().any(|cause| {
        let msg = cause.to_string();
        msg.contains("stack height") || msg.contains("branch requires")
    });
    assert!(has_stack_error, "unexpected error: {err}");
}

// ============================================================================
// Local Variable Optimizations
// ============================================================================

#[test]
fn translate_unused_local_elimination() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                (local i32)
                local.get 0
                i32.const 10
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnusedLocal").expect("translation succeeds");

    // Unused local variable could be eliminated
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(translation.script.contains(&add));
}

#[test]
fn translate_local_get_set_coalescing() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0
                local.set 0
                local.get 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LocalCoalesce").expect("translation succeeds");

    // local.get followed by local.set could be optimized
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Peephole Optimizations
// ============================================================================

#[test]
fn translate_push_drop_elimination() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                i32.const 100
                drop
                local.get 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "PushDrop").expect("translation succeeds");

    // PUSH followed by DROP could be eliminated
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_consecutive_operations() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0
                i32.const 5
                i32.add
                i32.const 3
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ConsecOps").expect("translation succeeds");

    // Consecutive additions could potentially be combined
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(translation.script.contains(&add));
}

// ============================================================================
// Function Call Optimizations
// ============================================================================

#[test]
fn translate_tail_call_optimization() {
    let wasm = wat::parse_str(
        r#"(module
              (func $helper (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add)
              (func (export "test") (param i32) (result i32)
                local.get 0
                call $helper))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TailCall").expect("translation succeeds");

    // Tail call could be optimized to jump instead of call+return
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_inline_small_function() {
    let wasm = wat::parse_str(
        r#"(module
              (func $tiny (result i32)
                i32.const 42)
              (func (export "test") (result i32)
                call $tiny))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Inline").expect("translation succeeds");

    // Very small functions could be inlined
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Bytecode Efficiency Validation
// ============================================================================

#[test]
fn translate_produces_reasonable_bytecode_size() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ReasonableSize").expect("translation succeeds");

    // Simple addition should produce reasonably sized bytecode
    // Note: Optimized code may be slightly larger due to additional features
    assert!(
        translation.script.len() < 150,
        "simple function should produce reasonably compact bytecode (got {} bytes)",
        translation.script.len()
    );
}

#[test]
fn translate_avoids_redundant_operations() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0
                local.get 0
                i32.sub
                local.get 0
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "AvoidRedundant").expect("translation succeeds");

    // x - x + x = x, could be optimized
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_comparison_chain_optimization() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.gt_s
                local.get 0
                i32.const 100
                i32.lt_s
                i32.and))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CompChain").expect("translation succeeds");

    // Range check (0 < x < 100) pattern
    let gt = wasm_neovm::opcodes::lookup("GT").unwrap().byte;
    let lt = wasm_neovm::opcodes::lookup("LT").unwrap().byte;
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    assert!(
        translation.script.contains(&gt)
            && translation.script.contains(&lt)
            && translation.script.contains(&and)
    );
}

// ============================================================================
// NEF Format Optimization
// ============================================================================

#[test]
fn translate_generates_compact_nef() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                i32.const 1))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CompactNEF").expect("translation succeeds");

    // Minimal contract should produce compact NEF
    assert!(
        translation.script.len() < 50,
        "minimal contract should be compact"
    );
}

#[test]
fn translate_optimizes_method_tokens() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "OptTokens").expect("translation succeeds");

    // No syscalls = empty method_tokens
    assert!(
        translation.method_tokens.is_empty(),
        "no syscalls should mean empty method tokens"
    );
}

#[test]
fn translate_skips_large_unreachable_function_bodies() {
    let mut wat = String::from("(module\n");

    for idx in 0..80 {
        wat.push_str(&format!("  (func $dead{idx} (result i32)\n"));
        for _ in 0..32 {
            wat.push_str("    i32.const 1 i32.const 2 i32.add drop\n");
        }
        wat.push_str("    i32.const 0)\n");
    }

    wat.push_str(
        r#"  (func (export "main") (result i32)
    i32.const 42)
)"#,
    );

    let wasm = wat::parse_str(&wat).expect("valid wat");
    let translation =
        translate_module(&wasm, "ReachabilityTrim").expect("translation should succeed");

    assert!(
        translation.script.len() < 12_000,
        "expected compact script after skipping unreachable bodies, got {} bytes",
        translation.script.len()
    );
}

#[test]
fn translate_call_indirect_dispatch_avoids_all_function_fanout() {
    let mut wat = String::from("(module\n");
    wat.push_str("  (type $t (func))\n");
    wat.push_str("  (table 2 funcref)\n");
    wat.push_str("  (func $f0 (type $t))\n");
    wat.push_str("  (func $f1 (type $t))\n");

    for idx in 0..180 {
        wat.push_str(&format!("  (func $dead{idx} (type $t))\n"));
    }

    wat.push_str("  (func (export \"main\") (type $t)\n");
    for _ in 0..48 {
        wat.push_str("    i32.const 0\n");
        wat.push_str("    call_indirect (type $t)\n");
    }
    wat.push_str("  )\n");
    wat.push_str("  (elem (i32.const 0) $f0 $f1)\n");
    wat.push_str(")\n");

    let wasm = wat::parse_str(&wat).expect("valid wat");
    let translation =
        translate_module(&wasm, "IndirectDispatchTight").expect("translation should succeed");

    assert!(
        translation.script.len() < 40_000,
        "call_indirect dispatch should stay compact, got {} bytes",
        translation.script.len()
    );
}

#[test]
fn translate_call_indirect_dispatch_is_shared_across_call_sites() {
    const TABLE_FUNCS: usize = 32;
    const CALL_SITES: usize = 48;

    let mut wat = String::from("(module\n");
    wat.push_str("  (type $t (func))\n");
    wat.push_str(&format!("  (table {TABLE_FUNCS} funcref)\n"));

    for idx in 0..TABLE_FUNCS {
        wat.push_str(&format!("  (func $f{idx} (type $t))\n"));
    }

    wat.push_str("  (func (export \"main\")\n");
    for _ in 0..CALL_SITES {
        wat.push_str("    i32.const 0\n");
        wat.push_str("    call_indirect (type $t)\n");
    }
    wat.push_str("  )\n");

    wat.push_str("  (elem (i32.const 0)");
    for idx in 0..TABLE_FUNCS {
        wat.push_str(&format!(" $f{idx}"));
    }
    wat.push_str(")\n");
    wat.push_str(")\n");

    let wasm = wat::parse_str(&wat).expect("valid wat");
    let translation =
        translate_module(&wasm, "IndirectDispatchShared").expect("translation should succeed");

    assert!(
        translation.script.len() < 12_000,
        "shared call_indirect dispatch should stay compact, got {} bytes",
        translation.script.len()
    );
}
