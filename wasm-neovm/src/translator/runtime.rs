use std::collections::BTreeMap;

use anyhow::{anyhow, bail, Context, Result};
use wasmparser::{ConstExpr, FuncType, Operator, ValType};

use crate::nef::{MethodToken, HASH160_LENGTH};

const BASE_MEMORY_STATIC_SLOTS: usize = 4;
const INIT_FLAG_SLOT: usize = BASE_MEMORY_STATIC_SLOTS;
const BASE_STATIC_SLOTS: usize = INIT_FLAG_SLOT + 1;

use super::constants::*;
use super::helpers::*;
use super::translation::{emit_binary_op, handle_import_call};
use super::types::{FunctionImport, StackValue};

pub(crate) struct RuntimeHelpers {
    memory_init_offset: Option<usize>,
    memory_init_calls: Vec<usize>,
    memory_config: MemoryConfig,
    memory_defined: bool,
    memory_helpers: BTreeMap<MemoryHelperKind, HelperRecord>,
    bit_helpers: BTreeMap<BitHelperKind, HelperRecord>,
    table_helpers: BTreeMap<TableHelperKind, HelperRecord>,
    data_segments: Vec<DataSegmentInfo>,
    element_segments: Vec<ElementSegmentInfo>,
    next_data_index: usize,
    next_element_index: usize,
    globals: Vec<GlobalDescriptor>,
    tables: Vec<TableDescriptor>,
}

impl Default for RuntimeHelpers {
    fn default() -> Self {
        RuntimeHelpers {
            memory_init_offset: None,
            memory_init_calls: Vec::new(),
            memory_config: MemoryConfig::default(),
            memory_defined: false,
            memory_helpers: BTreeMap::new(),
            bit_helpers: BTreeMap::new(),
            table_helpers: BTreeMap::new(),
            data_segments: Vec::new(),
            element_segments: Vec::new(),
            next_data_index: 0,
            next_element_index: 0,
            globals: Vec::new(),
            tables: Vec::new(),
        }
    }
}

pub(crate) struct StartDescriptor {
    pub(crate) function_index: u32,
    pub(crate) kind: StartKind,
}

pub(crate) enum StartKind {
    Defined { offset: usize },
    Import,
}

pub(crate) struct StartHelper<'a> {
    slot: usize,
    descriptor: &'a StartDescriptor,
}

pub(crate) struct FunctionRegistry {
    offsets: Vec<Option<usize>>,
    fixups: Vec<Vec<usize>>,
}

impl FunctionRegistry {
    pub(crate) fn new(total_functions: usize) -> Self {
        FunctionRegistry {
            offsets: vec![None; total_functions],
            fixups: vec![Vec::new(); total_functions],
        }
    }

    pub(crate) fn register_offset(
        &mut self,
        script: &mut Vec<u8>,
        function_index: usize,
        offset: usize,
    ) -> Result<()> {
        let entry = self
            .offsets
            .get_mut(function_index)
            .ok_or_else(|| anyhow!("function index {} out of range", function_index))?;
        if entry.is_some() {
            bail!(
                "function index {} registered multiple times",
                function_index
            );
        }
        *entry = Some(offset);

        if let Some(pending) = self.fixups.get_mut(function_index) {
            for call_pos in pending.drain(..) {
                patch_call(script, call_pos, offset)?;
            }
        }
        Ok(())
    }

    pub(crate) fn contains_index(&self, function_index: usize) -> bool {
        function_index < self.offsets.len()
    }

    pub(crate) fn emit_call(&mut self, script: &mut Vec<u8>, function_index: usize) -> Result<()> {
        if self.offsets.get(function_index).is_none() {
            bail!("function index {} out of range", function_index);
        }
        let call_pos = emit_call_placeholder(script)?;
        if let Some(offset) = self.offsets[function_index] {
            patch_call(script, call_pos, offset)?;
        } else {
            self.fixups[function_index].push(call_pos);
        }
        Ok(())
    }
}

