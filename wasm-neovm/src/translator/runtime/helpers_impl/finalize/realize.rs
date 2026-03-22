// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::super::*;

impl RuntimeHelpers {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn realize_helper_calls(
        &mut self,
        script: &mut Vec<u8>,
        imports: &[FunctionImport],
        types: &[FuncType],
        func_type_indices: &[u32],
        mut functions: Option<&mut FunctionRegistry>,
        features: &mut FeatureTracker,
        adapter: &dyn ChainAdapter,
    ) -> Result<()> {
        let memory_kinds: Vec<MemoryHelperKind> = self
            .memory_helpers
            .iter()
            .filter(|(_, record)| !record.calls.is_empty())
            .map(|(kind, _)| *kind)
            .collect();
        for kind in memory_kinds {
            let offset = self.realize_memory_helper(script, kind)?;
            if let Some(record) = self.memory_helpers.get_mut(&kind) {
                for &call_pos in &record.calls {
                    patch_call(script, call_pos, offset)?;
                }
            }
        }

        let bit_kinds: Vec<BitHelperKind> = self
            .bit_helpers
            .iter()
            .filter(|(_, record)| !record.calls.is_empty())
            .map(|(kind, _)| *kind)
            .collect();
        for kind in bit_kinds {
            let offset = self.realize_bit_helper(script, kind)?;
            if let Some(record) = self.bit_helpers.get_mut(&kind) {
                for &call_pos in &record.calls {
                    patch_call(script, call_pos, offset)?;
                }
            }
        }

        let table_kinds: Vec<TableHelperKind> = self
            .table_helpers
            .iter()
            .filter(|(_, record)| !record.calls.is_empty())
            .map(|(kind, _)| *kind)
            .collect();
        for kind in table_kinds {
            let offset = self.realize_table_helper(script, kind)?;
            if let Some(record) = self.table_helpers.get_mut(&kind) {
                for &call_pos in &record.calls {
                    patch_call(script, call_pos, offset)?;
                }
            }
        }

        let call_indirect_keys: Vec<CallIndirectHelperKey> = self
            .call_indirect_helpers
            .iter()
            .filter(|(_, record)| !record.calls.is_empty())
            .map(|(key, _)| *key)
            .collect();
        if !call_indirect_keys.is_empty() && functions.is_none() {
            bail!("call_indirect helpers requested without function registry");
        }
        for key in call_indirect_keys {
            let functions = functions.as_deref_mut().ok_or_else(|| {
                anyhow!("call_indirect helpers requested without function registry")
            })?;
            let offset = self.realize_call_indirect_helper(
                script,
                key,
                imports,
                types,
                func_type_indices,
                functions,
                features,
                adapter,
            )?;
            if let Some(record) = self.call_indirect_helpers.get_mut(&key) {
                for &call_pos in &record.calls {
                    patch_call(script, call_pos, offset)?;
                }
            }
        }

        Ok(())
    }

    fn realize_memory_helper(
        &mut self,
        script: &mut Vec<u8>,
        kind: MemoryHelperKind,
    ) -> Result<usize> {
        let record = self.memory_helpers.entry(kind).or_default();
        if let Some(offset) = record.offset {
            return Ok(offset);
        }
        let helper_offset = script.len();
        match kind {
            MemoryHelperKind::Load(bytes) => emit_memory_load_helper(script, bytes)?,
            MemoryHelperKind::Store(bytes) => emit_memory_store_helper(script, bytes)?,
            MemoryHelperKind::Grow => emit_memory_grow_helper(script, &self.memory_config)?,
            MemoryHelperKind::Fill => emit_memory_fill_helper(script)?,
            MemoryHelperKind::Copy => emit_memory_copy_helper(script)?,
            MemoryHelperKind::EnvMemcpy => emit_env_memcpy_helper(script)?,
            MemoryHelperKind::EnvMemmove => emit_env_memmove_helper(script)?,
            MemoryHelperKind::EnvMemset => emit_env_memset_helper(script)?,
        }
        record.offset = Some(helper_offset);
        Ok(helper_offset)
    }

    fn realize_bit_helper(&mut self, script: &mut Vec<u8>, kind: BitHelperKind) -> Result<usize> {
        let record = self.bit_helpers.entry(kind).or_default();
        if let Some(offset) = record.offset {
            return Ok(offset);
        }
        let helper_offset = script.len();
        match kind {
            BitHelperKind::Clz(bits) => emit_clz_helper(script, bits)?,
            BitHelperKind::Ctz(bits) => emit_ctz_helper(script, bits)?,
            BitHelperKind::Popcnt(bits) => emit_popcnt_helper(script, bits)?,
        }
        record.offset = Some(helper_offset);
        Ok(helper_offset)
    }

