use super::*;

pub(super) fn resolve_start_descriptor(
    start_function: Option<u32>,
    start_defined_offset: Option<usize>,
    frontend: &ModuleFrontend,
    feature_tracker: &mut FeatureTracker,
    adapter: &dyn ChainAdapter,
) -> Result<Option<StartDescriptor>> {
    let Some(start_idx) = start_function else {
        return Ok(None);
    };

    if (start_idx as usize) < frontend.import_len() {
        let import = frontend
            .imports()
            .get(start_idx as usize)
            .ok_or_else(|| anyhow!("start section references missing import {}", start_idx))?;
        let type_index = get_import_type_index(import)?;
        let func_type = frontend
            .module_types()
            .signature(type_index as usize)
            .ok_or_else(|| {
                anyhow!(
                    "invalid type index {} for start import {}::{}",
                    type_index,
                    import.module,
                    import.name
                )
            })?;
        if !func_type.params().is_empty() {
            bail!("start function must not take parameters");
        }
        if !func_type.results().is_empty() {
            bail!("start function must not return values");
        }
        register_import_features(adapter, import, feature_tracker)?;
        return Ok(Some(StartDescriptor {
            function_index: start_idx,
            kind: StartKind::Import,
        }));
    }

    let defined_index = (start_idx as usize)
        .checked_sub(frontend.import_len())
        .ok_or_else(|| anyhow!("start function index underflow"))?;
    let type_index = frontend
        .module_types()
        .defined_type_index(defined_index)
        .ok_or_else(|| anyhow!("no type index recorded for start function"))?;
    let func_type = frontend
        .module_types()
        .signature(type_index as usize)
        .ok_or_else(|| anyhow!("invalid type index {} for start function", type_index))?;
    if !func_type.params().is_empty() {
        bail!("start function must not take parameters");
    }
    if !func_type.results().is_empty() {
        bail!("start function must not return values");
    }
    let offset = start_defined_offset.ok_or_else(|| {
        anyhow!("failed to record offset for start function; ensure code section is present")
    })?;
    Ok(Some(StartDescriptor {
        function_index: start_idx,
        kind: StartKind::Defined { offset },
    }))
}

pub(super) fn append_start_stub(
    script: &mut Vec<u8>,
    init_helper_offset: usize,
    start_offset: Option<usize>,
    start_slot: usize,
) -> Result<usize> {
    let stub_offset = script.len();

    emit_load_static(script, INIT_FLAG_SLOT)?;
    let skip_init = emit_jump_placeholder(script, "JMPIF_L")?;
    let init_call = emit_call_placeholder(script)?;
    patch_call(script, init_call, init_helper_offset)?;
    let after_init = script.len();
    patch_jump(script, skip_init, after_init)?;

    emit_load_static(script, start_slot)?;
    let skip_start = emit_jump_placeholder(script, "JMPIF_L")?;

    if let Some(offset) = start_offset {
        let start_call = emit_call_placeholder(script)?;
        patch_call(script, start_call, offset)?;
    }
    let after_start = script.len();
    patch_jump(script, skip_start, after_start)?;
    let _ = emit_push_int(script, 1);
    emit_store_static(script, start_slot)?;

    script.push(RET);

    Ok(stub_offset)
}