impl RuntimeHelpers {
    pub(crate) fn helper_record_mut(&mut self, kind: MemoryHelperKind) -> &mut HelperRecord {
        self.memory_helpers.entry(kind).or_default()
    }
    pub(crate) fn bit_helper_record_mut(&mut self, kind: BitHelperKind) -> &mut HelperRecord {
        self.bit_helpers.entry(kind).or_default()
    }
    pub(crate) fn table_helper_record_mut(&mut self, kind: TableHelperKind) -> &mut HelperRecord {
        self.table_helpers.entry(kind).or_default()
    }
    pub(crate) fn emit_memory_init_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        emit_load_static(script, INIT_FLAG_SLOT)?;
        let skip_call = emit_jump_placeholder(script, "JMPIF_L")?;
        let call_pos = emit_call_placeholder(script)?;
        patch_jump(script, skip_call, script.len())?;
        self.memory_init_calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn finalize(
        &mut self,
        script: &mut Vec<u8>,
        start: Option<&StartDescriptor>,
        imports: &[FunctionImport],
        types: &[FuncType],
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

        let table_layouts: Vec<TableLayout<'_>> = self
            .tables
            .iter()
            .map(|table| TableLayout {
                slot: table
                    .slot
                    .expect("table slot should be assigned during finalize"),
                entries: &table.initial_entries,
            })
            .collect();

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

        let static_slot_count = BASE_STATIC_SLOTS
            + global_layouts.len()
            + table_layouts.len()
            + passive_layout_vec.len() * 2
            + passive_element_layouts.len() * 2
            + if start_helper.is_some() { 1 } else { 0 };

        let needs_init_helper = !self.memory_init_calls.is_empty() || start_helper.is_some();
        if needs_init_helper {
            let offset = match self.memory_init_offset {
                Some(existing) => existing,
                None => {
                    let helper_offset = script.len();
                    emit_runtime_init_helper(
                        script,
                        static_slot_count,
                        &self.memory_config,
                        &global_layouts,
                        &table_layouts,
                        &passive_layout_vec,
                        &active_layout_vec,
                        &passive_element_layouts,
                        start_helper.as_ref(),
                        imports,
                        types,
                    )?;
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

        for (kind, record) in self.memory_helpers.iter_mut() {
            if record.calls.is_empty() {
                continue;
            }

            let offset = match record.offset {
                Some(existing) => existing,
                None => {
                    let helper_offset = script.len();
                    match kind {
                        MemoryHelperKind::Load(bytes) => {
                            emit_memory_load_helper(script, *bytes)?;
                        }
                        MemoryHelperKind::Store(bytes) => {
                            emit_memory_store_helper(script, *bytes)?;
                        }
                        MemoryHelperKind::Grow => {
                            emit_memory_grow_helper(script, &self.memory_config)?;
                        }
                        MemoryHelperKind::Fill => {
                            emit_memory_fill_helper(script)?;
                        }
                        MemoryHelperKind::Copy => {
                            emit_memory_copy_helper(script)?;
                        }
                        MemoryHelperKind::EnvMemcpy => {
                            emit_env_memcpy_helper(script)?;
                        }
                        MemoryHelperKind::EnvMemmove => {
                            emit_env_memmove_helper(script)?;
                        }
                        MemoryHelperKind::EnvMemset => {
                            emit_env_memset_helper(script)?;
                        }
                    }
                    record.offset = Some(helper_offset);
                    helper_offset
                }
            };

            for &call_pos in &record.calls {
                patch_call(script, call_pos, offset)?;
            }
        }

        for (kind, record) in self.bit_helpers.iter_mut() {
            if record.calls.is_empty() {
                continue;
            }

            let offset = match record.offset {
                Some(existing) => existing,
                None => {
                    let helper_offset = script.len();
                    match kind {
                        BitHelperKind::Clz(bits) => emit_clz_helper(script, *bits)?,
                        BitHelperKind::Ctz(bits) => emit_ctz_helper(script, *bits)?,
                        BitHelperKind::Popcnt(bits) => emit_popcnt_helper(script, *bits)?,
                    }
                    record.offset = Some(helper_offset);
                    helper_offset
                }
            };

            for &call_pos in &record.calls {
                patch_call(script, call_pos, offset)?;
            }
        }

        let table_helpers_to_emit: Vec<TableHelperKind> = self
            .table_helpers
            .iter()
            .filter(|(_, record)| record.offset.is_none() && !record.calls.is_empty())
            .map(|(kind, _)| *kind)
            .collect();

        for kind in table_helpers_to_emit {
            let helper_offset = script.len();
            match kind {
                TableHelperKind::Get(table)
                | TableHelperKind::Set(table)
                | TableHelperKind::Size(table)
                | TableHelperKind::Fill(table)
                | TableHelperKind::Grow(table) => {
                    let slot = self.table_slot(table)?;
                    match kind {
                        TableHelperKind::Get(_) => emit_table_get_helper(script, slot)?,
                        TableHelperKind::Set(_) => emit_table_set_helper(script, slot)?,
                        TableHelperKind::Size(_) => emit_table_size_helper(script, slot)?,
                        TableHelperKind::Fill(_) => emit_table_fill_helper(script, slot)?,
                        TableHelperKind::Grow(_) => {
                            let maximum = self.table_descriptor_const(table)?.maximum;
                            emit_table_grow_helper(script, slot, maximum)?;
                        }
                        _ => unreachable!(),
                    }
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
            }
        }

        for record in self.table_helpers.values_mut() {
            if record.calls.is_empty() {
                continue;
            }
            let offset = match record.offset {
                Some(existing) => existing,
                None => continue,
            };
            for &call_pos in &record.calls {
                patch_call(script, call_pos, offset)?;
            }
        }

        for (index, segment) in self.data_segments.iter_mut().enumerate() {
            if let DataSegmentKind::Passive {
                init_record,
                drop_record,
                byte_slot,
                drop_slot,
            } = &mut segment.kind
            {
                let byte_slot = byte_slot
                    .ok_or_else(|| anyhow!("passive segment {} missing byte slot", index))?;
                let drop_slot = drop_slot
                    .ok_or_else(|| anyhow!("passive segment {} missing drop slot", index))?;

                if !init_record.calls.is_empty() {
                    let helper_offset = match init_record.offset {
                        Some(existing) => existing,
                        None => {
                            let helper_offset = script.len();
                            emit_data_init_helper(
                                script,
                                byte_slot,
                                drop_slot,
                                segment.bytes.len(),
                            )?;
                            init_record.offset = Some(helper_offset);
                            helper_offset
                        }
                    };

                    for &call_pos in &init_record.calls {
                        patch_call(script, call_pos, helper_offset)?;
                    }
                }

                if !drop_record.calls.is_empty() {
                    let helper_offset = match drop_record.offset {
                        Some(existing) => existing,
                        None => {
                            let helper_offset = script.len();
                            emit_data_drop_helper(script, drop_slot)?;
                            drop_record.offset = Some(helper_offset);
                            helper_offset
                        }
                    };

                    for &call_pos in &drop_record.calls {
                        patch_call(script, call_pos, helper_offset)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn set_memory_config(
        &mut self,
        initial_pages: u32,
        maximum_pages: Option<u32>,
    ) -> Result<()> {
        if self.memory_defined {
            bail!(
                "multiple memories are not supported (NeoVM exposes a single linear memory; see docs/wasm-pipeline.md#9-unsupported-wasm-features)"
            );
        }
        if maximum_pages.map_or(false, |max| max < initial_pages) {
            bail!("memory maximum smaller than initial size is invalid");
        }
        self.memory_config = MemoryConfig {
            initial_pages,
            maximum_pages,
        };
        self.memory_defined = true;
        Ok(())
    }

    pub(crate) fn memory_defined(&self) -> bool {
        self.memory_defined
    }

    pub(crate) fn emit_memory_load_call(&mut self, script: &mut Vec<u8>, bytes: u32) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Load(bytes));
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_memory_store_call(
        &mut self,
        script: &mut Vec<u8>,
        bytes: u32,
    ) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Store(bytes));
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_memory_grow_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Grow);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_memory_fill_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Fill);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_memory_copy_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Copy);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_env_memcpy_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::EnvMemcpy);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_env_memmove_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::EnvMemmove);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_env_memset_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::EnvMemset);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_bit_helper(
        &mut self,
        script: &mut Vec<u8>,
        kind: BitHelperKind,
    ) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.bit_helper_record_mut(kind);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_table_helper(
        &mut self,
        script: &mut Vec<u8>,
        kind: TableHelperKind,
    ) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.table_helper_record_mut(kind);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_data_init_call(
        &mut self,
        script: &mut Vec<u8>,
        segment_index: u32,
    ) -> Result<()> {
        let idx = segment_index as usize;
        let segment = self.ensure_passive_segment(idx)?;
        if let DataSegmentKind::Passive { init_record, .. } = &mut segment.kind {
            let call_pos = emit_call_placeholder(script)?;
            init_record.calls.push(call_pos);
            Ok(())
        } else {
            bail!("memory.init references active data segment {}", idx)
        }
    }

    pub(crate) fn emit_data_drop_call(
        &mut self,
        script: &mut Vec<u8>,
        segment_index: u32,
    ) -> Result<()> {
        let idx = segment_index as usize;
        let segment = self.ensure_passive_segment(idx)?;
        if let DataSegmentKind::Passive { drop_record, .. } = &mut segment.kind {
            let call_pos = emit_call_placeholder(script)?;
            drop_record.calls.push(call_pos);
            Ok(())
        } else {
            bail!("data.drop references active data segment {}", idx)
        }
    }

    pub(crate) fn register_passive_segment(&mut self, bytes: Vec<u8>) -> Result<()> {
        let index = self.next_data_index;
        self.next_data_index += 1;
        let segment = self.ensure_passive_segment(index)?;
        segment.bytes = bytes;
        segment.defined = true;
        Ok(())
    }

    pub(crate) fn register_active_segment(
        &mut self,
        _memory: u32,
        offset: u64,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let index = self.next_data_index;
        self.next_data_index += 1;
        if self.data_segments.len() <= index {
            self.data_segments.push(DataSegmentInfo {
                bytes,
                kind: DataSegmentKind::Active { offset },
                defined: true,
            });
            return Ok(());
        }

        let segment = &mut self.data_segments[index];
        if segment.defined {
            bail!("data segment {} defined multiple times", index);
        }
        segment.bytes = bytes;
        segment.kind = DataSegmentKind::Active { offset };
        segment.defined = true;
        Ok(())
    }

    pub(crate) fn ensure_passive_segment(&mut self, index: usize) -> Result<&mut DataSegmentInfo> {
        while self.data_segments.len() <= index {
            self.data_segments.push(DataSegmentInfo {
                bytes: Vec::new(),
                kind: DataSegmentKind::Passive {
                    init_record: HelperRecord::default(),
                    drop_record: HelperRecord::default(),
                    byte_slot: None,
                    drop_slot: None,
                },
                defined: false,
            });
        }

        let segment = &mut self.data_segments[index];
        match &segment.kind {
            DataSegmentKind::Passive { .. } => Ok(segment),
            DataSegmentKind::Active { .. } => {
                bail!("data segment {} is active", index)
            }
        }
    }

    pub(crate) fn register_table(&mut self, initial_len: usize, maximum: Option<u32>) -> usize {
        let mut entries = Vec::with_capacity(initial_len);
        for _ in 0..initial_len {
            entries.push(FUNCREF_NULL as i32);
        }
        self.tables.push(TableDescriptor {
            initial_entries: entries,
            maximum: maximum.map(|m| m as usize),
            slot: None,
        });
        self.tables.len() - 1
    }

    pub(crate) fn table_descriptor_mut(&mut self, index: usize) -> Result<&mut TableDescriptor> {
        self.tables
            .get_mut(index)
            .ok_or_else(|| anyhow!("table index {} out of range", index))
    }

    pub(crate) fn table_descriptor_const(&self, index: usize) -> Result<&TableDescriptor> {
        self.tables
            .get(index)
            .ok_or_else(|| anyhow!("table index {} out of range", index))
    }

    pub(crate) fn passive_element_slots_const(&self, index: usize) -> Result<(usize, usize)> {
        let segment = self
            .element_segments
            .get(index)
            .ok_or_else(|| anyhow!("element segment {} out of range", index))?;
        match &segment.kind {
            ElementSegmentKind::Passive {
                value_slot: Some(value_slot),
                drop_slot: Some(drop_slot),
            } => Ok((*value_slot, *drop_slot)),
            ElementSegmentKind::Passive { .. } => {
                bail!("passive element segment {} missing slot assignment", index)
            }
            ElementSegmentKind::Active { .. } => {
                bail!("element segment {} is active", index)
            }
        }
    }

    pub(crate) fn passive_element_drop_slot_const(&self, index: usize) -> Result<usize> {
        let (_, drop_slot) = self.passive_element_slots_const(index)?;
        Ok(drop_slot)
    }

    pub(crate) fn table_slot(&mut self, index: usize) -> Result<usize> {
        let base = BASE_STATIC_SLOTS + self.globals.len();
        let table = self.table_descriptor_mut(index)?;
        if table.slot.is_none() {
            table.slot = Some(base + index);
        }
        Ok(table.slot.expect("table slot should be assigned"))
    }

    pub(crate) fn register_active_element(
        &mut self,
        table_index: usize,
        offset: usize,
        values: Vec<i32>,
    ) -> Result<usize> {
        let index = self.next_element_index;
        self.next_element_index += 1;

        if self.element_segments.len() <= index {
            self.element_segments.push(ElementSegmentInfo {
                values: values.clone(),
                kind: ElementSegmentKind::Active {
                    _table_index: table_index,
                    _offset: offset,
                },
                defined: true,
            });
        } else {
            let entry = &mut self.element_segments[index];
            if entry.defined {
                bail!("element segment {} defined multiple times", index);
            }
            entry.values = values.clone();
            entry.kind = ElementSegmentKind::Active {
                _table_index: table_index,
                _offset: offset,
            };
            entry.defined = true;
        }

        self.apply_active_element(table_index, offset, &values)?;
        Ok(index)
    }

    pub(crate) fn register_passive_element(&mut self, values: Vec<i32>) -> usize {
        let index = self.next_element_index;
        self.next_element_index += 1;

        if self.element_segments.len() <= index {
            self.element_segments.push(ElementSegmentInfo {
                values,
                kind: ElementSegmentKind::Passive {
                    value_slot: None,
                    drop_slot: None,
                },
                defined: true,
            });
        } else {
            let entry = &mut self.element_segments[index];
            entry.values = values;
            entry.kind = ElementSegmentKind::Passive {
                value_slot: None,
                drop_slot: None,
            };
            entry.defined = true;
        }

        index
    }

    pub(crate) fn ensure_passive_element(
        &mut self,
        index: usize,
    ) -> Result<&mut ElementSegmentInfo> {
        while self.element_segments.len() <= index {
            self.element_segments.push(ElementSegmentInfo {
                values: Vec::new(),
                kind: ElementSegmentKind::Passive {
                    value_slot: None,
                    drop_slot: None,
                },
                defined: false,
            });
        }

        let segment = &mut self.element_segments[index];
        match &segment.kind {
            ElementSegmentKind::Passive { .. } => Ok(segment),
            ElementSegmentKind::Active { .. } => {
                bail!("element segment {} is active", index)
            }
        }
    }

    pub(crate) fn apply_active_element(
        &mut self,
        table_index: usize,
        offset: usize,
        values: &[i32],
    ) -> Result<()> {
        let table = self.table_descriptor_mut(table_index)?;
        let end = offset
            .checked_add(values.len())
            .ok_or_else(|| anyhow!("element segment offset overflow"))?;
        if end > table.initial_entries.len() {
            bail!(
                "element segment writes past table bounds (offset {}, length {}, table size {})",
                offset,
                values.len(),
                table.initial_entries.len()
            );
        }
        table.initial_entries[offset..end].copy_from_slice(values);
        Ok(())
    }

    pub(crate) fn register_global(&mut self, mutable: bool, initial_value: i128) -> usize {
        let slot = BASE_STATIC_SLOTS + self.globals.len();
        let const_value = if mutable { None } else { Some(initial_value) };
        self.globals.push(GlobalDescriptor {
            slot,
            mutable,
            initial_value,
            const_value,
        });
        self.globals.len() - 1
    }

    pub(crate) fn global_slot(&self, index: usize) -> Result<usize> {
        self.globals
            .get(index)
            .map(|g| g.slot)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    pub(crate) fn global_mutable(&self, index: usize) -> Result<bool> {
        self.globals
            .get(index)
            .map(|g| g.mutable)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    pub(crate) fn global_const_value(&self, index: usize) -> Result<Option<i128>> {
        self.globals
            .get(index)
            .map(|g| g.const_value)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    pub(crate) fn clear_global_const(&mut self, index: usize) -> Result<()> {
        let global = self
            .globals
            .get_mut(index)
            .ok_or_else(|| anyhow!("global index {} out of range", index))?;
        global.const_value = None;
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct MemoryConfig {
    initial_pages: u32,
    maximum_pages: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MemoryHelperKind {
    Load(u32),
    Store(u32),
    Grow,
    Fill,
    Copy,
    EnvMemcpy,
    EnvMemmove,
    EnvMemset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BitHelperKind {
    Clz(u32),
    Ctz(u32),
    Popcnt(u32),
}

impl BitHelperKind {
    pub(crate) fn bits(self) -> u32 {
        match self {
            BitHelperKind::Clz(bits) | BitHelperKind::Ctz(bits) | BitHelperKind::Popcnt(bits) => {
                bits
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TableHelperKind {
    Get(usize),
    Set(usize),
    Size(usize),
    Grow(usize),
    Fill(usize),
    Copy { dst: usize, src: usize },
    InitFromPassive { table: usize, segment: usize },
    ElemDrop(usize),
}

#[derive(Clone, Copy)]
pub(crate) enum CallTarget {
    Import(u32),
    Defined(usize),
}

#[derive(Default)]
pub(crate) struct HelperRecord {
    offset: Option<usize>,
    calls: Vec<usize>,
}

pub(crate) struct DataSegmentInfo {
    bytes: Vec<u8>,
    kind: DataSegmentKind,
    defined: bool,
}

pub(crate) enum DataSegmentKind {
    Passive {
        init_record: HelperRecord,
        drop_record: HelperRecord,
        byte_slot: Option<usize>,
        drop_slot: Option<usize>,
    },
    Active {
        offset: u64,
    },
}

pub(crate) struct GlobalDescriptor {
    slot: usize,
    mutable: bool,
    initial_value: i128,
    const_value: Option<i128>,
}

pub(crate) struct TableDescriptor {
    initial_entries: Vec<i32>,
    maximum: Option<usize>,
    slot: Option<usize>,
}

pub(crate) enum ElementSegmentKind {
    Passive {
        value_slot: Option<usize>,
        drop_slot: Option<usize>,
    },
    Active {
        _table_index: usize,
        _offset: usize,
    },
}

pub(crate) struct ElementSegmentInfo {
    values: Vec<i32>,
    kind: ElementSegmentKind,
    defined: bool,
}

pub(crate) struct TableInfo {
    pub(crate) entries: Vec<Option<u32>>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            initial_pages: 0,
            maximum_pages: None,
        }
    }
}

struct PassiveSegmentLayout<'a> {
    bytes: &'a [u8],
    byte_slot: usize,
    drop_slot: usize,
}

struct ActiveSegmentLayout<'a> {
    offset: u64,
    bytes: &'a [u8],
}

struct GlobalLayout {
    slot: usize,
    initial_value: i128,
}

struct TableLayout<'a> {
    slot: usize,
    entries: &'a [i32],
}

struct PassiveElementLayout<'a> {
    values: &'a [i32],
    value_slot: usize,
    drop_slot: usize,
}

fn emit_runtime_init_helper(
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
) -> Result<()> {
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

    if let Some(start_helper) = start {
        emit_load_static(script, start_helper.slot)?;
        let skip_start = emit_jump_placeholder(script, "JMPIF_L")?;

        match &start_helper.descriptor.kind {
            StartKind::Defined { offset } => {
                let call_pos = emit_call_placeholder(script)?;
                patch_call(script, call_pos, *offset)?;
            }
            StartKind::Import => {
                handle_import_call(
                    start_helper.descriptor.function_index,
                    script,
                    imports,
                    types,
                    &[],
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

    let end_label = script.len();

    patch_try_catch(script, try_pos, catch_pos)?;
    patch_endtry(script, endtry_pos, end_label)?;
    patch_endtry(script, catch_endtry_pos, end_label)?;
    patch_jump(script, skip_catch_jump, end_label)?;

    let _ = emit_push_int(script, 1);
    emit_store_static(script, INIT_FLAG_SLOT)?;

    script.push(RET);

    Ok(())
}

pub(crate) fn ensure_memory_access(runtime: &RuntimeHelpers, mem_index: u32) -> Result<()> {
    if mem_index != 0 {
        bail!(
            "only default memory index 0 is supported (NeoVM exposes a single linear memory; see docs/wasm-pipeline.md#9-unsupported-wasm-features)"
        );
    }
    if !runtime.memory_defined() {
        bail!("memory instructions require a defined memory section");
    }
    Ok(())
}

pub(crate) fn evaluate_offset_expr(expr: ConstExpr<'_>) -> Result<i64> {
    let mut reader = expr.get_operators_reader();
    let mut offset: Option<i64> = None;
    while !reader.eof() {
        let op = reader.read()?;
        match op {
            Operator::I32Const { value } => offset = Some(value as i64),
            Operator::I64Const { value } => offset = Some(value),
            Operator::End => break,
            other => {
                bail!(
                    "unsupported instruction {:?} in data segment offset expression",
                    other
                );
            }
        }
    }

    offset.ok_or_else(|| anyhow!("data segment offset expression did not yield a constant"))
}

pub(crate) fn evaluate_global_init(expr: ConstExpr<'_>, value_type: ValType) -> Result<i128> {
    let mut reader = expr.get_operators_reader();
    let mut value: Option<i128> = None;
    while !reader.eof() {
        let op = reader.read()?;
        match op {
            Operator::I32Const { value: v } => {
                value = Some(v as i128);
            }
            Operator::I64Const { value: v } => {
                value = Some(v as i128);
            }
            Operator::End => break,
            other => {
                bail!(
                    "unsupported instruction {:?} in global initialiser expression",
                    other
                );
            }
        }
    }

    let result = value.ok_or_else(|| anyhow!("global initialiser did not yield a constant"))?;
    match value_type {
        ValType::I32 => Ok((result as i32) as i128),
        ValType::I64 => Ok((result as i64) as i128),
        other => bail!(
            "unsupported global value type {:?}; expected i32 or i64",
            other
        ),
    }
}

pub(crate) fn translate_memory_load(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
    base: StackValue,
    mem_index: u32,
    offset: u64,
    bytes: u32,
    sign_extend: Option<(u32, u32)>,
    result_bits: u32,
    context: &str,
) -> Result<()> {
    ensure_memory_access(runtime, mem_index)?;
    runtime.emit_memory_init_call(script)?;
    let _addr = apply_memory_offset(script, base, offset)
        .with_context(|| format!("failed to apply offset for {}", context))?;
    runtime
        .emit_memory_load_call(script, bytes)
        .with_context(|| format!("failed to emit helper call for {}", context))?;

    let mut raw_value = StackValue {
        const_value: None,
        bytecode_start: None,
    };

    let load_bits = bytes * 8;
    let result = if let Some((from_bits, to_bits)) = sign_extend {
        emit_sign_extend(script, raw_value, from_bits, to_bits)?
    } else {
        if result_bits < load_bits {
            bail!(
                "result bit-width {} smaller than load width {}",
                result_bits,
                load_bits
            );
        }
        if result_bits > load_bits {
            raw_value = emit_zero_extend(script, raw_value, load_bits)?;
        }
        raw_value
    };

    value_stack.push(result);
    Ok(())
}

fn apply_memory_offset(script: &mut Vec<u8>, base: StackValue, offset: u64) -> Result<StackValue> {
    if offset == 0 {
        return Ok(base);
    }
    let offset_value = emit_push_int(script, offset as i128);
    emit_binary_op(script, "ADD", base, offset_value, |a, b| Some(a + b))
}

pub(crate) fn translate_memory_store(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value: StackValue,
    address: StackValue,
    mem_index: u32,
    offset: u64,
    bytes: u32,
    context: &str,
) -> Result<()> {
    let _ = value;
    ensure_memory_access(runtime, mem_index)?;
    runtime.emit_memory_init_call(script)?;
    script.push(lookup_opcode("SWAP")?.byte);
    let _addr = apply_memory_offset(script, address, offset)
        .with_context(|| format!("failed to apply offset for {}", context))?;
    script.push(lookup_opcode("SWAP")?.byte);
    runtime
        .emit_memory_store_call(script, bytes)
        .with_context(|| format!("failed to emit helper call for {}", context))?;
    Ok(())
}

pub(crate) fn translate_memory_fill(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    _dest: StackValue,
    _value: StackValue,
    _len: StackValue,
    mem_index: u32,
) -> Result<()> {
    ensure_memory_access(runtime, mem_index)?;
    runtime.emit_memory_init_call(script)?;
    runtime
        .emit_memory_fill_call(script)
        .context("failed to emit helper call for memory.fill")?;
    Ok(())
}

pub(crate) fn translate_memory_copy(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    _dest: StackValue,
    _src: StackValue,
    _len: StackValue,
    dst_mem: u32,
    src_mem: u32,
) -> Result<()> {
    if dst_mem != 0 || src_mem != 0 {
        bail!("only default memory index 0 is supported for memory.copy");
    }
    ensure_memory_access(runtime, dst_mem)?;
    runtime.emit_memory_init_call(script)?;
    runtime
        .emit_memory_copy_call(script)
        .context("failed to emit helper call for memory.copy")?;
    Ok(())
}

pub(crate) fn translate_memory_init(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    _dest: StackValue,
    _src: StackValue,
    _len: StackValue,
    data_index: u32,
    mem_index: u32,
) -> Result<()> {
    ensure_memory_access(runtime, mem_index)?;
    runtime.emit_memory_init_call(script)?;
    runtime
        .emit_data_init_call(script, data_index)
        .context("failed to emit helper call for memory.init")?;
    Ok(())
}

pub(crate) fn translate_data_drop(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    segment_index: u32,
) -> Result<()> {
    runtime.emit_memory_init_call(script)?;
    runtime
        .emit_data_drop_call(script, segment_index)
        .context("failed to emit helper call for data.drop")?;
    Ok(())
}

fn emit_memory_load_helper(script: &mut Vec<u8>, bytes: u32) -> Result<()> {
    let bytes_i128 = bytes as i128;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, bytes_i128);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    let _ = emit_push_int(script, bytes_i128);
    script.push(lookup_opcode("SUBSTR")?.byte);
    script.push(CONVERT);
    script.push(STACKITEMTYPE_INTEGER);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;

    Ok(())
}

fn emit_memory_store_helper(script: &mut Vec<u8>, bytes: u32) -> Result<()> {
    let bytes_i128 = bytes as i128;

    script.push(lookup_opcode("SWAP")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, bytes_i128);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("SWAP")?.byte);
    let mask = (1i128 << (bytes * 8)) - 1;
    let _ = emit_push_int(script, mask);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);

    for i in 0..bytes {
        script.push(lookup_opcode("OVER")?.byte);
        let shift = (i * 8) as i128;
        let _ = emit_push_int(script, shift);
        script.push(lookup_opcode("SHR")?.byte);
        let _ = emit_push_int(script, 0xFF);
        script.push(lookup_opcode("AND")?.byte);
        script.push(lookup_opcode("OVER")?.byte);
        let _ = emit_push_int(script, i as i128);
        script.push(lookup_opcode("ADD")?.byte);
        script.push(lookup_opcode("SWAP")?.byte);
        script.push(lookup_opcode("LDSFLD0")?.byte);
        script.push(lookup_opcode("ROT")?.byte);
        script.push(lookup_opcode("SETITEM")?.byte);
    }

    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("DROP")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;

    Ok(())
}

fn emit_memory_grow_helper(script: &mut Vec<u8>, _config: &MemoryConfig) -> Result<()> {
    let mask = (1u128 << 32) - 1;
    let _ = emit_push_int(script, mask as i128);
    script.push(lookup_opcode("AND")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDSFLD2")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("OVER")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("ADD")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("LDSFLD3")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSHM1")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let skip_limit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("GT")?.byte);
    let fail_on_max = emit_jump_placeholder(script, "JMPIF_L")?;
    let after_limit = emit_jump_placeholder(script, "JMP_L")?;

    let skip_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("DROP")?.byte);
    let after_label = script.len();
    patch_jump(script, skip_limit, skip_label)?;
    patch_jump(script, after_limit, after_label)?;

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 16);
    script.push(lookup_opcode("SHL")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("NEWBUFFER")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("MEMCPY")?.byte);

    script.push(lookup_opcode("STSFLD0")?.byte);
    script.push(lookup_opcode("STSFLD1")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("STSFLD2")?.byte);
    script.push(lookup_opcode("DROP")?.byte);
    script.push(RET);

    let zero_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("LDSFLD2")?.byte);
    script.push(RET);

    let fail_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("PUSHM1")?.byte);
    script.push(RET);

    patch_jump(script, zero_branch, zero_label)?;
    patch_jump(script, fail_on_max, fail_label)?;
    Ok(())
}

fn emit_memory_fill_helper(script: &mut Vec<u8>) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dest_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    let _ = emit_push_int(script, 0xFF);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);

