// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

/// Extract type index from Import's TypeRef.
pub(in super::super) fn get_import_type_index(import: &FunctionImport) -> Result<u32> {
    Ok(import.type_index)
}

pub(crate) fn handle_import_call(
    function_index: u32,
    script: &mut Vec<u8>,
    imports: &[FunctionImport],
    types: &[FuncType],
    params: &[StackValue],
    features: &mut FeatureTracker,
    adapter: &dyn ChainAdapter,
) -> Result<()> {
    let import = imports
        .get(function_index as usize)
        .ok_or_else(|| anyhow!("calls to user-defined functions are not supported"))?;
    let type_index = get_import_type_index(import)?;
    let func_type = types.get(type_index as usize).ok_or_else(|| {
        anyhow!(
            "invalid type index {} for import {}",
            type_index,
            import.name
        )
    })?;

    // Zero-copy case-insensitive module matching (Round 69 optimization)
    // Avoids to_ascii_lowercase() allocation by using eq_ignore_ascii_case
    if let Some(descriptor) = adapter.resolve_syscall(&import.module, &import.name) {
        let emitted = super::syscall::emit_descriptor_syscall(descriptor, script)?;
        features.register_syscall(emitted);
        return Ok(());
    }

    // Fast path using case-insensitive comparison without allocation (Round 69)
    let module = &import.module;
    match () {
        _ if module.eq_ignore_ascii_case("opcode") => {
            super::opcode::emit_opcode_call(import, func_type, params, script)
        }
        _ if module.eq_ignore_ascii_case("syscall") => {
            let descriptor = super::syscall::emit_syscall_call(import, script)?;
            features.register_syscall(descriptor);
            Ok(())
        }
        _ if module.eq_ignore_ascii_case("neo") => {
            let descriptor = super::syscall::emit_neo_syscall(import, script)?;
            features.register_syscall(descriptor);
            Ok(())
        }
        _ => {
            if adapter.recognizes_module(&import.module) {
                bail!(
                    "import '{}::{}' is recognized for {:?} but could not be mapped",
                    import.module,
                    import.name,
                    adapter.source_chain()
                );
            }
            bail!("unsupported import module '{}::{}'", module, import.name)
        }
    }
}
