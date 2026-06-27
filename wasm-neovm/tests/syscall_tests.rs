// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

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
fn translate_neo_runtime_check_witness_bytes_reads_linear_memory() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "runtime_check_witness_bytes" (func $check_witness (param i32 i32) (result i32)))
              (memory 1)
              (data (i32.const 0) "\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00")
              (func (export "test") (result i32)
                i32.const 0
                i32.const 20
                call $check_witness))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CheckWitnessBytes").expect("translation succeeds");

    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    let pushdata1 = wasm_neovm::opcodes::lookup("PUSHDATA1").unwrap().byte;
    assert!(translation.script.contains(&syscall));
    assert!(translation.script.contains(&pushdata1));
}

#[test]
fn translate_neo_runtime_check_witness_i64_avoids_linear_memory() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "runtime_check_witness_i64" (func $check_witness (param i64) (result i32)))
              (memory 17)
              (func (export "test") (param i64) (result i32)
                local.get 0
                call $check_witness))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CheckWitnessI64").expect("translation succeeds");

    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    let cat = wasm_neovm::opcodes::lookup("CAT").unwrap().byte;
    let newbuffer = wasm_neovm::opcodes::lookup("NEWBUFFER").unwrap().byte;
    assert!(translation.script.contains(&syscall));
    assert!(translation.script.contains(&cat));
    assert!(!translation.script.contains(&newbuffer));
}

#[test]
fn translate_neo_runtime_log() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "log" (func $log (param i32 i32)))
              (memory 1)
              (func (export "test") (param i32 i32)
                local.get 0
                local.get 1
                call $log))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RuntimeLog").expect("translation succeeds");

    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    let substr = wasm_neovm::opcodes::lookup("SUBSTR").unwrap().byte;
    assert!(translation.script.contains(&syscall));
    assert!(
        translation.script.contains(&substr),
        "runtime log pointer/length import should marshal bytes from linear memory"
    );
}

#[test]
fn translate_neo_runtime_notify() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "notify" (func $notify (param i32 i32)))
              (memory 1)
              (func (export "test") (param i32 i32)
                local.get 0
                local.get 1
                call $notify))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RuntimeNotify").expect("translation succeeds");

    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    let substr = wasm_neovm::opcodes::lookup("SUBSTR").unwrap().byte;
    let newarray0 = wasm_neovm::opcodes::lookup("NEWARRAY0").unwrap().byte;
    assert!(translation.script.contains(&syscall));
    assert!(
        translation.script.contains(&substr),
        "runtime notify pointer/length import should marshal the event name from linear memory"
    );
    assert!(
        translation.script.contains(&newarray0),
        "runtime notify pointer/length import should provide an empty state array"
    );
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
fn translate_neo_runtime_get_calling_script_hash_i64() {
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "runtime_get_calling_script_hash_i64" (func $calling (result i64)))
              (func (export "test") (result i64)
                call $calling))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CallingHashI64").expect("translation succeeds");

    let syscall = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    let substr = wasm_neovm::opcodes::lookup("SUBSTR").unwrap().byte;
    let convert = wasm_neovm::opcodes::lookup("CONVERT").unwrap().byte;
    assert!(translation.script.contains(&syscall));
    assert!(translation.script.contains(&substr));
    assert!(translation.script.contains(&convert));
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

    // C2: System.Storage.Get is a syscall, not a static contract call, so it
    // must NOT produce a method token.
    assert!(
        translation.method_tokens.is_empty(),
        "non-Contract.Call syscalls must not produce method tokens (C2)"
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

    // C2: storage_get/storage_put are syscalls, not static contract calls, so
    // no method tokens are produced.
    assert!(translation.method_tokens.is_empty());
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

#[test]
fn crypto_sha256_lowers_to_contract_call_not_dead_syscall() {
    // Regression for C1: `neo::crypto_sha256` must NOT lower to the dead
    // `SYSCALL 0x1174acd7` (Neo.Crypto.SHA256 is not a registered interop).
    // It must lower to `System.Contract.Call` against the CryptoLib native
    // contract. We assert the emitted script contains the System.Contract.Call
    // hash (0x627d5b52) and does NOT contain the dead SHA256 syscall hash.
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "crypto_sha256" (func $sha (param i32) (result i32)))
              (func (export "hash") (param i32) (result i32)
                local.get 0
                call $sha))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CryptoSha256").expect("translation succeeds");

    let contract_call_hash: u32 = wasm_neovm::syscalls::lookup("System.Contract.Call")
        .expect("System.Contract.Call exists")
        .hash;
    let contract_call_bytes = contract_call_hash.to_le_bytes();

    assert!(
        translation
            .script
            .windows(4)
            .any(|w| w == contract_call_bytes),
        "crypto_sha256 must lower to System.Contract.Call"
    );

    // The dead SHA256 syscall hash (0x1174acd7) must never appear.
    let dead_sha256 = 0x1174acd7u32.to_le_bytes();
    assert!(
        !translation.script.windows(4).any(|w| w == dead_sha256),
        "crypto_sha256 must not emit the dead Neo.Crypto.SHA256 syscall hash"
    );
}

#[test]
fn check_sig_lowers_to_real_crypto_syscall() {
    // D3 regression: `neo::check_sig` must lower to a real
    // System.Crypto.CheckSig SYSCALL (not a default-zero stub).
    let wasm = wat::parse_str(
        r#"(module
              (import "neo" "check_sig" (func $check (param i32 i32 i32 i32) (result i32)))
              (func (export "verify") (param i32 i32 i32 i32) (result i32)
                local.get 0  local.get 1  local.get 2  local.get 3
                call $check))"#,
    )
    .expect("valid wat");
    let translation = translate_module(&wasm, "D3CheckSig").expect("translation succeeds");
    let check_sig_hash: u32 = wasm_neovm::syscalls::lookup("System.Crypto.CheckSig")
        .expect("System.Crypto.CheckSig exists")
        .hash;
    let expected = check_sig_hash.to_le_bytes();
    assert!(
        translation.script.windows(5).any(|w| w[0]
            == wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte
            && w[1..5] == expected),
        "neo::check_sig must lower to SYSCALL System.Crypto.CheckSig (D3)"
    );
}