    let loop_start = script.len();

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let exit_jump = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC2")?.byte);

    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dest_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

fn emit_memory_copy_helper(script: &mut Vec<u8>) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dest_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_src_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("MEMCPY")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dest_negative, trap_label)?;
    patch_jump(script, trap_src_negative, trap_label)?;
    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    Ok(())
}

fn emit_env_memcpy_helper(script: &mut Vec<u8>) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dest_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_src_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("MEMCPY")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dest_negative, trap_label)?;
    patch_jump(script, trap_src_negative, trap_label)?;
    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    Ok(())
}

fn emit_env_memmove_helper(script: &mut Vec<u8>) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(5);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dest_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_src_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_len = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let forward_copy = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    let back_loop = script.len();
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let back_exit = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(lookup_opcode("STLOC4")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    let back_jump = emit_jump_placeholder(script, "JMP_L")?;

    let back_exit_label = script.len();
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(RET);

    patch_jump(script, back_exit, back_exit_label)?;
    patch_jump(script, back_jump, back_loop)?;

    let forward_label = script.len();
    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("MEMCPY")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(RET);

    let zero_label = script.len();
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dest_negative, trap_label)?;
    patch_jump(script, trap_src_negative, trap_label)?;
    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, zero_len, zero_label)?;
    patch_jump(script, forward_copy, forward_label)?;
    Ok(())
}

