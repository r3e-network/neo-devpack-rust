//! Solana syscall adapter
//!
//! Maps Solana program syscalls to Neo equivalents.
//!
//! # Supported Mappings
//!
//! - Logging: `sol_log_*` → `System.Runtime.Log`
//! - Clock: `sol_get_clock_sysvar` → `System.Runtime.GetTime`
//! - Crypto: `sol_sha256`, `sol_keccak256` → `Neo.Crypto.*`
//! - CPI: `sol_invoke*` → `System.Contract.Call`
//! - Signatures: `sol_verify_signature` → `System.Runtime.CheckWitness`
//! - Storage: Account data operations → `System.Storage.*`

use super::{ChainAdapter, SourceChain};

/// Solana-to-Neo syscall adapter
pub struct SolanaAdapter;

impl ChainAdapter for SolanaAdapter {
    fn source_chain(&self) -> SourceChain {
        SourceChain::Solana
    }

    fn resolve_syscall(&self, module: &str, name: &str) -> Option<&'static str> {
        // Handle neo-solana-compat imports
        if module == "neo" {
            return crate::neo_syscalls::lookup_neo_syscall(name);
        }

        // Handle direct Solana syscall names
        if module == "solana" || module == "sol" {
            return map_solana_syscall(name);
        }

        // Handle SPL Token program calls
        if module == "spl_token" {
            return map_spl_token_syscall(name);
        }

        // Handle env imports that might come from Solana programs
        if module == "env" {
            return map_env_import(name);
        }

        None
    }

    fn recognizes_module(&self, module: &str) -> bool {
        matches!(
            module,
            "neo" | "solana" | "sol" | "env" | "syscall" | "opcode" | "spl_token"
        )
    }
}

/// Map Solana syscall names to Neo syscall descriptors
fn map_solana_syscall(name: &str) -> Option<&'static str> {
    match name {
        // ===== Logging =====
        "sol_log_" | "sol_log" | "log" => Some("System.Runtime.Log"),
        "sol_log_64" | "log_64" => Some("System.Runtime.Log"),
        "sol_log_pubkey" => Some("System.Runtime.Log"),
        "sol_log_compute_units" => Some("System.Runtime.Log"),
        "sol_log_data" => Some("System.Runtime.Log"),

        // ===== Time/Clock =====
        "sol_get_clock_sysvar" | "get_clock" => Some("System.Runtime.GetTime"),
        "sol_get_epoch_schedule_sysvar" => Some("System.Runtime.GetTime"),
        "sol_get_rent_sysvar" => None, // No direct equivalent

        // ===== Crypto =====
        "sol_sha256" | "sha256" => Some("Neo.Crypto.SHA256"),
        "sol_keccak256" | "keccak256" => Some("Neo.Crypto.Keccak256"),
        "sol_blake3" => Some("Neo.Crypto.SHA256"), // No direct equivalent, fallback
        "sol_secp256k1_recover" => Some("Neo.Crypto.VerifyWithECDsa"),
        "sol_alt_bn128_group_op" => None, // BN128 not supported
        "sol_poseidon" => None, // Poseidon not supported
        "sol_curve25519_validate" => None, // Ed25519 validation

        // ===== Program Invocation (CPI) =====
        "sol_invoke_signed" | "sol_invoke" | "invoke" => Some("System.Contract.Call"),
        "sol_invoke_signed_c" => Some("System.Contract.Call"),
        "sol_invoke_signed_rust" => Some("System.Contract.Call"),

        // ===== Memory Operations =====
        // These are handled by wasm-neovm runtime helpers, not syscalls
        "sol_memcpy_" | "sol_memcpy" => None,
        "sol_memmove_" | "sol_memmove" => None,
        "sol_memset_" | "sol_memset" => None,
        "sol_memcmp_" | "sol_memcmp" => None,

        // ===== Return Data =====
        "sol_set_return_data" => None, // Stack-based in NeoVM
        "sol_get_return_data" => None,

        // ===== Account/Program Info =====
        "sol_get_processed_sibling_instruction" => None,
        "sol_get_stack_height" => None,
        "sol_get_last_restart_slot" => None,

        // ===== Signature Verification =====
        "sol_verify_signature" => Some("System.Runtime.CheckWitness"),

        // ===== Address Lookup Tables =====
        "sol_get_epoch_rewards_sysvar" => None,

        // ===== System Program Operations =====
        "sol_create_program_address" => None, // PDA - needs runtime emulation
        "sol_try_find_program_address" => None,

        _ => None,
    }
}

