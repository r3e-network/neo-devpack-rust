//! Export section generation
//!
//! This module handles the generation of WASM exports for:
//! - Public and entry Move functions
//! - Memory (when storage is needed)

use crate::bytecode::MoveModule;
use anyhow::Result;
use wasm_encoder::{ExportKind, ExportSection, MemorySection};

/// Build export section and optional memory section
///
/// Returns:
/// - ExportSection: Exports for public functions and memory
/// - Option<MemorySection>: Memory section if storage is needed
pub fn build_exports(
    module: &MoveModule,
    imported_functions: u32,
    needs_storage: bool,
) -> Result<(ExportSection, Option<MemorySection>)> {
    let mut exports = ExportSection::new();
    let export_base = imported_functions;

    // Export public and entry functions
    for (idx, func) in module.functions.iter().enumerate() {
        if func.is_public || func.is_entry {
            exports.export(&func.name, ExportKind::Func, export_base + idx as u32);
        }
    }

    // Memory for resource key/value scratch space
    let memory = if needs_storage {
        let mut mem = MemorySection::new();
        mem.memory(wasm_encoder::MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        exports.export("memory", ExportKind::Memory, 0);
        Some(mem)
    } else {
        None
    };

    Ok((exports, memory))
}
