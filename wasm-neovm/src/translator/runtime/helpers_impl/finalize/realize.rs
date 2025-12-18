use super::super::super::*;

impl RuntimeHelpers {
    pub(super) fn realize_helper_calls(&mut self, script: &mut Vec<u8>) -> Result<()> {
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
}
