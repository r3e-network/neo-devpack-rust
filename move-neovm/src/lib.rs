//! Move Bytecode to NeoVM Translator
//!
//! This crate provides a minimal Move bytecode → WASM translator to feed the
//! wasm-neovm pipeline. The lowering is experimental and does not cover the
//! full Move semantics yet.
//!
//! # Pipeline
//!
//! ```text
//! Move Source → Move Compiler → Move Bytecode → move-neovm → WASM → wasm-neovm → NEF
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use move_neovm::{parse_move_bytecode, translate_to_wasm};
//!
//! // Parse Move bytecode
//! let module = parse_move_bytecode(&bytecode)?;
//!
//! // Translate to WASM
//! let wasm = translate_to_wasm(&module)?;
//!
//! // Then use wasm-neovm to generate NEF
//! ```
//!
//! # Challenges
//!
//! Move has unique features that don't map directly to WASM/NeoVM:
//! - Linear resource types (no copy/drop without ability)
//! - Module system with publishing semantics
//! - Global typed storage model
//!
//! These require runtime emulation or compile-time transformation.

pub mod bytecode;
pub mod runtime;
pub mod translator;

pub use bytecode::{
    parse_move_bytecode, validate_move_bytecode, BytecodeVersion, FieldDef, FunctionDef,
    MoveModule, MoveOpcode, StructDef, TypeTag,
};
pub use runtime::{global_storage_key, signer_to_checkwitness, ResourceError, ResourceTracker};
pub use translator::translate_to_wasm;

use anyhow::Result;

/// Translation result containing WASM module bytes
pub struct MoveTranslation {
    /// Generated WASM module bytes
    pub wasm: Vec<u8>,
    /// Module metadata
    pub metadata: MoveModuleMetadata,
}

/// Metadata about the translated Move module
#[derive(Debug, Clone, Default)]
pub struct MoveModuleMetadata {
    /// Module name
    pub name: String,
    /// Exported functions
    pub functions: Vec<String>,
    /// Resource types defined
    pub resources: Vec<String>,
}

/// Translate Move bytecode to WASM
///
/// # Arguments
/// * `bytecode` - Raw Move bytecode bytes
/// * `module_name` - Name for the output module
///
/// # Returns
/// Translation result containing WASM bytes and metadata
pub fn translate_move_to_wasm(bytecode: &[u8], module_name: &str) -> Result<MoveTranslation> {
    // Parse the Move bytecode
    let module = parse_move_bytecode(bytecode)?;

    // Translate to WASM
    let wasm = translate_to_wasm(&module)?;

    // Collect metadata
    let metadata = MoveModuleMetadata {
        name: module_name.to_string(),
        functions: module
            .functions
            .iter()
            .filter(|f| f.is_public || f.is_entry)
            .map(|f| f.name.clone())
            .collect(),
        resources: module
            .structs
            .iter()
            .filter(|s| s.abilities.is_resource())
            .map(|s| s.name.clone())
            .collect(),
    };

    Ok(MoveTranslation { wasm, metadata })
}

/// Check if the given bytes appear to be valid Move bytecode
pub fn is_move_bytecode(bytes: &[u8]) -> bool {
    validate_move_bytecode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_move_bytecode() {
        // Valid Move magic
        assert!(is_move_bytecode(&[
            0xa1, 0x1c, 0xeb, 0x0b, 0x00, 0x00, 0x00, 0x00
        ]));

        // Invalid - WASM magic
        assert!(!is_move_bytecode(&[0x00, 0x61, 0x73, 0x6d]));
    }
}
