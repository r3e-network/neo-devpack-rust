// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::{syscalls, translate_module};

fn translate_descriptor(
    module: &str,
    descriptor: &str,
    contract_name: &str,
) -> wasm_neovm::Translation {
    let wat = format!(
        r#"(module
              (import "{module}" "{descriptor}" (func $syscall))
              (func (export "main")
                call $syscall)
            )"#
    );
    let wasm = wat::parse_str(&wat).expect("valid wat");
    translate_module(&wasm, contract_name).expect("translation succeeds")
}

fn assert_descriptor_tokenized(module: &str, descriptor: &str, contract_name: &str) {
    let translation = translate_descriptor(module, descriptor, contract_name);
    let syscall = wasm_neovm::opcodes::lookup("SYSCALL")
        .expect("SYSCALL opcode exists")
        .byte;
    let hash = syscalls::lookup_extended(descriptor)
        .unwrap_or_else(|| panic!("descriptor '{descriptor}' should resolve"))
        .hash
        .to_le_bytes();

    let emitted_hash = translation
        .script
        .windows(5)
        .any(|window| window[0] == syscall && window[1..5] == hash);

    assert!(
        emitted_hash,
        "descriptor '{descriptor}' should emit SYSCALL with the expected hash"
    );
}

#[test]
fn direct_translation_covers_all_system_syscalls() {
    for (idx, info) in syscalls::all().iter().enumerate() {
        let contract_name = format!("DirectSystemDescriptor{idx}");
        assert_descriptor_tokenized("neo", info.name, &contract_name);
    }
}

#[test]
fn direct_translation_covers_all_extended_crypto_descriptors() {
    // C1: Neo.Crypto.* names are CryptoLib native-contract methods invoked via
    // System.Contract.Call, NOT bare syscalls (there is no Register(...) for
    // them, so a SYSCALL <hash> would fault at runtime). The single-method
    // ones must emit a System.Contract.Call; the composite Hash160/Hash256
    // have no single method and must be rejected with a clear error.
    let single_method = [
        "Neo.Crypto.SHA256",
        "Neo.Crypto.RIPEMD160",
        "Neo.Crypto.Murmur32",
        "Neo.Crypto.Keccak256",
        "Neo.Crypto.VerifyWithECDsa",
    ];

    let contract_call_hash: [u8; 4] = syscalls::lookup("System.Contract.Call")
        .expect("System.Contract.Call exists")
        .hash
        .to_le_bytes();

    for (idx, descriptor) in single_method.iter().enumerate() {
        let contract_name = format!("DirectExtendedDescriptor{idx}");
        let translation = translate_descriptor("neo", descriptor, &contract_name);
        assert!(
            translation
                .script
                .windows(4)
                .any(|w| w == &contract_call_hash[..]),
            "{descriptor} must lower to System.Contract.Call (C1)"
        );
    }

    // Composite hashes must be rejected loudly at translation time.
    for composite in ["Neo.Crypto.Hash160", "Neo.Crypto.Hash256"] {
        let wat = format!(
            r#"(module
                  (import "neo" "{composite}" (func $syscall))
                  (func (export "main")
                    call $syscall)
                )"#
        );
        let wasm = wat::parse_str(&wat).expect("valid wat");
        let result = translate_module(&wasm, "CompositeCrypto");
        let err =
            result.expect_err("{composite} must be rejected (no single CryptoLib method) (C1)");
        // The error is wrapped with "failed to translate function"; walk the
        // full anyhow chain to confirm the composite-hash explanation.
        let full: Vec<String> = err.chain().map(|e| e.to_string()).collect();
        let joined = full.join(" :: ");
        assert!(
            joined.contains("composite hash"),
            "error should explain the composite-hash limitation, got: {joined}"
        );
    }
}
