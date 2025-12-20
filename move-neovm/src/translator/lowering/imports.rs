//! Import section generation for storage syscalls
//!
//! This module handles the generation of WASM imports for Neo storage operations:
//! - storage_get: Read from contract storage
//! - storage_put: Write to contract storage
//! - storage_delete: Delete from contract storage

use anyhow::Result;
use wasm_encoder::{EntityType, ImportSection};

use super::ImportLayout;

/// Memory offsets for resource key/value scratch space
pub const SCRATCH_KEY_OFFSET: i32 = 0;
pub const SCRATCH_VALUE_OFFSET: i32 = 16;
pub const SCRATCH_KEY_SIZE: i32 = 16;
pub const SCRATCH_VALUE_SIZE: i32 = 8;

/// Build the import section for storage syscalls
///
/// The type indices for storage syscalls are assumed to be 0, 1, 2
/// (defined in types.rs when needs_storage is true).
///
/// Returns:
/// - ImportSection: The complete import section
/// - ImportLayout: Mapping of import names to function indices
/// - u32: Number of imported functions
pub fn build_imports(needs_storage: bool) -> Result<(ImportSection, ImportLayout, u32)> {
    let mut imports = ImportSection::new();
    let mut import_layout = ImportLayout::default();
    let mut imported_functions = 0u32;

    if needs_storage {
        // storage_get: type index 0
        imports.import("neo", "storage_get", EntityType::Function(0));
        import_layout.storage_get = Some(imported_functions);
        imported_functions += 1;

        // storage_put: type index 1
        imports.import("neo", "storage_put", EntityType::Function(1));
        import_layout.storage_put = Some(imported_functions);
        imported_functions += 1;

        // storage_delete: type index 2
        imports.import("neo", "storage_delete", EntityType::Function(2));
        import_layout.storage_delete = Some(imported_functions);
        imported_functions += 1;
    }

    Ok((imports, import_layout, imported_functions))
}
