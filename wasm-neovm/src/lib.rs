//! wasm-neovm: WebAssembly to NeoVM translator
//!
//! This crate provides functionality to translate WebAssembly modules into
//! Neo N3 compatible NEF (Neo Executable Format) files.
//!
//! # Main Components
//!
//! - [`translator`]: Core WASM to NeoVM translation logic
//! - [`manifest`]: Neo N3 contract manifest generation
//! - [`metadata`]: NEF metadata extraction and handling
//! - [`nef`]: NEF file format utilities
//! - [`adapters`]: Cross-chain compilation adapters (Solana, Move)
//! - [`opcodes`]: NeoVM opcode definitions
//! - [`syscalls`]: NeoVM syscall definitions
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

pub mod adapters;
pub mod manifest;
pub mod metadata;
pub mod nef;
pub mod neo_syscalls;
pub mod numeric;
pub mod opcodes;
pub mod syscalls;
pub mod translator;

pub use adapters::SourceChain;
pub use manifest::RenderedManifest;
pub use metadata::{extract_nef_metadata, NefMetadata};
pub use nef::{write_nef, write_nef_with_metadata, MethodToken};
pub use translator::{
    translate_module, translate_with_config, ManifestOverlay, Translation, TranslationConfig,
};
