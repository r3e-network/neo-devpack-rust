// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Move to WASM lowering module
//!
//! This module orchestrates the translation of Move bytecode to WASM by
//! coordinating type generation, imports, function lowering, and exports.

use crate::bytecode::MoveModule;
use anyhow::Result;
use wasm_encoder::{Module, ValType};

mod exports;
mod functions;
mod imports;
mod instructions;
mod types;

// Re-export for internal use by resources module
pub(super) use imports::{SCRATCH_KEY_OFFSET, SCRATCH_VALUE_OFFSET};

/// Kind of value on the Move/WASM operand stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    I32,
    I64,
}

impl ValueKind {
    pub fn val_type(self) -> ValType {
        match self {
            ValueKind::I32 => ValType::I32,
            ValueKind::I64 => ValType::I64,
        }
    }
}

/// Import indices for generated helper syscalls
#[derive(Debug, Default)]
pub struct ImportLayout {
    pub storage_get: Option<u32>,
    pub storage_put: Option<u32>,
    pub storage_delete: Option<u32>,
}

/// Translate a Move module to WASM bytes
///
/// This is the main entry point for Move → WASM translation.
pub fn translate_to_wasm(module: &MoveModule) -> Result<Vec<u8>> {
    super::analysis::validate_supported_module(module)?;

    let needs_storage = module
        .functions
        .iter()
        .any(|f| f.code.iter().any(super::resources::is_resource_opcode));

    // Build type section
    let (types, func_type_indices, _next_type_index) = types::build_types(module, needs_storage)?;

    // Build import section
    let (imports, import_layout, imported_functions) = imports::build_imports(needs_storage)?;

    // Build function and code sections
    let (functions, code) = functions::build_functions(
        module,
        &func_type_indices,
        imported_functions,
        &import_layout,
        needs_storage,
    )?;

    // Build export section and memory
    let (exports, memory) = exports::build_exports(module, imported_functions, needs_storage)?;

    // Assemble module
    let mut module_bytes = Module::new();
    module_bytes.section(&types);
    if imported_functions > 0 {
        module_bytes.section(&imports);
    }
    module_bytes.section(&functions);
    if let Some(mem) = memory {
        module_bytes.section(&mem);
    }
    module_bytes.section(&exports);
    module_bytes.section(&code);

    Ok(module_bytes.finish())
}
