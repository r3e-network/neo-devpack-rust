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

mod env;
mod spl_token;
mod storage;
mod syscalls;

use env::map_env_import;
use spl_token::map_spl_token_syscall;
use syscalls::map_solana_syscall;

pub use storage::{account_layout, solana_pubkey_to_storage_key};

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

#[cfg(test)]
mod tests;
