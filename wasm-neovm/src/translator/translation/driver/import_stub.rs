// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(super) fn emit_import_export_stub(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    imports: &[FunctionImport],
    types: &[FuncType],
    import_index: usize,
    features: &mut FeatureTracker,
    adapter: &dyn ChainAdapter,
) -> Result<String> {
    let import = imports
        .get(import_index)
        .ok_or_else(|| anyhow!("import index {} out of range", import_index))?;
    let type_index = get_import_type_index(import)?;
    let func_type = types.get(type_index as usize).ok_or_else(|| {
        anyhow!(
            "invalid type index {} for import {}::{}",
            type_index,
            import.module,
            import.name
        )
    })?;

    for ty in func_type.params() {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!(
                "import '{}::{}' exported with unsupported parameter type {:?}",
                import.module,
                import.name,
                other
            ),
        }
    }

    if func_type.results().len() > 1 {
        bail!(
            "import '{}::{}' exported with unsupported multi-value return",
            import.module,
            import.name
        );
    }

    let mut params_stack: Vec<StackValue> = Vec::with_capacity(func_type.params().len());
    for (idx, _) in func_type.params().iter().enumerate() {
        emit_load_arg(script, idx as u32)?;
        params_stack.push(StackValue {
            const_value: None,
            bytecode_start: None,
            pending_sign_extend: None,
        });
    }

    let mut synthetic_stack: Vec<StackValue> = Vec::new();
    if try_handle_env_import(
        import,
        func_type,
        &params_stack,
        runtime,
        script,
        &mut synthetic_stack,
    )? {
        script.push(RET);
        let return_kind = func_type
            .results()
            .first()
            .map(wasm_val_type_to_manifest)
            .transpose()?
            .unwrap_or_else(|| "Void".to_string());
        return Ok(return_kind);
    }

    handle_import_call(
        import_index as u32,
        script,
        imports,
        types,
        &params_stack,
        features,
        adapter,
    )?;

    script.push(RET);

    let return_kind = func_type
        .results()
        .first()
        .map(wasm_val_type_to_manifest)
        .transpose()?
        .unwrap_or_else(|| "Void".to_string());

    Ok(return_kind)
}
