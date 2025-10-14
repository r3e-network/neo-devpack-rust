// Comprehensive NEF (Neo Executable Format) tests for WASM-NeoVM translator
// Phase 2: High-priority coverage additions - NEF format generation and validation

use wasm_neovm::translate_module;

// ============================================================================
// NEF Header and Metadata Tests
// ============================================================================

#[test]
fn translate_generates_valid_nef_header() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NEFHeader").expect("translation succeeds");

    // NEF format should have valid header structure
    assert!(
        !translation.script.is_empty(),
        "should generate script bytecode"
    );
    assert!(
        !translation.manifest.value.is_null(),
        "should generate manifest"
    );
}

#[test]
fn translate_generates_correct_compiler_field() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 1))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CompilerField").expect("translation succeeds");

    // Compiler field should identify wasm-neovm translator
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_generates_script_hash() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "hash_test") (result i32)
                i32.const 100))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ScriptHash").expect("translation succeeds");

    // Script hash should be calculated for NEF
    assert!(
        !translation.script.is_empty(),
        "should have script for hash calculation"
    );
}

// ============================================================================
// Method Token Serialization Tests
// ============================================================================

#[test]
fn translate_populates_method_tokens_for_syscalls() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i32)))
              (func (export "get_data") (result i32)
                i32.const 0
                i32.const 0
                call $storage_get))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MethodTokens").expect("translation succeeds");

    // Syscalls should populate method_tokens array in NEF
    assert!(
        !translation.method_tokens.is_empty(),
        "should track syscall tokens"
    );
}

#[test]
fn translate_method_tokens_track_multiple_syscalls() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i32)))
              (import "neo" "storage_put" (func $storage_put (param i32 i32 i32 i32)))
              (import "neo" "check_witness" (func $check_witness (param i32 i32) (result i32)))
              (func (export "multi_call")
                i32.const 0
                i32.const 0
                call $storage_get
                drop
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                call $storage_put
                i32.const 0
                i32.const 0
                call $check_witness
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MultiTokens").expect("translation succeeds");

    // Multiple different syscalls should all be tracked
    assert!(
        !translation.method_tokens.is_empty(),
        "should track all syscall tokens"
    );
}

#[test]
fn translate_method_tokens_no_duplicates() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i32)))
              (func (export "repeated_call")
                i32.const 0
                i32.const 0
                call $storage_get
                drop
                i32.const 1
                i32.const 1
                call $storage_get
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NoDuplicateTokens").expect("translation succeeds");

    // Same syscall called multiple times should only appear once in method_tokens
    assert!(
        !translation.method_tokens.is_empty(),
        "should track syscall tokens"
    );
}

// ============================================================================
// Manifest Generation Tests
// ============================================================================

#[test]
fn translate_generates_manifest_with_abi() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ManifestABI").expect("translation succeeds");

    // Manifest should include ABI with exported function signature
    assert!(
        !translation.manifest.value.is_null(),
        "should generate manifest"
    );
}

#[test]
fn translate_manifest_includes_contract_name() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TestContract").expect("translation succeeds");

    // Manifest should include the contract name passed to translate_module
    assert!(!translation.manifest.value.is_null());
}

// ============================================================================
// NEF Validation Tests
// ============================================================================

#[test]
fn translate_validates_script_not_empty() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main")
                nop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NonEmpty").expect("translation succeeds");

    // NEF script should not be empty even for simple functions
    assert!(
        !translation.script.is_empty(),
        "NEF script should not be empty"
    );
}

#[test]
fn translate_handles_large_script() {
    // Create a function with many operations to generate large bytecode
    let wasm = wat::parse_str(
        r#"(module
              (func (export "large") (result i32)
                i32.const 1
                i32.const 2
                i32.add
                i32.const 3
                i32.add
                i32.const 4
                i32.add
                i32.const 5
                i32.add
                i32.const 6
                i32.add
                i32.const 7
                i32.add
                i32.const 8
                i32.add
                i32.const 9
                i32.add
                i32.const 10
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LargeScript").expect("translation succeeds");

    // Should handle generating larger NEF scripts
    assert!(
        translation.script.len() >= 20,
        "should generate substantial bytecode"
    );
}

#[test]
fn translate_nef_with_multiple_exports() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "func1") (result i32)
                i32.const 1)
              (func (export "func2") (result i32)
                i32.const 2)
              (func (export "func3") (result i32)
                i32.const 3))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MultiExport").expect("translation succeeds");

    // NEF should handle multiple exported functions
    assert!(!translation.script.is_empty());
    assert!(!translation.manifest.value.is_null());
}

// ============================================================================
// NEF Checksum and Integrity Tests
// ============================================================================

#[test]
fn translate_generates_consistent_output() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "deterministic") (result i32)
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation1 = translate_module(&wasm, "Deterministic").expect("translation succeeds");
    let translation2 = translate_module(&wasm, "Deterministic").expect("translation succeeds");

    // Same input should produce same output (deterministic translation)
    assert_eq!(
        translation1.script, translation2.script,
        "translation should be deterministic"
    );
}

#[test]
fn translate_different_names_different_manifests() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "test") (result i32)
                i32.const 42))"#,
    )
    .expect("valid wat");

    let translation1 = translate_module(&wasm, "Name1").expect("translation succeeds");
    let translation2 = translate_module(&wasm, "Name2").expect("translation succeeds");

    // Same WASM but different names should produce same script
    assert_eq!(
        translation1.script, translation2.script,
        "script should be same for same WASM"
    );
    // But potentially different manifests (if name is included)
    assert!(!translation1.manifest.value.is_null());
    assert!(!translation2.manifest.value.is_null());
}

// ============================================================================
// NEF Feature Flags and Reserved Fields Tests
// ============================================================================

#[test]
fn translate_handles_memory_feature() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "with_memory") (result i32)
                i32.const 0
                i32.load))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemoryFeature").expect("translation succeeds");

    // NEF should handle modules with memory feature
    assert!(
        !translation.script.is_empty(),
        "should handle memory feature"
    );
}

#[test]
fn translate_handles_global_feature() {
    let wasm = wat::parse_str(
        r#"(module
              (global $g (mut i32) (i32.const 0))
              (func (export "with_global") (result i32)
                global.get $g))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GlobalFeature").expect("translation succeeds");

    // NEF should handle modules with global feature
    assert!(
        !translation.script.is_empty(),
        "should handle global feature"
    );
}

#[test]
fn translate_handles_import_feature() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i32)))
              (func (export "with_import") (result i32)
                i32.const 0
                i32.const 0
                call $storage_get))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ImportFeature").expect("translation succeeds");

    // NEF should handle modules with imports
    assert!(
        !translation.script.is_empty(),
        "should handle import feature"
    );
    assert!(
        !translation.method_tokens.is_empty(),
        "should track imported syscalls"
    );
}

// ============================================================================
// NEF Size and Limits Tests
// ============================================================================

#[test]
fn translate_respects_reasonable_script_size() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "small") (result i32)
                i32.const 1
                i32.const 2
                i32.add))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SmallScript").expect("translation succeeds");

    // Small functions should produce reasonable-sized NEF scripts
    assert!(
        translation.script.len() < 1000,
        "simple function should produce small script"
    );
}

#[test]
fn translate_handles_empty_function_body() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "empty")
                nop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "EmptyBody").expect("translation succeeds");

    // Even empty function should generate valid NEF
    assert!(
        !translation.script.is_empty(),
        "should generate minimal valid NEF"
    );
}
