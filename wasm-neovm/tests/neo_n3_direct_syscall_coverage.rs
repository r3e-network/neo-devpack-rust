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
    let descriptors = [
        "Neo.Crypto.SHA256",
        "Neo.Crypto.RIPEMD160",
        "Neo.Crypto.Murmur32",
        "Neo.Crypto.Keccak256",
        "Neo.Crypto.Hash160",
        "Neo.Crypto.Hash256",
        "Neo.Crypto.VerifyWithECDsa",
    ];

    for (idx, descriptor) in descriptors.iter().enumerate() {
        let contract_name = format!("DirectExtendedDescriptor{idx}");
        assert_descriptor_tokenized("neo", descriptor, &contract_name);
    }
}
