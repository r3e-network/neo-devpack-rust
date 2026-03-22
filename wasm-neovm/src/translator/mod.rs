// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Core WASM to NeoVM translation engine
//!
//! This module contains the main translation logic that converts WebAssembly
//! bytecode into NeoVM opcodes.
//!
//! # Translation Process
//!
//! 1. **Frontend**: Parses and validates WASM module
//! 2. **IR**: Intermediate representation with type information
//! 3. **Translation**: Converts WASM instructions to NeoVM opcodes
//! 4. **Runtime**: Generates helper functions for memory, tables, etc.
//!
//! # Example
//!
//! ```rust,ignore
//! use wasm_neovm::translator::{translate_module, TranslationConfig};
//!
//! let config = TranslationConfig::new("MyContract");
//! let translation = translate_with_config(&wasm_bytes, config)?;
//! ```

mod constants;
mod frontend;
mod helpers;
mod ir;
mod runtime;
mod translation;
mod types;

// Arena allocator for temporary objects (Round 83)
pub mod arena;

// Profiling instrumentation (Round 70, 90)
pub mod profiling;

pub use translation::{translate_module, translate_with_config};
pub use types::{ManifestData, ManifestOverlay, Translation, TranslationConfig};

pub(crate) use frontend::ModuleFrontend;
pub(crate) use ir::{FunctionImport, ModuleTypes};
