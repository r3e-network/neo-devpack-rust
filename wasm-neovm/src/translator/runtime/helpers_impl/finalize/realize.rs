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
        // Pre-emit mask_u32 helper if memory/table helpers that USE mask_u32 are registered.
        // Load/Store/Grow don't use mask_u32, so only check Fill/Copy/Env* and table ops.
        let needs_mask_u32 = self.memory_helpers.keys().any(|k| matches!(k,
            MemoryHelperKind::Fill | MemoryHelperKind::Copy |
            MemoryHelperKind::EnvMemcpy | MemoryHelperKind::EnvMemmove | MemoryHelperKind::EnvMemset
        )) || self.table_helpers.keys().any(|k| matches!(k,
            TableHelperKind::Get(_) | TableHelperKind::Set(_) |
            TableHelperKind::Fill(_) | TableHelperKind::Copy { .. } |
            TableHelperKind::InitFromPassive { .. }
        ));
        let mask_u32_offset = if needs_mask_u32 {
            let offset = script.len();
            // mask_u32 body: (1 << 32) - 1, AND, RET
            let _ = emit_push_int(script, 1);
            let _ = emit_push_int(script, 32);
            script.push(lookup_opcode("SHL")?.byte);
            script.push(lookup_opcode("DEC")?.byte);
            script.push(lookup_opcode("AND")?.byte);
            script.push(lookup_opcode("RET")?.byte);
            Some(offset)
        } else {
            None
        };

        let memory_kinds: Vec<MemoryHelperKind> = self
            .memory_helpers
            .iter()
            .filter(|(_, record)| !record.calls.is_empty())
            .map(|(kind, _)| *kind)
            .collect();
        for kind in memory_kinds {
            let offset = self.realize_memory_helper(script, kind, mask_u32_offset)?;
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
            let offset = self.realize_table_helper(script, kind, mask_u32_offset)?;
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
                mask_u32_offset,
            )?;
            if let Some(record) = self.call_indirect_helpers.get_mut(&key) {
                for &call_pos in &record.calls {
                    patch_call(script, call_pos, offset)?;
                }
            }
        }

        // Realize param normalization + sign-extension helpers.
        // Param normalize falls through to sign-extend (no JMP needed) when
        // emitted immediately before it. The sign-extend entry point is also
        // used by arithmetic operations.
        for bits in [32u32, 64] {
            let has_normalize = if bits == 32 {
                !self.param_normalize_i32_helper.calls.is_empty()
            } else {
                !self.param_normalize_i64_helper.calls.is_empty()
            };
            let has_sign_ext = if bits == 32 {
                !self.sign_extend_32_helper.calls.is_empty()
            } else {
                !self.sign_extend_64_helper.calls.is_empty()
            };

            if has_normalize {
                // Emit param normalize body that falls through to sign-extend
                let normalize_offset = self.realize_param_normalize_inline(script, bits)?;
                let normalize_calls = if bits == 32 {
                    self.param_normalize_i32_helper.calls.clone()
                } else {
                    self.param_normalize_i64_helper.calls.clone()
                };
                for &call_pos in &normalize_calls {
                    patch_call(script, call_pos, normalize_offset)?;
                }

                // Sign-extend body immediately follows (param normalize falls through)
                let sign_ext_offset = self.realize_sign_extend_helper(script, bits)?;
                let sign_ext_calls = if bits == 32 {
                    self.sign_extend_32_helper.calls.clone()
                } else {
                    self.sign_extend_64_helper.calls.clone()
                };
                for &call_pos in &sign_ext_calls {
                    patch_call(script, call_pos, sign_ext_offset)?;
                }
            } else if has_sign_ext {
                // Only sign-extend needed (no param normalize)
                let offset = self.realize_sign_extend_helper(script, bits)?;
                let calls = if bits == 32 {
                    self.sign_extend_32_helper.calls.clone()
                } else {
                    self.sign_extend_64_helper.calls.clone()
                };
                for &call_pos in &calls {
                    patch_call(script, call_pos, offset)?;
                }
            }
        }

        Ok(())
    }

    /// Emit param normalize body that falls through to sign-extend (no JMP at end).
    /// Must be immediately followed by realize_sign_extend_helper.
    fn realize_param_normalize_inline(
        &mut self,
        script: &mut Vec<u8>,
        _bits: u32,
    ) -> Result<usize> {
        let offset = script.len();

        // DUP + ISNULL + JMPIFNOT (skip null path)
        script.push(lookup_opcode("DUP")?.byte);
        script.push(lookup_opcode("ISNULL")?.byte);
        let not_null_fixup = emit_jump_placeholder(script, "JMPIFNOT_L")?;

        // Null path: DROP, PUSH0, RET (0 is already sign-extended)
        script.push(lookup_opcode("DROP")?.byte);
        let _ = emit_push_int(script, 0);
        script.push(lookup_opcode("RET")?.byte);

        // Not-null label
        let not_null_label = script.len();
        patch_jump(script, not_null_fixup, not_null_label)?;

        // Convert to Integer unconditionally, then fall through to sign-extend
        script.push(lookup_opcode("CONVERT")?.byte);
        script.push(0x21); // StackItemType.Integer

        // No JMP or RET — execution falls through to the sign-extend body
        Ok(offset)
    }

    /// Emit the body for the sign-extension helper.
    /// Input: top-of-stack is the value to sign-extend.
    /// Output: top-of-stack is the sign-extended value.
    ///
    /// Uses an optimized sequence that computes sign_bit first, derives mask from it:
    ///   sign_bit = 1 << (bits-1)
    ///   mask = (sign_bit << 1) - 1
    ///   result = ((value AND mask) XOR sign_bit) - sign_bit
    /// This is 1 byte smaller than computing mask and sign_bit independently.
    fn realize_sign_extend_helper(
        &self,
        script: &mut Vec<u8>,
        bits: u32,
    ) -> Result<usize> {
        let offset = script.len();

        // Stack: [value]
        let _ = emit_push_int(script, 1);
        let _ = emit_push_int(script, (bits - 1) as i128);
        script.push(lookup_opcode("SHL")?.byte);         // [value, sign_bit]
        script.push(lookup_opcode("TUCK")?.byte);         // [sign_bit, value, sign_bit]
        script.push(lookup_opcode("DUP")?.byte);          // [sign_bit, value, sign_bit, sign_bit]
        let _ = emit_push_int(script, 1);
        script.push(lookup_opcode("SHL")?.byte);          // [sign_bit, value, sign_bit, mask+1]
        script.push(lookup_opcode("DEC")?.byte);          // [sign_bit, value, sign_bit, mask]
        script.push(lookup_opcode("ROT")?.byte);          // [sign_bit, sign_bit, mask, value]
        script.push(lookup_opcode("AND")?.byte);          // [sign_bit, sign_bit, masked]
        script.push(lookup_opcode("XOR")?.byte);          // [sign_bit, masked^sign]
        script.push(lookup_opcode("SWAP")?.byte);         // [masked^sign, sign_bit]
        script.push(lookup_opcode("SUB")?.byte);          // [result]

        script.push(lookup_opcode("RET")?.byte);

        Ok(offset)
    }

    fn realize_memory_helper(
        &mut self,
        script: &mut Vec<u8>,
        kind: MemoryHelperKind,
        mask_u32_offset: Option<usize>,
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
            MemoryHelperKind::Fill => emit_memory_fill_helper(script, mask_u32_offset)?,
            MemoryHelperKind::Copy => emit_memory_copy_helper(script, mask_u32_offset)?,
            MemoryHelperKind::EnvMemcpy => emit_env_memcpy_helper(script, mask_u32_offset)?,
            MemoryHelperKind::EnvMemmove => emit_env_memmove_helper(script, mask_u32_offset)?,
            MemoryHelperKind::EnvMemset => emit_env_memset_helper(script, mask_u32_offset)?,
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
        mask_u32_offset: Option<usize>,
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
                emit_table_get_helper(script, slot, mask_u32_offset)?;
            }
            TableHelperKind::Set(table) => {
                let slot = self.table_slot(table)?;
                emit_table_set_helper(script, slot, mask_u32_offset)?;
            }
            TableHelperKind::Size(table) => {
                let slot = self.table_slot(table)?;
                emit_table_size_helper(script, slot)?;
            }
            TableHelperKind::Fill(table) => {
                let slot = self.table_slot(table)?;
                emit_table_fill_helper(script, slot, mask_u32_offset)?;
            }
            TableHelperKind::Grow(table) => {
                let slot = self.table_slot(table)?;
                let maximum = self.table_descriptor_const(table)?.maximum;
                emit_table_grow_helper(script, slot, maximum)?;
            }
            TableHelperKind::Copy { dst, src } => {
                let dst_slot = self.table_slot(dst)?;
                let src_slot = self.table_slot(src)?;
                emit_table_copy_helper(script, dst_slot, src_slot, mask_u32_offset)?;
            }
            TableHelperKind::InitFromPassive { table, segment } => {
                let slot = self.table_slot(table)?;
                let (value_slot, drop_slot) = self.passive_element_slots_const(segment)?;
                emit_table_init_from_passive_helper(script, slot, value_slot, drop_slot, mask_u32_offset)?;
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
        mask_u32_offset: Option<usize>,
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
            self.realize_table_helper(script, TableHelperKind::Get(key.table_index), mask_u32_offset)?;
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
                pending_sign_extend: None,
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