fn emit_env_memset_helper(script: &mut Vec<u8>) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dest_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    let _ = emit_push_int(script, 0xFF);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);

    let loop_start = script.len();

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let exit_jump = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC2")?.byte);

    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dest_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

fn emit_table_get_helper(script: &mut Vec<u8>, table_slot: usize) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(2);
    script.push(0);

    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC1")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;
    Ok(())
}

fn emit_table_set_helper(script: &mut Vec<u8>, table_slot: usize) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC0")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;
    Ok(())
}

fn emit_table_size_helper(script: &mut Vec<u8>, table_slot: usize) -> Result<()> {
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(RET);
    Ok(())
}

fn emit_table_grow_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
    maximum: Option<usize>,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(5);
    script.push(0);

    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    let mask = (1u128 << 32) - 1;
    let _ = emit_push_int(script, mask as i128);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    let exceed_jump = if let Some(maximum) = maximum {
        script.push(lookup_opcode("LDLOC3")?.byte);
        script.push(lookup_opcode("LDLOC1")?.byte);
        script.push(lookup_opcode("ADD")?.byte);
        let _ = emit_push_int(script, maximum as i128);
        script.push(lookup_opcode("GT")?.byte);
        let jump = emit_jump_placeholder(script, "JMPIF_L")?;
        script.push(lookup_opcode("DROP")?.byte);
        Some(jump)
    } else {
        None
    };

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("STLOC4")?.byte);

    let loop_start = script.len();
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let loop_exit = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("APPEND")?.byte);
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC4")?.byte);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(RET);
    patch_jump(script, loop_exit, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;

    let zero_label = script.len();
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;
    if let Some(exceed_jump) = exceed_jump {
        let fail_label = script.len();
        script.push(lookup_opcode("PUSHM1")?.byte);
        script.push(RET);
        patch_jump(script, exceed_jump, fail_label)?;
    }
    Ok(())
}

