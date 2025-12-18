//! Type section generation for WASM modules
//!
//! This module handles the generation of WASM type signatures for:
//! - Storage syscall imports (storage_get, storage_put, storage_delete)
//! - Move function signatures

use crate::bytecode::MoveModule;
use anyhow::Result;
use wasm_encoder::{TypeSection, ValType};

use super::super::analysis::val_type_from_tag;

/// Build the type section for the WASM module
///
/// Returns:
/// - TypeSection: The complete type section
/// - Vec<u32>: Type indices for each function in the module
/// - u32: Next available type index
pub fn build_types(
    module: &MoveModule,
    needs_storage: bool,
) -> Result<(TypeSection, Vec<u32>, u32)> {
    let mut types = TypeSection::new();
    let mut next_type_index = 0u32;

    // Storage syscall types (if needed)
    if needs_storage {
        // storage_get: (i32, i32) -> i64
        {
            let encoder = types.ty();
            encoder.function(vec![ValType::I32, ValType::I32], vec![ValType::I64]);
        }
        next_type_index += 1;

        // storage_put: (i32, i32, i32, i32) -> ()
        {
            let encoder = types.ty();
            encoder.function(
                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                vec![],
            );
        }
        next_type_index += 1;

        // storage_delete: (i32, i32) -> ()
        {
            let encoder = types.ty();
            encoder.function(vec![ValType::I32, ValType::I32], vec![]);
        }
        next_type_index += 1;
    }

    // Function signatures
    let mut func_type_indices = Vec::with_capacity(module.functions.len());
    for func in &module.functions {
        let params: Vec<_> = func.parameters.iter().map(val_type_from_tag).collect();
        let results: Vec<_> = func.returns.iter().map(val_type_from_tag).collect();
        {
            let encoder = types.ty();
            encoder.function(params, results);
        }
        func_type_indices.push(next_type_index);
        next_type_index += 1;
    }

    Ok((types, func_type_indices, next_type_index))
}