/// Map SPL Token program calls to Neo equivalents
fn map_spl_token_syscall(name: &str) -> Option<&'static str> {
    match name {
        // SPL Token operations map to NEP-17 equivalents via contract calls
        "transfer" | "transfer_checked" => Some("System.Contract.Call"),
        "mint_to" | "mint_to_checked" => Some("System.Contract.Call"),
        "burn" | "burn_checked" => Some("System.Contract.Call"),
        "approve" | "approve_checked" => Some("System.Contract.Call"),
        "revoke" => Some("System.Contract.Call"),
        "initialize_mint" => Some("System.Contract.Call"),
        "initialize_account" => Some("System.Contract.Call"),
        "close_account" => Some("System.Contract.Call"),
        "freeze_account" => Some("System.Contract.Call"),
        "thaw_account" => Some("System.Contract.Call"),
        "get_account_data_size" => Some("System.Storage.Get"),
        _ => None,
    }
}

/// Map common env imports from Solana programs
fn map_env_import(name: &str) -> Option<&'static str> {
    match name {
        // Memory operations - handled by wasm-neovm runtime helpers
        "memcpy" | "__memcpy" => None,
        "memmove" | "__memmove" => None,
        "memset" | "__memset" => None,
        "memcmp" | "__memcmp" => None,

        // Panic/abort
        "abort" | "__rust_panic" | "rust_begin_unwind" => None, // Maps to ABORT opcode

        // Math functions (from libm)
        "__floatundidf" | "__floatundisf" => None, // Float conversion
        "__fixdfdi" | "__fixsfdi" => None,

        _ => None,
    }
}

/// Solana account info layout constants
#[allow(dead_code)]
pub mod account_layout {
    /// Size of a public key in bytes
    pub const PUBKEY_SIZE: usize = 32;

    /// Offset of lamports in serialized account info
    pub const LAMPORTS_OFFSET: usize = 0;

    /// Offset of data length in serialized account info
    pub const DATA_LEN_OFFSET: usize = 8;

    /// Offset of data in serialized account info
    pub const DATA_OFFSET: usize = 16;
}

/// Helper to generate storage key from Solana account pubkey
#[allow(dead_code)]
pub fn solana_pubkey_to_storage_key(pubkey: &[u8; 32]) -> Vec<u8> {
    // Use first 20 bytes as Neo-compatible key
    // Prefix with "sol:" to namespace Solana-origin data
    let mut key = Vec::with_capacity(24);
    key.extend_from_slice(b"sol:");
    key.extend_from_slice(&pubkey[..20]);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_syscall_mapping() {
        // Logging
        assert_eq!(map_solana_syscall("sol_log_"), Some("System.Runtime.Log"));
        assert_eq!(map_solana_syscall("sol_log"), Some("System.Runtime.Log"));
        assert_eq!(map_solana_syscall("sol_log_data"), Some("System.Runtime.Log"));

        // Crypto
        assert_eq!(map_solana_syscall("sol_sha256"), Some("Neo.Crypto.SHA256"));
        assert_eq!(map_solana_syscall("sol_keccak256"), Some("Neo.Crypto.Keccak256"));

        // CPI
        assert_eq!(map_solana_syscall("sol_invoke"), Some("System.Contract.Call"));
        assert_eq!(map_solana_syscall("sol_invoke_signed"), Some("System.Contract.Call"));
        assert_eq!(map_solana_syscall("sol_invoke_signed_rust"), Some("System.Contract.Call"));

        // Time
        assert_eq!(map_solana_syscall("sol_get_clock_sysvar"), Some("System.Runtime.GetTime"));

        // Signature
        assert_eq!(map_solana_syscall("sol_verify_signature"), Some("System.Runtime.CheckWitness"));
    }

    #[test]
    fn test_spl_token_syscall_mapping() {
        assert_eq!(map_spl_token_syscall("transfer"), Some("System.Contract.Call"));
        assert_eq!(map_spl_token_syscall("transfer_checked"), Some("System.Contract.Call"));
        assert_eq!(map_spl_token_syscall("mint_to"), Some("System.Contract.Call"));
        assert_eq!(map_spl_token_syscall("burn"), Some("System.Contract.Call"));
        assert_eq!(map_spl_token_syscall("get_account_data_size"), Some("System.Storage.Get"));
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
        assert!(adapter.recognizes_module("solana"));
        assert!(adapter.recognizes_module("sol"));
        assert!(adapter.recognizes_module("env"));
        assert!(adapter.recognizes_module("spl_token"));
        assert!(!adapter.recognizes_module("unknown"));
    }

    #[test]
    fn test_storage_key_generation() {
        let pubkey: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];
        let key = solana_pubkey_to_storage_key(&pubkey);
        assert_eq!(&key[..4], b"sol:");
        assert_eq!(key.len(), 24); // 4 + 20
    }
}
