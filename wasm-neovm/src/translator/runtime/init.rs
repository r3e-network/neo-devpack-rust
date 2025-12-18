use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_runtime_init_helper(
    script: &mut Vec<u8>,
    static_slot_count: usize,
    config: &MemoryConfig,
    globals: &[GlobalLayout],
    tables: &[TableLayout<'_>],
    passive_segments: &[PassiveSegmentLayout<'_>],
    active_segments: &[ActiveSegmentLayout<'_>],
    passive_elements: &[PassiveElementLayout<'_>],
    start: Option<&StartHelper<'_>>,
    imports: &[FunctionImport],
    types: &[FuncType],
    adapter: &dyn ChainAdapter,
) -> Result<Option<usize>> {
    let try_pos = emit_try_placeholder(script)?;
    if static_slot_count > u8::MAX as usize {
        bail!("too many static slots required for runtime initialisation");
    }

    script.push(lookup_opcode("INITSSLOT")?.byte);
    script.push(static_slot_count as u8);

    let initial_bytes = (config.initial_pages as i128) * 65_536i128;
    if initial_bytes == 0 {
        script.push(lookup_opcode("PUSH0")?.byte);
    } else {
        let _ = emit_push_int(script, initial_bytes);
    }
    script.push(lookup_opcode("NEWBUFFER")?.byte);
    script.push(lookup_opcode("STSFLD0")?.byte);

    if initial_bytes == 0 {
        script.push(lookup_opcode("PUSH0")?.byte);
    } else {
        let _ = emit_push_int(script, initial_bytes);
    }
    script.push(lookup_opcode("STSFLD1")?.byte);

    if config.initial_pages == 0 {
        script.push(lookup_opcode("PUSH0")?.byte);
    } else {
        let _ = emit_push_int(script, config.initial_pages as i128);
    }
    script.push(lookup_opcode("STSFLD2")?.byte);

    match config.maximum_pages {
        Some(max) => {
            let _ = emit_push_int(script, max as i128);
        }
        None => {
            let _ = emit_push_int(script, -1);
        }
    }
    script.push(lookup_opcode("STSFLD3")?.byte);

    script.push(lookup_opcode("PUSH0")?.byte);
    emit_store_static(script, INIT_FLAG_SLOT)?;

    for table in tables {
        let len = table.entries.len();
        if len == 0 {
            script.push(lookup_opcode("NEWARRAY0")?.byte);
        } else {
            let _ = emit_push_int(script, len as i128);
            script.push(lookup_opcode("NEWARRAY")?.byte);
        }
        emit_store_static(script, table.slot)?;
        if len > 0 {
            emit_load_static(script, table.slot)?;
            for (idx, value) in table.entries.iter().enumerate() {
                script.push(lookup_opcode("DUP")?.byte);
                let _ = emit_push_int(script, idx as i128);
                let _ = emit_push_int(script, *value as i128);
                script.push(lookup_opcode("SETITEM")?.byte);
            }
            script.push(lookup_opcode("DROP")?.byte);
        }
    }

    for global in globals {
        let _ = emit_push_int(script, global.initial_value);
        emit_store_static(script, global.slot)?;
    }

    for segment in passive_segments {
        emit_push_data(script, segment.bytes)?;
        emit_store_static(script, segment.byte_slot)?;
        script.push(lookup_opcode("PUSH0")?.byte);
        emit_store_static(script, segment.drop_slot)?;
    }

    for segment in active_segments {
        if segment.bytes.is_empty() {
            continue;
        }
        script.push(lookup_opcode("LDSFLD0")?.byte);
        let _ = emit_push_int(script, segment.offset as i128);
        emit_push_data(script, segment.bytes)?;
        script.push(lookup_opcode("PUSH0")?.byte);
        let _ = emit_push_int(script, segment.bytes.len() as i128);
        script.push(lookup_opcode("MEMCPY")?.byte);
    }

    for element in passive_elements {
        let len = element.values.len();
        if len == 0 {
            script.push(lookup_opcode("NEWARRAY0")?.byte);
        } else {
            let _ = emit_push_int(script, len as i128);
            script.push(lookup_opcode("NEWARRAY")?.byte);
        }
        emit_store_static(script, element.value_slot)?;
        if len > 0 {
            emit_load_static(script, element.value_slot)?;
            for (idx, value) in element.values.iter().enumerate() {
                script.push(lookup_opcode("DUP")?.byte);
                let _ = emit_push_int(script, idx as i128);
                let _ = emit_push_int(script, *value as i128);
                script.push(lookup_opcode("SETITEM")?.byte);
            }
            script.push(lookup_opcode("DROP")?.byte);
        }
        script.push(lookup_opcode("PUSH0")?.byte);
        emit_store_static(script, element.drop_slot)?;
    }

    let mut start_call_pos: Option<usize> = None;
    if let Some(start_helper) = start {
        let _ = emit_push_int(script, 1);
        emit_store_static(script, INIT_FLAG_SLOT)?;
        emit_load_static(script, start_helper.slot)?;
        let skip_start = emit_jump_placeholder(script, "JMPIF_L")?;

        match &start_helper.descriptor.kind {
            StartKind::Defined { offset } => {
                let call_pos = emit_call_placeholder(script)?;
                start_call_pos = Some(call_pos);
                patch_call(script, call_pos, *offset)?;
            }
            StartKind::Import => {
                let mut unused_features = FeatureTracker::default();
                handle_import_call(
                    start_helper.descriptor.function_index,
                    script,
                    imports,
                    types,
                    &[],
                    &mut unused_features,
                    adapter,
                )?;
            }
        }

        let _ = emit_push_int(script, 1);
        emit_store_static(script, start_helper.slot)?;

        let skip_label = script.len();
        patch_jump(script, skip_start, skip_label)?;
    }

    let endtry_pos = emit_endtry_placeholder(script)?;
    let skip_catch_jump = emit_jump_placeholder(script, "JMP_L")?;

    let catch_pos = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    let catch_endtry_pos = emit_endtry_placeholder(script)?;
    script.push(lookup_opcode("ABORT")?.byte);

    let end_label = script.len();

    patch_try_catch(script, try_pos, catch_pos)?;
    patch_endtry(script, endtry_pos, end_label)?;
    patch_endtry(script, catch_endtry_pos, end_label)?;
    patch_jump(script, skip_catch_jump, end_label)?;

    let _ = emit_push_int(script, 1);
    emit_store_static(script, INIT_FLAG_SLOT)?;

    script.push(RET);

    Ok(start_call_pos)
}
