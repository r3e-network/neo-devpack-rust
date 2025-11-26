//! Chain-specific adapters for cross-chain compilation
//!
//! This module provides adapters that map syscalls and imports from
//! other blockchain platforms to their Neo equivalents.

pub mod solana;

use crate::neo_syscalls;
use crate::syscalls;

/// Source chain identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceChain {
    /// Native Neo/WASM contract
    Neo,
    /// Solana program compiled to WASM
    Solana,
    /// Move contract (future support)
    Move,
}

impl Default for SourceChain {
    fn default() -> Self {
        Self::Neo
    }
}

impl SourceChain {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "neo" | "native" => Some(Self::Neo),
            "solana" | "sol" => Some(Self::Solana),
            "move" | "aptos" | "sui" => Some(Self::Move),
            _ => None,
        }
    }
}

/// Adapter trait for chain-specific syscall mapping
pub trait ChainAdapter {
    /// Get the source chain
    fn source_chain(&self) -> SourceChain;

    /// Try to resolve an import to a Neo syscall descriptor
    ///
    /// Returns the Neo syscall descriptor if the import can be mapped,
    /// or None if it's not a recognized syscall from this chain.
    fn resolve_syscall(&self, module: &str, name: &str) -> Option<&'static str>;

    /// Check if an import module is recognized by this adapter
    fn recognizes_module(&self, module: &str) -> bool;
}

/// Get the appropriate adapter for a source chain
pub fn get_adapter(chain: SourceChain) -> Box<dyn ChainAdapter> {
    match chain {
        SourceChain::Neo => Box::new(NeoAdapter),
        SourceChain::Solana => Box::new(solana::SolanaAdapter),
        SourceChain::Move => Box::new(MoveAdapter), // Placeholder
    }
}

/// Native Neo adapter (passthrough)
struct NeoAdapter;

impl ChainAdapter for NeoAdapter {
    fn source_chain(&self) -> SourceChain {
        SourceChain::Neo
    }

    fn resolve_syscall(&self, module: &str, name: &str) -> Option<&'static str> {
        match module {
            "syscall" => syscalls::lookup(name).map(|s| s.name),
            "neo" => neo_syscalls::lookup_neo_syscall(name),
            _ => None,
        }
    }

    fn recognizes_module(&self, module: &str) -> bool {
        matches!(module, "syscall" | "neo" | "opcode" | "env")
    }
}

/// Move adapter placeholder
struct MoveAdapter;

impl ChainAdapter for MoveAdapter {
    fn source_chain(&self) -> SourceChain {
        SourceChain::Move
    }

    fn resolve_syscall(&self, module: &str, name: &str) -> Option<&'static str> {
        // Handle Move-compatible Neo syscall imports
        if module == "neo" {
            return crate::neo_syscalls::lookup_neo_syscall(name);
        }

        // Handle Move stdlib equivalents
        if module == "move_stdlib" || module == "aptos_stdlib" || module == "sui" {
            return map_move_stdlib(name);
        }

        // Handle resource operations
        if module == "move_resource" {
            return map_move_resource(name);
        }

        None
    }

    fn recognizes_module(&self, module: &str) -> bool {
        matches!(
            module,
            "neo" | "syscall" | "opcode" | "move_stdlib" | "aptos_stdlib" | "sui" | "move_resource"
        )
    }
}

/// Map Move stdlib functions to Neo equivalents
fn map_move_stdlib(name: &str) -> Option<&'static str> {
    match name {
        // Crypto
        "hash_sha256" | "sha2_256" => Some("Neo.Crypto.SHA256"),
        "hash_sha3_256" | "sha3_256" => Some("Neo.Crypto.SHA256"), // Fallback
        "hash_keccak256" | "keccak256" => Some("Neo.Crypto.Keccak256"),

        // Signature verification
        "ed25519_verify" | "verify_signature" => Some("System.Runtime.CheckWitness"),
        "secp256k1_verify" => Some("Neo.Crypto.VerifyWithECDsa"),

        // Time
        "timestamp_now" | "now_seconds" | "now_microseconds" => Some("System.Runtime.GetTime"),

        // Events/logging
        "emit_event" | "event_emit" => Some("System.Runtime.Notify"),
        "debug_print" | "native_print" => Some("System.Runtime.Log"),

        // Signer
        "signer_address_of" | "address_of" => Some("System.Runtime.GetCallingScriptHash"),
        "signer_borrow_address" => Some("System.Runtime.GetCallingScriptHash"),

        _ => None,
    }
}

/// Map Move resource operations to Neo storage
fn map_move_resource(name: &str) -> Option<&'static str> {
    match name {
        // Global resource operations
        "borrow_global" | "borrow_global_mut" => Some("System.Storage.Get"),
        "move_to" | "move_to_sender" => Some("System.Storage.Put"),
        "move_from" => Some("System.Storage.Delete"),
        "exists" | "exists_at" => Some("System.Storage.Get"), // Check if non-null

        _ => None,
    }
}