fn emit_table_fill_helper(script: &mut Vec<u8>, table_slot: usize) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(5);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC3")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dest_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("STLOC4")?.byte);

    let loop_start = script.len();
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let loop_exit = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC4")?.byte);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    script.push(RET);
    patch_jump(script, loop_exit, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;

    let zero_label = script.len();
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dest_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;
    Ok(())
}

fn emit_table_copy_helper(script: &mut Vec<u8>, dst_slot: usize, src_slot: usize) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(7);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, dst_slot)?;
    script.push(lookup_opcode("STLOC3")?.byte);
    emit_load_static(script, src_slot)?;
    script.push(lookup_opcode("STLOC4")?.byte);
    script.push(lookup_opcode("NEWARRAY0")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dst_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_src_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dst_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    let collect_start = script.len();
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let collect_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(lookup_opcode("LDLOC5")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(lookup_opcode("APPEND")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);

    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);
    let collect_back = emit_jump_placeholder(script, "JMP_L")?;
    let collect_done = script.len();
    patch_jump(script, collect_exit, collect_done)?;
    patch_jump(script, collect_back, collect_start)?;

    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);

    let store_start = script.len();
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let store_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC5")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);
    let store_back = emit_jump_placeholder(script, "JMP_L")?;
    let store_done = script.len();
    patch_jump(script, store_exit, store_done)?;
    patch_jump(script, store_back, store_start)?;

    let zero_label = script.len();
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dst_negative, trap_label)?;
    patch_jump(script, trap_src_negative, trap_label)?;
    patch_jump(script, trap_dst_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    Ok(())
}