    fn realize_table_helper(
        &mut self,
        script: &mut Vec<u8>,
        kind: TableHelperKind,
    ) -> Result<usize> {
        if let Some(record) = self.table_helpers.get(&kind) {
            if let Some(offset) = record.offset {
                return Ok(offset);
            }
        }

        let helper_offset = script.len();
        match kind {
            TableHelperKind::Get(table) => {
                let slot = self.table_slot(table)?;
                emit_table_get_helper(script, slot)?;
            }
            TableHelperKind::Set(table) => {
                let slot = self.table_slot(table)?;
                emit_table_set_helper(script, slot)?;
            }
            TableHelperKind::Size(table) => {
                let slot = self.table_slot(table)?;
                emit_table_size_helper(script, slot)?;
            }
            TableHelperKind::Fill(table) => {
                let slot = self.table_slot(table)?;
                emit_table_fill_helper(script, slot)?;
            }
            TableHelperKind::Grow(table) => {
                let slot = self.table_slot(table)?;
                let maximum = self.table_descriptor_const(table)?.maximum;
                emit_table_grow_helper(script, slot, maximum)?;
            }
            TableHelperKind::Copy { dst, src } => {
                let dst_slot = self.table_slot(dst)?;
                let src_slot = self.table_slot(src)?;
                emit_table_copy_helper(script, dst_slot, src_slot)?;
            }
            TableHelperKind::InitFromPassive { table, segment } => {
                let slot = self.table_slot(table)?;
                let (value_slot, drop_slot) = self.passive_element_slots_const(segment)?;
                emit_table_init_from_passive_helper(script, slot, value_slot, drop_slot)?;
            }
            TableHelperKind::ElemDrop(segment) => {
                let drop_slot = self.passive_element_drop_slot_const(segment)?;
                emit_elem_drop_helper(script, drop_slot)?;
            }
        }

        if let Some(record) = self.table_helpers.get_mut(&kind) {
            record.offset = Some(helper_offset);
        } else {
            self.table_helpers.insert(
                kind,
                HelperRecord {
                    offset: Some(helper_offset),
                    calls: Vec::new(),
                },
            );
        }
        Ok(helper_offset)
    }

    #[allow(clippy::too_many_arguments)]
    fn realize_call_indirect_helper(
        &mut self,
        script: &mut Vec<u8>,
        key: CallIndirectHelperKey,
        imports: &[FunctionImport],
        types: &[FuncType],
        func_type_indices: &[u32],
        functions: &mut FunctionRegistry,
        features: &mut FeatureTracker,
        adapter: &dyn ChainAdapter,
    ) -> Result<usize> {
        if let Some(record) = self.call_indirect_helpers.get(&key) {
            if let Some(offset) = record.offset {
                return Ok(offset);
            }
        }

        types.get(key.type_index as usize).ok_or_else(|| {
            anyhow!(
                "type index {} out of bounds for call_indirect helper",
                key.type_index
            )
        })?;

        let table_helper_offset =
            self.realize_table_helper(script, TableHelperKind::Get(key.table_index))?;
        let helper_offset = script.len();

        let table_get_call = emit_call_placeholder(script)?;
        patch_call(script, table_get_call, table_helper_offset)?;

        script.push(lookup_opcode("DUP")?.byte);
        let _ = emit_push_int(script, FUNCREF_NULL);
        script.push(lookup_opcode("EQUAL")?.byte);
        let trap_null = emit_jump_placeholder(script, "JMPIF_L")?;

        let total_functions = imports.len() + func_type_indices.len();
        let candidate_functions = self.call_indirect_candidates(key.table_index)?;

        let estimated_matches = candidate_functions.len().min(32);
        let mut case_fixups: Vec<(usize, CallTarget)> = Vec::with_capacity(estimated_matches);
        for fn_index_u32 in candidate_functions {
            let fn_index = fn_index_u32 as usize;
            if fn_index >= total_functions {
                bail!(
                    "call_indirect candidate function {} out of range (total functions: {})",
                    fn_index,
                    total_functions
                );
            }

            let candidate_type_index = if fn_index < imports.len() {
                imports[fn_index].type_index
            } else {
                let defined_index = fn_index - imports.len();
                *func_type_indices.get(defined_index).ok_or_else(|| {
                    anyhow!(
                        "call_indirect target function {} missing type entry",
                        fn_index
                    )
                })?
            };

            if candidate_type_index != key.type_index {
                continue;
            }

            script.push(lookup_opcode("DUP")?.byte);
            let _ = emit_push_int(script, fn_index as i128);
            script.push(lookup_opcode("EQUAL")?.byte);
            let jump = emit_jump_placeholder(script, "JMPIF_L")?;

            let target = if fn_index < imports.len() {
                CallTarget::Import(fn_index as u32)
            } else {
                CallTarget::Defined(fn_index)
            };
            case_fixups.push((jump, target));
        }

        let trap_label = script.len();
        script.push(lookup_opcode("DROP")?.byte);
        script.push(lookup_opcode("ABORT")?.byte);
        patch_jump(script, trap_null, trap_label)?;

        let helper_type = types.get(key.type_index as usize).ok_or_else(|| {
            anyhow!(
                "type index {} out of bounds for call_indirect helper",
                key.type_index
            )
        })?;
        let import_params = vec![
            StackValue {
                const_value: None,
                bytecode_start: None,
            };
            helper_type.params().len()
        ];

        let mut end_fixups: Vec<usize> = Vec::with_capacity(estimated_matches);
        for (jump, target) in case_fixups {
            let label = script.len();
            patch_jump(script, jump, label)?;
            script.push(lookup_opcode("DROP")?.byte);
            match target {
                CallTarget::Import(idx) => {
                    handle_import_call(
                        idx,
                        script,
                        imports,
                        types,
                        &import_params,
                        features,
                        adapter,
                    )?;
                }
                CallTarget::Defined(idx) => {
                    functions.emit_call(script, idx)?;
                }
            }
            let end_jump = emit_jump_placeholder(script, "JMP_L")?;
            end_fixups.push(end_jump);
        }

        let end_label = script.len();
        for fixup in end_fixups {
            patch_jump(script, fixup, end_label)?;
        }

        script.push(lookup_opcode("RET")?.byte);

        if let Some(record) = self.call_indirect_helpers.get_mut(&key) {
            record.offset = Some(helper_offset);
        } else {
            self.call_indirect_helpers.insert(
                key,
                HelperRecord {
                    offset: Some(helper_offset),
                    calls: Vec::new(),
                },
            );
        }

        Ok(helper_offset)
    }
}
