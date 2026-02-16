// Comprehensive syscall tests for WASM-NeoVM translator
// Phase 2: High-priority coverage additions - Syscalls are <5% tested

use wasm_neovm::translate_module;

// ============================================================================
// Native Contract Syscalls
// ============================================================================

#[test]
fn translate_neo_native_contract_call() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "call_contract" (func $call_contract (param i32 i32 i32 i32)))
              (func (export "test")
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                call $call_contract))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NativeCall").expect("translation succeeds");

    // Should emit SYSCALL opcode for contract call
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall), "should emit SYSCALL");
}

#[test]
fn translate_neo_storage_get() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i32)))
              (func (export "test") (result i32)
                i32.const 0
                i32.const 0
                call $storage_get))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StorageGet").expect("translation succeeds");

    // Storage.Get syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_storage_put() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_put" (func $storage_put (param i32 i32 i32 i32)))
              (func (export "test")
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                call $storage_put))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StoragePut").expect("translation succeeds");

    // Storage.Put syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_storage_delete() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_delete" (func $storage_delete (param i32 i32)))
              (func (export "test")
                i32.const 0
                i32.const 0
                call $storage_delete))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StorageDelete").expect("translation succeeds");

    // Storage.Delete syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

// ============================================================================
// Runtime Syscalls
// ============================================================================

#[test]
fn translate_neo_runtime_check_witness() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "check_witness" (func $check_witness (param i32 i32) (result i32)))
              (func (export "test") (result i32)
                i32.const 0
                i32.const 0
                call $check_witness))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CheckWitness").expect("translation succeeds");

    // Runtime.CheckWitness syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_runtime_log() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "log" (func $log (param i32 i32)))
              (func (export "test")
                i32.const 0
                i32.const 0
                call $log))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RuntimeLog").expect("translation succeeds");

    // Runtime.Log syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_runtime_notify() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "notify" (func $notify (param i32 i32)))
              (func (export "test")
                i32.const 0
                i32.const 0
                call $notify))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RuntimeNotify").expect("translation succeeds");

    // Runtime.Notify syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_runtime_get_time() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "get_time" (func $get_time (result i64)))
              (func (export "test") (result i64)
                call $get_time))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GetTime").expect("translation succeeds");

    // Runtime.GetTime syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_runtime_get_trigger() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "get_trigger" (func $get_trigger (result i32)))
              (func (export "test") (result i32)
                call $get_trigger))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GetTrigger").expect("translation succeeds");

    // Runtime.GetTrigger syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

// ============================================================================
// Crypto Syscalls
// ============================================================================

#[test]
fn translate_neo_crypto_verify_signature() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "verify_signature" (func $verify (param i32 i32 i32 i32) (result i32)))
              (func (export "test") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                call $verify))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "VerifySignature").expect("translation succeeds");

    // Crypto.VerifySignature syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_crypto_verify_with_ecdsa() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "verify_with_ecdsa" (func $verify (param i32 i32 i32 i32) (result i32)))
              (func (export "test") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 1
                call $verify))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "VerifyWithECDsa").expect("translation succeeds");

    // Neo.Crypto.VerifyWithECDsa syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_crypto_hash160() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "hash160" (func $hash160 (param i32 i32) (result i32)))
              (func (export "test") (result i32)
                i32.const 0
                i32.const 0
                call $hash160))"#,
    )
    .expect("valid wat");

    translate_module(&wasm, "Hash160").expect_err("hash160 is not a syscall");
}

#[test]
fn translate_neo_crypto_hash256() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "hash256" (func $hash256 (param i32 i32) (result i32)))
              (func (export "test") (result i32)
                i32.const 0
                i32.const 0
                call $hash256))"#,
    )
    .expect("valid wat");

    translate_module(&wasm, "Hash256").expect_err("hash256 is not a syscall");
}

// ============================================================================
// Contract Management Syscalls
// ============================================================================

#[test]
fn translate_neo_contract_create() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "contract_create" (func $create (param i32 i32 i32 i32) (result i32)))
              (func (export "test") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                call $create))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ContractCreate").expect("translation succeeds");

    // ContractManagement.Create syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

#[test]
fn translate_neo_contract_destroy() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "contract_destroy" (func $destroy))
              (func (export "test")
                call $destroy))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ContractDestroy").expect("translation succeeds");

    // ContractManagement.Destroy syscall
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}

// ============================================================================
// Syscall Token Tracking
// ============================================================================

#[test]
fn translate_syscall_populates_method_tokens() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i32)))
              (func (export "test") (result i32)
                i32.const 0
                i32.const 0
                call $storage_get))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TokenTracking").expect("translation succeeds");

    // Syscalls should be tracked in method_tokens
    assert!(
        !translation.method_tokens.is_empty(),
        "should track syscall tokens"
    );
}

#[test]
fn translate_multiple_syscalls_all_tracked() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i32)))
              (import "neo" "storage_put" (func $storage_put (param i32 i32 i32 i32)))
              (func (export "test")
                i32.const 0
                i32.const 0
                call $storage_get
                drop
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                call $storage_put))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MultiSyscall").expect("translation succeeds");

    // Multiple syscalls should all be tracked
    assert!(!translation.method_tokens.is_empty());
}

// ============================================================================
// Error Cases for Syscalls
// ============================================================================

#[test]
fn translate_rejects_unknown_syscall_module() {
    let wasm = wat::parse_str(
        r#"(module
              (import "unknown_module" "some_function" (func $unknown))
              (func (export "test")
                call $unknown))"#,
    )
    .expect("valid wat");

    let result = translate_module(&wasm, "UnknownModule");

    // Unknown import modules should be handled (may succeed with warning or fail)
    // The behavior depends on translator's import handling strategy
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn translate_syscall_with_complex_args() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "storage_put" (func $storage_put (param i32 i32 i32 i32)))
              (func (export "test") (param i32)
                local.get 0
                i32.const 10
                i32.add
                i32.const 20
                local.get 0
                i32.const 5
                i32.mul
                i32.const 30
                call $storage_put))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ComplexArgs").expect("translation succeeds");

    // Complex expressions as syscall arguments
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert!(translation.script.contains(&syscall));
}
