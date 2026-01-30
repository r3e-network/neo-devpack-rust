// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! wasm-neovm: WebAssembly to NeoVM translator
//!
//! This crate provides functionality to translate WebAssembly modules into
//! Neo N3 compatible NEF (Neo Executable Format) files.
//!
//! # Module Organization
//!
//! ## Core Modules
//! - [`core`]: Core abstractions and shared traits
//! - [`types`]: Type-safe identifiers and primitive wrappers
//! - [`config`]: Centralized configuration management
//!
//! ## Translation Modules
//! - [`translator`]: Core WASM to NeoVM translation logic
//! - [`adapters`]: Cross-chain compilation adapters (Solana, Move)
//!
//! ## Output Modules
//! - [`manifest`]: Neo N3 contract manifest generation
//! - [`metadata`]: NEF metadata extraction and handling
//! - [`nef`]: NEF file format utilities
//!
//! ## Definition Modules
//! - [`opcodes`]: NeoVM opcode definitions
//! - [`syscalls`]: NeoVM syscall definitions
//! - [`neo_syscalls`]: Neo-specific syscall aliases
//!
//! # Example
//!
//! ```rust,ignore
//! use wasm_neovm::{translate_module, write_nef};
//!
//! let wasm_bytes = std::fs::read("contract.wasm")?;
//! let translation = translate_module(&wasm_bytes, "MyContract")?;
//! write_nef(&translation.script, "contract.nef")?;
//! ```

// Core modules (Round 131 - Module Reorganization)
pub mod config;
pub mod core;
pub mod types;

// Logging (Round 135 - Logging Standardization)
pub mod logging;

// API consistency layer (Round 136 - API Consistency)
pub mod api;

// Translation modules
pub mod adapters;
pub mod translator;

// Output modules
pub mod manifest;
pub mod metadata;
pub mod nef;

// Definition modules
pub mod neo_syscalls;
pub mod numeric;
pub mod opcodes;
pub mod syscalls;

// Re-exports for convenient access
pub use adapters::SourceChain;
pub use config::{BehaviorConfig, DebugConfig, OutputConfig, TranslationConfig};
pub use core::traits;
pub use logging::LogLevel;
pub use manifest::RenderedManifest;
pub use metadata::{extract_nef_metadata, NefMetadata};
pub use nef::{write_nef, write_nef_with_metadata, MethodToken};
pub use translator::{translate_module, translate_with_config, ManifestOverlay, Translation};
pub use types::{
    BytecodeOffset, ContractName, GlobalIndex, LocalIndex, MemoryOffset, MethodIndex,
    SyscallDescriptor, WasmValueType,
};

/// Version information for the wasm-neovm crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get version information as a formatted string
pub fn version_info() -> String {
    format!(
        "wasm-neovm v{} (Rust {})",
        VERSION,
        rustc_version_runtime::version()
    )
}

pub mod prelude {
    //! Commonly used types and traits

    pub use crate::adapters::SourceChain;
    pub use crate::config::{BehaviorConfig, DebugConfig, OutputConfig, TranslationConfig};
    pub use crate::core::traits::{BytecodeEmitter, ToBytecode, Translatable};
    pub use crate::translator::{translate_module, translate_with_config, Translation};
    pub use crate::types::ContractName;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
    }
}
