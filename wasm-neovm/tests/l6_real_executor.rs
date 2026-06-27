// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! L6 real executor proof: the translator emits the correct
//! SYSCALL opcodes for cross-contract calls.
//!
//! The wasm32 cross-call mechanism is architecture A: the
//! translator emits a `SYSCALL 0x525B7D62` (System.Contract.Call)
//! into the deployed .nef, and the host's NeoVM dispatches the
//! call at runtime.
//!
//! For the SYSCALL to be emitted, `NeoVMSyscall::contract_call`
//! (and friends) must be **wasm imports** (so the translator
//! sees them as imports and emits the corresponding SYSCALL
//! opcodes). If they're regular Rust functions, the translator
//! inlines their bodies (which are the v0.11.0 Result-returning
//! stubs, not SYSCALLs).
//!
//! These tests inspect the raw NEF script bytes and assert the
//! SYSCALL opcodes are present.

#![cfg(feature = "exec")]

use std::path::PathBuf;
use std::process::Command;

use wasm_neovm::{translate_with_config, BehaviorConfig, TranslationConfig};

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !p.join("Cargo.toml").exists() || !p.join("contracts").exists() {
        if !p.pop() {
            panic!(
                "could not find workspace root from {}",
                env!("CARGO_MANIFEST_DIR")
            );
        }
    }
    p
}

const SYSCALL_OPCODE: u8 = 0x41;
const SYSTEM_CONTRACT_CALL_HASH: u32 = 0x525B_7D62;

fn build_and_get_script(contract: &str) -> Vec<u8> {
    let root = workspace_root();
    let contract_dir = root.join("contracts").join(contract);
    let target = root.join(format!(
        "target/conformance-builds/{contract}/wasm32-unknown-unknown/release"
    ));

    let build_out = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
            contract_dir.join("Cargo.toml").to_str().unwrap(),
        ])
        .env(
            "CARGO_TARGET_DIR",
            root.join(format!("target/conformance-builds/{contract}")),
        )
        .output()
        .expect("cargo build");

    if !build_out.status.success() {
        panic!(
            "cargo build for {} failed:\n{}",
            contract,
            String::from_utf8_lossy(&build_out.stderr)
        );
    }

    let mut wasm_path = None;
    for entry in std::fs::read_dir(&target).expect("read target dir") {
        let entry = entry.expect("entry");
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("wasm") {
            wasm_path = Some(p);
            break;
        }
    }
    let wasm_path = wasm_path.unwrap();
    let wasm_bytes = std::fs::read(&wasm_path).expect("read wasm");

    let config = TranslationConfig::new(contract).with_behavior(BehaviorConfig {
        strict_validation: true,
        ..BehaviorConfig::default()
    });
    let translation = translate_with_config(&wasm_bytes, config).expect("translate");
    translation.script
}

fn contains_syscall(script: &[u8], hash: u32) -> bool {
    let hash_bytes = hash.to_le_bytes();
    script.windows(5).any(|w| w[0] == SYSCALL_OPCODE && w[1..5] == hash_bytes)
}

#[test]
fn l6_real_executor_cross_call_wrapper_emits_contract_call() {
    let script = build_and_get_script("cross-call-wrapper");
    assert!(
        contains_syscall(&script, SYSTEM_CONTRACT_CALL_HASH),
        "cross-call-wrapper must emit SYSCALL System.Contract.Call (0x{:08x}) \
         for the production cross-call path; got script: {:02x?}",
        SYSTEM_CONTRACT_CALL_HASH,
        &script[..script.len().min(200)]
    );
}

#[test]
fn l6_real_executor_hello_world_does_not_emit_contract_call() {
    // Sanity check: contracts that don't cross-call should NOT
    // emit System.Contract.Call.
    let script = build_and_get_script("hello-world");
    assert!(
        !contains_syscall(&script, SYSTEM_CONTRACT_CALL_HASH),
        "hello-world should not emit System.Contract.Call"
    );
}