fn emit_table_init_from_passive_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
    value_slot: usize,
    drop_slot: usize,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(7);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC3")?.byte);
    emit_load_static(script, value_slot)?;
    script.push(lookup_opcode("STLOC4")?.byte);
    emit_load_static(script, drop_slot)?;
    script.push(lookup_opcode("STLOC5")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);

    script.push(lookup_opcode("LDLOC5")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let trap_dropped = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dst_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_src_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dst_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    let loop_start = script.len();
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let loop_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;
    let loop_done = script.len();
    patch_jump(script, loop_exit, loop_done)?;
    patch_jump(script, loop_back, loop_start)?;

    let zero_label = script.len();
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_dropped, trap_label)?;
    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dst_negative, trap_label)?;
    patch_jump(script, trap_src_negative, trap_label)?;
    patch_jump(script, trap_dst_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    Ok(())
}

fn emit_elem_drop_helper(script: &mut Vec<u8>, drop_slot: usize) -> Result<()> {
    let _ = emit_push_int(script, 1);
    emit_store_static(script, drop_slot)?;
    script.push(RET);
    Ok(())
}

pub(crate) fn infer_contract_tokens(script: &[u8]) -> Vec<MethodToken> {
    use crate::opcodes;
    use crate::syscalls;

    #[derive(Debug, Clone)]
    enum Literal {
        Integer(i128),
        Bytes(Vec<u8>),
        Array(usize),
        Unknown,
    }

    let mut tokens = Vec::new();

    let get_byte = |name: &str| -> Option<u8> { opcodes::lookup(name).map(|info| info.byte) };

    let pushint8 = get_byte("PUSHINT8");
    let pushint16 = get_byte("PUSHINT16");
    let pushint32 = get_byte("PUSHINT32");
    let pushint64 = get_byte("PUSHINT64");
    let pushint128 = get_byte("PUSHINT128");
    let pushm1 = get_byte("PUSHM1");
    let push0 = get_byte("PUSH0");
    let pushdata1 = get_byte("PUSHDATA1");
    let pushdata2 = get_byte("PUSHDATA2");
    let pushdata4 = get_byte("PUSHDATA4");
    let newarray0 = get_byte("NEWARRAY0");
    let newarray = get_byte("NEWARRAY");
    let pack = get_byte("PACK");
    let drop_op = get_byte("DROP");
    let syscall = get_byte("SYSCALL");
    let ret = get_byte("RET");

    if syscall.is_none() {
        return tokens;
    }

    let mut stack: Vec<Literal> = Vec::new();
    let mut pc = 0usize;
    while pc < script.len() {
        let op = script[pc];
        pc += 1;

        let mut cleared = false;
        let literal = if Some(op) == pushm1 {
            Some(Literal::Integer(-1))
        } else if let Some(p0) = push0 {
            if op >= p0 && op <= p0 + 16 {
                Some(Literal::Integer((op - p0) as i128))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(lit) = literal {
            stack.push(lit);
            continue;
        }

        if Some(op) == pushint8 {
            if pc + 1 > script.len() {
                break;
            }
            let value = i8::from_le_bytes([script[pc]]);
            pc += 1;
            stack.push(Literal::Integer(value.into()));
            continue;
        }
        if Some(op) == pushint16 {
            if pc + 2 > script.len() {
                break;
            }
            let value = i16::from_le_bytes([script[pc], script[pc + 1]]);
            pc += 2;
            stack.push(Literal::Integer(value.into()));
            continue;
        }
        if Some(op) == pushint32 {
            if pc + 4 > script.len() {
                break;
            }
            let value =
                i32::from_le_bytes([script[pc], script[pc + 1], script[pc + 2], script[pc + 3]]);
            pc += 4;
            stack.push(Literal::Integer(value.into()));
            continue;
        }
        if Some(op) == pushint64 {
            if pc + 8 > script.len() {
                break;
            }
            let value = i64::from_le_bytes([
                script[pc],
                script[pc + 1],
                script[pc + 2],
                script[pc + 3],
                script[pc + 4],
                script[pc + 5],
                script[pc + 6],
                script[pc + 7],
            ]);
            pc += 8;
            stack.push(Literal::Integer(value.into()));
            continue;
        }
        if Some(op) == pushint128 {
            if pc + 16 > script.len() {
                break;
            }
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(&script[pc..pc + 16]);
            pc += 16;
            let value = i128::from_le_bytes(bytes);
            stack.push(Literal::Integer(value));
            continue;
        }
        if Some(op) == pushdata1 {
            if pc >= script.len() {
                break;
            }
            let len = script[pc] as usize;
            pc += 1;
            if pc + len > script.len() {
                break;
            }
            let data = script[pc..pc + len].to_vec();
            pc += len;
            stack.push(Literal::Bytes(data));
            continue;
        }
        if Some(op) == pushdata2 {
            if pc + 2 > script.len() {
                break;
            }
            let len = u16::from_le_bytes([script[pc], script[pc + 1]]) as usize;
            pc += 2;
            if pc + len > script.len() {
                break;
            }
            let data = script[pc..pc + len].to_vec();
            pc += len;
            stack.push(Literal::Bytes(data));
            continue;
        }
        if Some(op) == pushdata4 {
            if pc + 4 > script.len() {
                break;
            }
            let len =
                u32::from_le_bytes([script[pc], script[pc + 1], script[pc + 2], script[pc + 3]])
                    as usize;
            pc += 4;
            if pc + len > script.len() {
                break;
            }
            let data = script[pc..pc + len].to_vec();
            pc += len;
            stack.push(Literal::Bytes(data));
            continue;
        }
        if Some(op) == newarray0 {
            stack.push(Literal::Array(0));
            continue;
        }
        if Some(op) == newarray {
            let count = match stack.pop() {
                Some(Literal::Integer(v)) => v,
                _ => {
                    stack.push(Literal::Unknown);
                    continue;
                }
            };
            if count < 0 {
                stack.push(Literal::Unknown);
                continue;
            }
            let count = count as usize;
            for _ in 0..count {
                if stack.pop().is_none() {
                    cleared = true;
                    break;
                }
            }
            if cleared {
                stack.clear();
                continue;
            }
            stack.push(Literal::Array(count));
            continue;
        }
        if Some(op) == pack {
            let count = match stack.pop() {
                Some(Literal::Integer(v)) => v,
                _ => {
                    stack.push(Literal::Unknown);
                    continue;
                }
            };
            if count < 0 {
                stack.push(Literal::Unknown);
                continue;
            }
            let count = count as usize;
            if stack.len() < count {
                stack.clear();
                continue;
            }
            for _ in 0..count {
                stack.pop();
            }
            stack.push(Literal::Array(count));
            continue;
        }
        if Some(op) == drop_op {
            let _ = stack.pop();
            continue;
        }
        if Some(op) == ret {
            stack.clear();
            continue;
        }
        if Some(op) == syscall {
            if pc + 4 > script.len() {
                break;
            }
            let hash =
                u32::from_le_bytes([script[pc], script[pc + 1], script[pc + 2], script[pc + 3]]);
            pc += 4;

            if let Some(info) = syscalls::lookup_by_hash(hash) {
                if info.name.eq_ignore_ascii_case("System.Contract.Call") {
                    let args = stack.pop().unwrap_or(Literal::Unknown);
                    let call_flags = stack.pop().unwrap_or(Literal::Unknown);
                    let method = stack.pop().unwrap_or(Literal::Unknown);
                    let hash_bytes = stack.pop().unwrap_or(Literal::Unknown);

                    if let (
                        Literal::Bytes(contract_hash),
                        Literal::Bytes(method_bytes),
                        Literal::Integer(flags),
                        Literal::Array(param_count),
                    ) = (
                        hash_bytes.clone(),
                        method.clone(),
                        call_flags.clone(),
                        args.clone(),
                    ) {
                        if contract_hash.len() == HASH160_LENGTH {
                            if let Ok(method_name) = String::from_utf8(method_bytes.clone()) {
                                if flags >= 0 && flags <= u8::MAX as i128 {
                                    let has_return_value = {
                                        if pc < script.len() {
                                            Some(script[pc]) != drop_op
                                        } else {
                                            true
                                        }
                                    };
                                    let token = MethodToken {
                                        contract_hash: {
                                            let mut array = [0u8; HASH160_LENGTH];
                                            array.copy_from_slice(&contract_hash);
                                            array
                                        },
                                        method: method_name,
                                        parameters_count: param_count as u16,
                                        has_return_value,
                                        call_flags: flags as u8,
                                    };
                                    tokens.push(token);
                                }
                            }
                        }
                    }
                } else {
                    let has_return_value = {
                        if pc < script.len() {
                            Some(script[pc]) != drop_op
                        } else {
                            true
                        }
                    };
                    tokens.push(MethodToken {
                        contract_hash: [0u8; HASH160_LENGTH],
                        method: info.name.to_string(),
                        parameters_count: 0,
                        has_return_value,
                        call_flags: 0,
                    });
                }
            }

            // push placeholder for syscall return value
            stack.push(Literal::Unknown);
            continue;
        }

        stack.clear();
    }

    tokens
}

fn emit_data_init_helper(
    script: &mut Vec<u8>,
    byte_slot: usize,
    drop_slot: usize,
    segment_len: usize,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    emit_load_static(script, drop_slot)?;
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("NOTEQUAL")?.byte);
    let trap_dropped = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_len_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_dest_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_src_negative = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    let _ = emit_push_int(script, segment_len as i128);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let skip_copy = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    emit_load_static(script, byte_slot)?;
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("MEMCPY")?.byte);

    let done_label = script.len();
    script.push(lookup_opcode("RET")?.byte);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_dropped, trap_label)?;
    patch_jump(script, trap_len_negative, trap_label)?;
    patch_jump(script, trap_dest_negative, trap_label)?;
    patch_jump(script, trap_src_negative, trap_label)?;
    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, skip_copy, done_label)?;
    Ok(())
}

