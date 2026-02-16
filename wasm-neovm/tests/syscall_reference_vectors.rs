use wasm_neovm::translate_module;

fn translate_import(module: &str, import: &str, contract_name: &str) -> wasm_neovm::Translation {
    let wat = format!(
        r#"(module
              (import "{module}" "{import}" (func $syscall))
              (func (export "main")
                call $syscall)
            )"#
    );
    let wasm = wat::parse_str(&wat).expect("valid wat");
    translate_module(&wasm, contract_name).expect("translation succeeds")
}

fn assert_emits_syscall_hash(translation: &wasm_neovm::Translation, expected_hash: u32) {
    let syscall_opcode = wasm_neovm::opcodes::lookup("SYSCALL")
        .expect("SYSCALL opcode exists")
        .byte;
    let expected_hash = expected_hash.to_le_bytes();

    assert!(
        translation
            .script
            .windows(5)
            .any(|window| window[0] == syscall_opcode && window[1..5] == expected_hash),
        "expected SYSCALL with hash 0x{:08x}",
        u32::from_le_bytes(expected_hash)
    );
}

#[test]
fn critical_syscall_descriptors_match_reference_vectors() {
    let vectors: &[(&str, &str, u32)] = &[
        ("syscall", "System.Contract.Call", 0x525b7d62),
        ("syscall", "System.Contract.GetCallFlags", 0x813ada95),
        ("syscall", "System.Crypto.CheckSig", 0x27b3e756),
        ("syscall", "System.Crypto.CheckMultisig", 0x3adcd09e),
        ("syscall", "System.Iterator.Next", 0x9ced089c),
        ("syscall", "System.Runtime.CheckWitness", 0x8cec27f8),
        (
            "syscall",
            "System.Runtime.GetExecutingScriptHash",
            0x74a8fedb,
        ),
        ("syscall", "System.Runtime.GetTime", 0x0388c3b7),
        ("syscall", "System.Storage.Get", 0x31e85d92),
        ("syscall", "System.Storage.Put", 0x84183fe6),
        ("neo", "Neo.Crypto.Hash160", 0xac67b840),
        ("neo", "Neo.Crypto.VerifyWithECDsa", 0xcf822a6a),
    ];

    for (idx, (module, descriptor, expected_hash)) in vectors.iter().enumerate() {
        let resolved = wasm_neovm::syscalls::lookup_extended(descriptor)
            .unwrap_or_else(|| panic!("missing descriptor: {descriptor}"));
        assert_eq!(
            resolved.hash, *expected_hash,
            "reference hash mismatch for {descriptor}"
        );

        let contract_name = format!("ReferenceDescriptorVector{idx}");
        let translation = translate_import(module, descriptor, &contract_name);
        assert_emits_syscall_hash(&translation, *expected_hash);
    }
}

#[test]
fn critical_neo_aliases_lower_to_reference_vectors() {
    let vectors: &[(&str, &str, u32)] = &[
        ("storage_get", "System.Storage.Get", 0x31e85d92),
        ("storage_put", "System.Storage.Put", 0x84183fe6),
        ("check_witness", "System.Runtime.CheckWitness", 0x8cec27f8),
        ("get_time", "System.Runtime.GetTime", 0x0388c3b7),
        ("verify_signature", "System.Crypto.CheckSig", 0x27b3e756),
        (
            "verify_with_ecdsa",
            "Neo.Crypto.VerifyWithECDsa",
            0xcf822a6a,
        ),
        (
            "crypto_verify_with_ecdsa",
            "Neo.Crypto.VerifyWithECDsa",
            0xcf822a6a,
        ),
    ];

    for (idx, (alias, descriptor, expected_hash)) in vectors.iter().enumerate() {
        let resolved = wasm_neovm::neo_syscalls::lookup_neo_syscall(alias)
            .unwrap_or_else(|| panic!("missing alias: {alias}"));
        assert_eq!(
            resolved, *descriptor,
            "alias '{alias}' should resolve to '{descriptor}'"
        );

        let contract_name = format!("ReferenceAliasVector{idx}");
        let translation = translate_import("neo", alias, &contract_name);
        assert_emits_syscall_hash(&translation, *expected_hash);
    }
}
