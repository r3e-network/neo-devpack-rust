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

    let module = import.module.to_ascii_lowercase();
    if let Some(descriptor) = adapter.resolve_syscall(&import.module, &import.name) {
        let emitted = super::syscall::emit_descriptor_syscall(descriptor, script)?;
        features.register_syscall(emitted);
        return Ok(());
    }

    match module.as_str() {
        "opcode" => super::opcode::emit_opcode_call(import, func_type, params, script),
        "syscall" => {
            let descriptor = super::syscall::emit_syscall_call(import, script)?;
            features.register_syscall(descriptor);
            Ok(())
        }
        "neo" => {
            let descriptor = super::syscall::emit_neo_syscall(import, script)?;
            features.register_syscall(descriptor);
            Ok(())
        }
        other => {
            if adapter.recognizes_module(&import.module) {
                bail!(
                    "import module '{}' is recognized for {:?} but '{other}' could not be mapped",
                    import.module,
                    adapter.source_chain()
                );
            }
            bail!("unsupported import module '{}::{}'", other, import.name)
        }
    }
}