fn emit_data_drop_helper(script: &mut Vec<u8>, drop_slot: usize) -> Result<()> {
    emit_load_static(script, drop_slot)?;
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("NOTEQUAL")?.byte);
    let trap_already = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    let _ = emit_push_int(script, 1);
    emit_store_static(script, drop_slot)?;
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_already, trap_label)?;
    Ok(())
}

fn emit_popcnt_helper(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    mask_top_bits(script, bits)?;
    emit_popcnt_core(script, bits)?;
    script.push(RET);
    Ok(())
}

fn emit_ctz_helper(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    mask_top_bits(script, bits)?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("NEGATE")?.byte);
    script.push(lookup_opcode("AND")?.byte);
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("SUB")?.byte);
    mask_top_bits(script, bits)?;
    emit_popcnt_core(script, bits)?;
    script.push(RET);

    let zero_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    let _ = emit_push_int(script, bits as i128);
    script.push(RET);

    patch_jump(script, zero_branch, zero_label)?;
    Ok(())
}

fn emit_clz_helper(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    mask_top_bits(script, bits)?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let shifts: &[u32] = match bits {
        32 => &[1, 2, 4, 8, 16],
        64 => &[1, 2, 4, 8, 16, 32],
        _ => bail!("unsupported bit-width {} for clz helper", bits),
    };

    for &shift in shifts {
        script.push(lookup_opcode("DUP")?.byte);
        let _ = emit_push_int(script, shift as i128);
        script.push(lookup_opcode("SHR")?.byte);
        script.push(lookup_opcode("OR")?.byte);
    }

    script.push(lookup_opcode("INVERT")?.byte);
    mask_top_bits(script, bits)?;
    emit_popcnt_core(script, bits)?;
    script.push(RET);

    let zero_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    let _ = emit_push_int(script, bits as i128);
    script.push(RET);

    patch_jump(script, zero_branch, zero_label)?;
    Ok(())
}

