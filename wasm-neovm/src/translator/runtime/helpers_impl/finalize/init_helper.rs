// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::super::*;

impl RuntimeHelpers {
    pub(super) fn prepare_init_helper(
        &mut self,
        script: &mut Vec<u8>,
        start: Option<&StartDescriptor>,
        imports: &[FunctionImport],
        types: &[FuncType],
        adapter: &dyn ChainAdapter,
    ) -> Result<()> {
        if !self.data_segments.is_empty() && !self.memory_defined {
            bail!("data segments require a defined memory section");
        }

        let global_count = self.globals.len();
        let table_count = self.tables.len();
        let table_base = BASE_STATIC_SLOTS + global_count;

        for (idx, table) in self.tables.iter_mut().enumerate() {
            table.slot = Some(table_base + idx);
        }

        let data_base = table_base + table_count;
        let mut passive_indices: Vec<usize> = Vec::new();
        for (idx, segment) in self.data_segments.iter_mut().enumerate() {
            if let DataSegmentKind::Passive {
                byte_slot,
                drop_slot,
                ..
            } = &mut segment.kind
            {
                let order = passive_indices.len();
                *byte_slot = Some(data_base + order * 2);
                *drop_slot = Some(data_base + order * 2 + 1);
                passive_indices.push(idx);
            }
        }

        let passive_element_base = data_base + passive_indices.len() * 2;
        let mut passive_element_indices: Vec<usize> = Vec::new();
        for (idx, segment) in self.element_segments.iter_mut().enumerate() {
            if let ElementSegmentKind::Passive {
                value_slot,
                drop_slot,
            } = &mut segment.kind
            {
                let order = passive_element_indices.len();
                *value_slot = Some(passive_element_base + order * 2);
                *drop_slot = Some(passive_element_base + order * 2 + 1);
                passive_element_indices.push(idx);
            }
        }

        let passive_layout_vec: Vec<PassiveSegmentLayout<'_>> = passive_indices
            .iter()
            .map(|&idx| {
                let segment = &self.data_segments[idx];
                match &segment.kind {
                    DataSegmentKind::Passive {
                        byte_slot: Some(byte_slot),
                        drop_slot: Some(drop_slot),
                        ..
                    } => PassiveSegmentLayout {
                        bytes: &segment.bytes,
                        byte_slot: *byte_slot,
                        drop_slot: *drop_slot,
                    },
                    _ => unreachable!("passive slot assignment missing"),
                }
            })
            .collect();

        let active_layout_vec: Vec<ActiveSegmentLayout<'_>> = {
            let mut layouts = Vec::new();
            if !self.data_segments.is_empty() {
                let initial_bytes = (self.memory_config.initial_pages as u128) * 65_536u128;
                for segment in &self.data_segments {
                    if !segment.defined {
                        continue;
                    }
                    if let DataSegmentKind::Active { offset } = &segment.kind {
                        if (*offset as u128) + (segment.bytes.len() as u128) > initial_bytes {
                            bail!("active data segment exceeds initial memory size");
                        }
                        layouts.push(ActiveSegmentLayout {
                            offset: *offset,
                            bytes: &segment.bytes,
                        });
                    }
                }
            }
            layouts
        };

        let mut table_layouts: Vec<TableLayout<'_>> = Vec::with_capacity(self.tables.len());
        for (idx, table) in self.tables.iter().enumerate() {
            let slot = table.slot.ok_or_else(|| {
                anyhow::anyhow!("table {} missing assigned slot during finalize", idx)
            })?;
            table_layouts.push(TableLayout {
                slot,
                entries: &table.initial_entries,
            });
        }

        let passive_element_layouts: Vec<PassiveElementLayout<'_>> = passive_element_indices
            .iter()
            .map(|&idx| {
                let segment = &self.element_segments[idx];
                match &segment.kind {
                    ElementSegmentKind::Passive {
                        value_slot: Some(value_slot),
                        drop_slot: Some(drop_slot),
                    } => PassiveElementLayout {
                        values: &segment.values,
                        value_slot: *value_slot,
                        drop_slot: *drop_slot,
                    },
                    _ => unreachable!("passive element slot assignment missing"),
                }
            })
            .collect();

        for (idx, segment) in self.data_segments.iter().enumerate() {
            if !segment.defined {
                bail!("data segment {} referenced but not defined", idx);
            }
        }

        for (idx, segment) in self.element_segments.iter().enumerate() {
            if !segment.defined {
                bail!("element segment {} referenced but not defined", idx);
            }
        }

        let global_layouts: Vec<GlobalLayout> = self
            .globals
            .iter()
            .map(|g| GlobalLayout {
                slot: g.slot,
                initial_value: g.initial_value,
            })
            .collect();

        let start_helper = start.map(|descriptor| StartHelper {
            slot: BASE_STATIC_SLOTS
                + global_layouts.len()
                + table_layouts.len()
                + passive_layout_vec.len() * 2
                + passive_element_layouts.len() * 2,
            descriptor,
        });
        self.start_slot = start_helper.as_ref().map(|helper| helper.slot);

        let static_slot_count = BASE_STATIC_SLOTS
            + global_layouts.len()
            + table_layouts.len()
            + passive_layout_vec.len() * 2
            + passive_element_layouts.len() * 2
            + if start_helper.is_some() { 1 } else { 0 };

        let needs_init_helper = !self.memory_init_calls.is_empty()
            || start_helper.is_some()
            || self.runtime_state_requires_entry_init();
        if needs_init_helper {
            let offset = match self.memory_init_offset {
                Some(existing) => existing,
                None => {
                    let helper_offset = script.len();
                    let chunked_memory = self.uses_chunked_memory();
                    let start_call = emit_runtime_init_helper(
                        script,
                        static_slot_count,
                        self.memory_defined,
                        chunked_memory,
                        &self.memory_config,
                        &global_layouts,
                        &table_layouts,
                        &passive_layout_vec,
                        &active_layout_vec,
                        &passive_element_layouts,
                        start_helper.as_ref(),
                        imports,
                        types,
                        adapter,
                    )?;
                    if let Some(pos) = start_call {
                        self.start_call_positions.push(pos);
                    }
                    self.memory_init_offset = Some(helper_offset);
                    helper_offset
                }
            };

            for &call_pos in &self.memory_init_calls {
                patch_call(script, call_pos, offset)?;
            }
        }

        drop(passive_layout_vec);
        drop(active_layout_vec);
        drop(table_layouts);
        drop(passive_element_layouts);
        drop(global_layouts);

        Ok(())
    }
}
