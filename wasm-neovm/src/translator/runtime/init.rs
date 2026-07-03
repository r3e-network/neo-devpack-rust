// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

#[allow(unused_imports)]
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_runtime_init_helper(
    script: &mut Vec<u8>,
    static_slot_count: usize,
    has_memory: bool,
    chunked_memory: bool,
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
    // Export entry stubs call this helper before translated bodies can read
    // static slots. Body-level guards may then use the init flag safely.
    if static_slot_count > u8::MAX as usize {
        bail!("too many static slots required for runtime initialisation");
    }

    script.push(op::INITSSLOT);
    script.push(static_slot_count as u8);

    // Only emit memory-related slot initialization when memory is actually declared.
    // This saves ~10 bytes for contracts that don't use linear memory.
    if has_memory {
        let initial_bytes = (config.initial_pages as i128) * 65_536i128;
        if chunked_memory {
            script.push(op::NEWARRAY0);
            for _ in 0..config.initial_pages {
                script.push(op::DUP);
                emit_chunked_new_page(script)?;
                script.push(op::APPEND);
            }
            script.push(op::STSFLD0);
            let _ = emit_push_int(script, initial_bytes);
        } else {
            if initial_bytes == 0 {
                script.push(op::PUSH0);
                script.push(op::NEWBUFFER);
                script.push(op::STSFLD0);
                script.push(op::PUSH0);
            } else {
                let _ = emit_push_int(script, initial_bytes);
                script.push(op::DUP); // reuse initial_bytes for STSFLD1
                script.push(op::NEWBUFFER);
                script.push(op::STSFLD0);
                // DUP'd value is still on stack for STSFLD1
            }
        }
        script.push(op::STSFLD1);

        if config.initial_pages == 0 {
            script.push(op::PUSH0);
        } else {
            let _ = emit_push_int(script, config.initial_pages as i128);
        }
        script.push(op::STSFLD2);

        match config.maximum_pages {
            Some(max) => {
                let _ = emit_push_int(script, max as i128);
            }
            None => {
                let _ = emit_push_int(script, -1);
            }
        }
        script.push(op::STSFLD3);
    }

    // NeoVM's INITSSLOT initializes all static slots to null, then the helper
    // materializes memory/table/global state and sets the init flag.

    for table in tables {
        let len = table.entries.len();
        if len == 0 {
            script.push(op::NEWARRAY0);
        } else {
            let _ = emit_push_int(script, len as i128);
            script.push(op::NEWARRAY);
        }
        emit_store_static(script, table.slot)?;
        if len > 0 {
            emit_load_static(script, table.slot)?;
            for (idx, value) in table.entries.iter().enumerate() {
                script.push(op::DUP);
                let _ = emit_push_int(script, idx as i128);
                let _ = emit_push_int(script, *value as i128);
                script.push(op::SETITEM);
            }
            script.push(op::DROP);
        }
    }

    for global in globals {
        let _ = emit_push_int(script, global.initial_value);
        emit_store_static(script, global.slot)?;
    }

    for segment in passive_segments {
        emit_push_data(script, segment.bytes)?;
        emit_store_static(script, segment.byte_slot)?;
        script.push(op::PUSH0);
        emit_store_static(script, segment.drop_slot)?;
    }

    for segment in active_segments {
        if segment.bytes.is_empty() {
            continue;
        }
        if has_memory && chunked_memory {
            emit_chunked_copy_literal_to_memory(script, segment.offset, segment.bytes)?;
        } else {
            script.push(op::LDSFLD0);
            let _ = emit_push_int(script, segment.offset as i128);
            emit_push_data(script, segment.bytes)?;
            script.push(op::PUSH0);
            let _ = emit_push_int(script, segment.bytes.len() as i128);
            script.push(op::MEMCPY);
        }
    }

    for element in passive_elements {
        let len = element.values.len();
        if len == 0 {
            script.push(op::NEWARRAY0);
        } else {
            let _ = emit_push_int(script, len as i128);
            script.push(op::NEWARRAY);
        }
        emit_store_static(script, element.value_slot)?;
        if len > 0 {
            emit_load_static(script, element.value_slot)?;
            for (idx, value) in element.values.iter().enumerate() {
                script.push(op::DUP);
                let _ = emit_push_int(script, idx as i128);
                let _ = emit_push_int(script, *value as i128);
                script.push(op::SETITEM);
            }
            script.push(op::DROP);
        }
        script.push(op::PUSH0);
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

    let _ = emit_push_int(script, 1);
    emit_store_static(script, INIT_FLAG_SLOT)?;

    script.push(RET);

    Ok(start_call_pos)
}