fn emit_popcnt_core(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    let (mask1, mask2, mask4, h01, shift) = match bits {
        32 => (
            0x5555_5555u64 as i128,
            0x3333_3333u64 as i128,
            0x0F0F_0F0Fu64 as i128,
            0x0101_0101u64 as i128,
            24,
        ),
        64 => (
            0x5555_5555_5555_5555u64 as i128,
            0x3333_3333_3333_3333u64 as i128,
            0x0F0F_0F0F_0F0F_0F0Fu64 as i128,
            0x0101_0101_0101_0101u64 as i128,
            56,
        ),
        _ => bail!("unsupported bit-width {} for popcnt helper", bits),
    };

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("SHR")?.byte);
    let _ = emit_push_int(script, mask1);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("SUB")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, mask2);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("OVER")?.byte);
    let _ = emit_push_int(script, 2);
    script.push(lookup_opcode("SHR")?.byte);
    let _ = emit_push_int(script, mask2);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 4);
    script.push(lookup_opcode("SHR")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    let _ = emit_push_int(script, mask4);
    script.push(lookup_opcode("AND")?.byte);

    let _ = emit_push_int(script, h01);
    script.push(lookup_opcode("MUL")?.byte);
    let _ = emit_push_int(script, shift as i128);
    script.push(lookup_opcode("SHR")?.byte);
    Ok(())
}

pub(crate) fn emit_select(
    script: &mut Vec<u8>,
    true_value: StackValue,
    false_value: StackValue,
    condition: StackValue,
) -> Result<StackValue> {
    let jmp_false = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);
    let jmp_end = emit_jump_placeholder(script, "JMP_L")?;
    let else_target = script.len();
    patch_jump(script, jmp_false, else_target)?;
    script.push(lookup_opcode("NIP")?.byte);
    let end_target = script.len();
    patch_jump(script, jmp_end, end_target)?;

    let const_value = match condition.const_value {
        Some(value) if value != 0 => true_value.const_value,
        Some(_) => false_value.const_value,
        None => match (true_value.const_value, false_value.const_value) {
            (Some(a), Some(b)) if a == b => Some(a),
            _ => None,
        },
    };

    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

pub(crate) fn emit_zero_extend(
    script: &mut Vec<u8>,
    value: StackValue,
    bits: u32,
) -> Result<StackValue> {
    let const_result = value.const_value.map(|c| truncate_to_bits(c, bits));

    if let (Some(result), Some(start)) = (const_result, value.bytecode_start) {
        script.truncate(start);
        return Ok(emit_push_int(script, result));
    }

    mask_top_bits(script, bits)?;
    Ok(StackValue {
        const_value: const_result,
        bytecode_start: None,
    })
}

pub(crate) fn emit_bit_count(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value: StackValue,
    kind: BitHelperKind,
) -> Result<StackValue> {
    let bits = kind.bits();
    if let Some(constant) = value.const_value {
        let result = match kind {
            BitHelperKind::Clz(_) => clz_const(constant, bits),
            BitHelperKind::Ctz(_) => ctz_const(constant, bits),
            BitHelperKind::Popcnt(_) => popcnt_const(constant, bits),
        };

        if let Some(start) = value.bytecode_start {
            script.truncate(start);
            return Ok(emit_push_int(script, result));
        }
    }

    runtime.emit_bit_helper(script, kind)?;
    Ok(StackValue {
        const_value: None,
        bytecode_start: None,
    })
}

fn popcnt_const(value: i128, bits: u32) -> i128 {
    let masked = truncate_to_bits(value, bits);
    match bits {
        32 => (masked as u32).count_ones() as i128,
        64 => (masked as u64).count_ones() as i128,
        _ => unreachable!("unsupported bit-width {} for popcnt", bits),
    }
}

fn ctz_const(value: i128, bits: u32) -> i128 {
    let masked = truncate_to_bits(value, bits);
    if masked == 0 {
        return bits as i128;
    }
    match bits {
        32 => (masked as u32).trailing_zeros() as i128,
        64 => (masked as u64).trailing_zeros() as i128,
        _ => unreachable!("unsupported bit-width {} for ctz", bits),
    }
}

fn clz_const(value: i128, bits: u32) -> i128 {
    let masked = truncate_to_bits(value, bits);
    if masked == 0 {
        return bits as i128;
    }
    match bits {
        32 => (masked as u32).leading_zeros() as i128,
        64 => (masked as u64).leading_zeros() as i128,
        _ => unreachable!("unsupported bit-width {} for clz", bits),
    }
}

pub(crate) fn emit_sign_extend(
    script: &mut Vec<u8>,
    value: StackValue,
    from_bits: u32,
    total_bits: u32,
) -> Result<StackValue> {
    let const_result = value
        .const_value
        .map(|c| sign_extend_const(truncate_to_bits(c, from_bits), from_bits));

    if let (Some(result), Some(start)) = (const_result, value.bytecode_start) {
        script.truncate(start);
        return Ok(emit_push_int(script, result));
    }

    mask_top_bits(script, from_bits)?;
    let shift = total_bits.saturating_sub(from_bits);
    if shift > 0 {
        let _ = emit_push_int(script, shift as i128);
        script.push(lookup_opcode("SHL")?.byte);
        let _ = emit_push_int(script, shift as i128);
        script.push(lookup_opcode("SHR")?.byte);
    }

    Ok(StackValue {
        const_value: const_result,
        bytecode_start: None,
    })
}

fn mask_top_bits(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    if bits >= 128 {
        return Ok(());
    }
    let mask = ((1u128 << bits) - 1) as i128;
    let _ = emit_push_int(script, mask);
    script.push(lookup_opcode("AND")?.byte);
    Ok(())
}

fn truncate_to_bits(value: i128, bits: u32) -> i128 {
    if bits >= 128 {
        value
    } else {
        let mask = (1i128 << bits) - 1;
        value & mask
    }
}

fn sign_extend_const(value: i128, bits: u32) -> i128 {
    if bits == 0 || bits >= 128 {
        value
    } else {
        let shift = 128 - bits;
        let masked = truncate_to_bits(value, bits);
        (masked << shift) >> shift
    }
}
