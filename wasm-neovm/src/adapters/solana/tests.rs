use super::*;

#[test]
fn test_solana_syscall_mapping() {
    // Logging
    assert_eq!(map_solana_syscall("sol_log_"), Some("System.Runtime.Log"));
    assert_eq!(map_solana_syscall("sol_log"), Some("System.Runtime.Log"));
    assert_eq!(
        map_solana_syscall("sol_log_data"),
        Some("System.Runtime.Log")
    );

    // Crypto
    assert_eq!(map_solana_syscall("sol_sha256"), Some("Neo.Crypto.SHA256"));
    assert_eq!(
        map_solana_syscall("sol_keccak256"),
        Some("Neo.Crypto.Keccak256")
    );

    // CPI
    assert_eq!(
        map_solana_syscall("sol_invoke"),
        Some("System.Contract.Call")
    );
    assert_eq!(
        map_solana_syscall("sol_invoke_signed"),
        Some("System.Contract.Call")
    );
    assert_eq!(
        map_solana_syscall("sol_invoke_signed_rust"),
        Some("System.Contract.Call")
    );

    // Time
    assert_eq!(
        map_solana_syscall("sol_get_clock_sysvar"),
        Some("System.Runtime.GetTime")
    );

    // Signature
    assert_eq!(
        map_solana_syscall("sol_verify_signature"),
        Some("System.Runtime.CheckWitness")
    );
}

#[test]
fn test_spl_token_syscall_mapping() {
    assert_eq!(
        map_spl_token_syscall("transfer"),
        Some("System.Contract.Call")
    );
    assert_eq!(
        map_spl_token_syscall("transfer_checked"),
        Some("System.Contract.Call")
    );
    assert_eq!(
        map_spl_token_syscall("mint_to"),
        Some("System.Contract.Call")
    );
    assert_eq!(map_spl_token_syscall("burn"), Some("System.Contract.Call"));
    assert_eq!(
        map_spl_token_syscall("get_account_data_size"),
        Some("System.Storage.Get")
    );
}

#[test]
fn test_env_import_mapping() {
    // Memory ops return None (handled by runtime)
    assert_eq!(map_env_import("memcpy"), None);
    assert_eq!(map_env_import("__memcpy"), None);
    assert_eq!(map_env_import("memmove"), None);

    // Panic/abort
    assert_eq!(map_env_import("abort"), None);
    assert_eq!(map_env_import("__rust_panic"), None);
}

#[test]
fn test_adapter_recognizes_modules() {
    let adapter = SolanaAdapter;
    assert!(adapter.recognizes_module("neo"));
    assert!(adapter.recognizes_module("NeO"));
    assert!(adapter.recognizes_module("solana"));
    assert!(adapter.recognizes_module("SoLaNa"));
    assert!(adapter.recognizes_module("sol"));
    assert!(adapter.recognizes_module("env"));
    assert!(adapter.recognizes_module("spl_token"));
    assert!(adapter.recognizes_module("SyScAlL"));
    assert!(!adapter.recognizes_module("unknown"));
}

#[test]
fn test_adapter_passthroughs_syscall_module_extended_descriptors() {
    let adapter = SolanaAdapter;
    assert_eq!(
        adapter.resolve_syscall("syscall", "Neo.Crypto.VerifyWithECDsa"),
        Some("Neo.Crypto.VerifyWithECDsa")
    );
    assert_eq!(
        adapter.resolve_syscall("SyScAlL", "Neo.Crypto.VerifyWithECDsa"),
        Some("Neo.Crypto.VerifyWithECDsa")
    );
}

#[test]
fn test_storage_key_generation() {
    let pubkey: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];
    let key = solana_pubkey_to_storage_key(&pubkey);
    assert_eq!(&key[..4], b"sol:");
    assert_eq!(key.len(), 24); // 4 + 20
}
