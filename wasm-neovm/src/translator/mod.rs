// Required imports
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{Deserializer, Value};
use std::collections::{BTreeMap, HashSet};
use wasmparser::{
    BlockType, BrTable, CompositeInnerType, DataKind, ElementKind, ExternalKind, FuncType,
    HeapType, Import, Operator, Parser, Payload, RefType, TableType, TypeRef, ValType,
};

use wasmparser::ConstExpr;

use crate::manifest::{
    build_manifest, merge_manifest, ManifestMethod, ManifestParameter, RenderedManifest,
};
use crate::metadata::{
    dedup_method_tokens, extract_nef_metadata, parse_method_token_section, update_manifest_metadata,
};
use crate::nef::{MethodToken, HASH160_LENGTH};
use crate::neo_syscalls;
use crate::numeric;
use crate::opcodes;
use crate::syscalls;

// ============================================================================
// Constants
// ============================================================================

/// Sentinel value for null function references in tables
const FUNCREF_NULL: i128 = -1;

/// RET opcode byte value
const RET: u8 = 0x40;

/// Push integer opcode constants
const PUSHM1: u8 = 0x0F;
const PUSH0: u8 = 0x10;
const PUSH_BASE: u8 = 0x10; // PUSH1-PUSH16 are 0x11-0x20
const PUSHINT8: u8 = 0x00;
const PUSHINT16: u8 = 0x01;
const PUSHINT32: u8 = 0x02;
const PUSHINT64: u8 = 0x03;
const PUSHINT128: u8 = 0x04;
const PUSHINT256: u8 = 0x05;

/// CONVERT opcode for type conversions
const CONVERT: u8 = 0xD3;

/// Stack item type constants for CONVERT
const STACKITEMTYPE_INTEGER: u8 = 0x21;

/// Documentation hint for unsupported features
const UNSUPPORTED_FEATURE_DOC: &str = "Refer to docs/wasm-neovm-status.md for current coverage.";

const CUSTOM_SECTION_PREFIX: &str = ".custom_section.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomSectionKind {
    Manifest,
    MethodTokens,
}

fn classify_custom_section(name: &str) -> Option<CustomSectionKind> {
    let stripped = name.strip_prefix(CUSTOM_SECTION_PREFIX).unwrap_or(name);
    if stripped == "neo.manifest" || stripped.starts_with("neo.manifest.") {
        Some(CustomSectionKind::Manifest)
    } else if stripped == "neo.methodtokens" || stripped.starts_with("neo.methodtokens.") {
        Some(CustomSectionKind::MethodTokens)
    } else {
        None
    }
}

fn parse_concatenated_json(data: &[u8], context: &str) -> Result<Vec<Value>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let text = std::str::from_utf8(data)
        .with_context(|| format!("{context} custom section must contain valid UTF-8 data"))?;

    let stream = Deserializer::from_str(text).into_iter::<Value>();
    let mut values = Vec::new();
    for fragment in stream {
        let value = fragment.with_context(|| format!("failed to parse {context} JSON fragment"))?;
        values.push(value);
    }

    Ok(values)
}

// ============================================================================
// Translation Result Types
// ============================================================================

/// The result of translating a WASM module
#[derive(Debug)]
pub struct Translation {
    pub script: Vec<u8>,
    pub method_tokens: Vec<MethodToken>,
    pub manifest: RenderedManifest,
    pub source_url: Option<String>,
}

/// Represents a value on the WASM value stack during translation
#[derive(Debug, Clone)]
struct StackValue {
    const_value: Option<i128>,
    bytecode_start: Option<usize>,
}

#[derive(Debug, Clone)]
struct FunctionImport {
    module: String,
    name: String,
    type_index: u32,
}

#[derive(Debug)]
pub struct ManifestData {
    pub methods: Vec<ManifestMethod>,
}

// ============================================================================
// Control Flow Types
// ============================================================================

/// Represents different kinds of control flow constructs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlKind {
    Block,
    Loop,
    If,
    Function,
}

/// Represents a control flow frame on the control stack
#[derive(Debug, Clone)]
struct ControlFrame {
    kind: ControlKind,
    stack_height: usize,
    result_count: usize, // Expected number of results (for Function and Block types)
    start_offset: usize,
    end_fixups: Vec<usize>,
    if_false_fixup: Option<usize>,
    has_else: bool,
}

// ============================================================================
// Jump Helper Functions
// ============================================================================

/// Emit a jump instruction with a placeholder target that will be patched later
fn emit_jump_placeholder(script: &mut Vec<u8>, opcode: &str) -> Result<usize> {
    let opcode_byte = opcodes::lookup(opcode)
        .ok_or_else(|| anyhow!("unknown opcode: {}", opcode))?
        .byte;

    script.push(opcode_byte);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder (will be patched later with actual offset)
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Patch a previously emitted jump instruction with the actual target offset
fn patch_jump(script: &mut Vec<u8>, position: usize, target: usize) -> Result<()> {
    if position + 4 > script.len() {
        bail!(
            "invalid jump patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // Calculate relative offset (target - position)
    // NeoVM jump offsets are relative to the position after the jump instruction
    let offset = (target as i32) - (position as i32 + 4);
    let bytes = offset.to_le_bytes();

    // Patch the 4-byte placeholder
    script[position..position + 4].copy_from_slice(&bytes);

    Ok(())
}

/// Emit a call instruction with a placeholder that will be patched later
fn emit_call_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    let call_opcode = opcodes::lookup("CALL_L")
        .ok_or_else(|| anyhow!("CALL_L opcode not found"))?
        .byte;

    script.push(call_opcode);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder for call target
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Patch a previously emitted call instruction with the actual function offset
fn patch_call(script: &mut Vec<u8>, position: usize, target: usize) -> Result<()> {
    if position + 4 > script.len() {
        bail!(
            "invalid call patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // Calculate relative offset for call
    let offset = (target as i32) - (position as i32 + 4);
    let bytes = offset.to_le_bytes();

    // Patch the 4-byte placeholder
    script[position..position + 4].copy_from_slice(&bytes);

    Ok(())
}

/// Emit a jump to a specific target offset
fn emit_jump_to(script: &mut Vec<u8>, opcode: &str, target: usize) -> Result<()> {
    let opcode_byte = opcodes::lookup(opcode)
        .ok_or_else(|| anyhow!("unknown opcode: {}", opcode))?
        .byte;

    script.push(opcode_byte);
    let current_pos = script.len();

    // Calculate relative offset
    let offset = (target as i32) - (current_pos as i32 + 4);
    let bytes = offset.to_le_bytes();

    // Emit the offset
    script.extend_from_slice(&bytes);

    Ok(())
}

/// Emit a load from a static field slot
fn emit_load_static(script: &mut Vec<u8>, slot: usize) -> Result<()> {
    // NeoVM has optimized opcodes for slots 0-6
    let opcode = match slot {
        0 => "LDSFLD0",
        1 => "LDSFLD1",
        2 => "LDSFLD2",
        3 => "LDSFLD3",
        4 => "LDSFLD4",
        5 => "LDSFLD5",
        6 => "LDSFLD6",
        _ => {
            // For slots >= 7, use LDSFLD with explicit slot index
            let ldsfld = opcodes::lookup("LDSFLD")
                .ok_or_else(|| anyhow!("LDSFLD opcode not found"))?
                .byte;
            script.push(ldsfld);
            script.push(slot as u8);
            return Ok(());
        }
    };

    let opcode_byte = opcodes::lookup(opcode)
        .ok_or_else(|| anyhow!("unknown opcode: {}", opcode))?
        .byte;

    script.push(opcode_byte);
    Ok(())
}

/// Emit a store to a static field slot
fn emit_store_static(script: &mut Vec<u8>, slot: usize) -> Result<()> {
    // NeoVM has optimized opcodes for slots 0-6
    let opcode = match slot {
        0 => "STSFLD0",
        1 => "STSFLD1",
        2 => "STSFLD2",
        3 => "STSFLD3",
        4 => "STSFLD4",
        5 => "STSFLD5",
        6 => "STSFLD6",
        _ => {
            // For slots >= 7, use STSFLD with explicit slot index
            let stsfld = opcodes::lookup("STSFLD")
                .ok_or_else(|| anyhow!("STSFLD opcode not found"))?
                .byte;
            script.push(stsfld);
            script.push(slot as u8);
            return Ok(());
        }
    };

    let opcode_byte = opcodes::lookup(opcode)
        .ok_or_else(|| anyhow!("unknown opcode: {}", opcode))?
        .byte;

    script.push(opcode_byte);
    Ok(())
}

/// Emit a push data instruction with byte array data
fn emit_push_data(script: &mut Vec<u8>, data: &[u8]) -> Result<()> {
    let len = data.len();

    if len <= 75 {
        // For data <= 75 bytes, use direct push with length prefix
        script.push(len as u8);
        script.extend_from_slice(data);
    } else if len <= 255 {
        // For data <= 255 bytes, use PUSHDATA1
        let pushdata1 = opcodes::lookup("PUSHDATA1")
            .ok_or_else(|| anyhow!("PUSHDATA1 opcode not found"))?
            .byte;
        script.push(pushdata1);
        script.push(len as u8);
        script.extend_from_slice(data);
    } else if len <= 65535 {
        // For data <= 65535 bytes, use PUSHDATA2
        let pushdata2 = opcodes::lookup("PUSHDATA2")
            .ok_or_else(|| anyhow!("PUSHDATA2 opcode not found"))?
            .byte;
        script.push(pushdata2);
        script.extend_from_slice(&(len as u16).to_le_bytes());
        script.extend_from_slice(data);
    } else {
        // For larger data, use PUSHDATA4
        let pushdata4 = opcodes::lookup("PUSHDATA4")
            .ok_or_else(|| anyhow!("PUSHDATA4 opcode not found"))?
            .byte;
        script.push(pushdata4);
        script.extend_from_slice(&(len as u32).to_le_bytes());
        script.extend_from_slice(data);
    }

    Ok(())
}

/// Emit a TRY_L instruction with placeholder catch offset
fn emit_try_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    let try_l = opcodes::lookup("TRY_L")
        .ok_or_else(|| anyhow!("TRY_L opcode not found"))?
        .byte;

    script.push(try_l);
    let placeholder_pos = script.len();

    // Emit 8-byte placeholder (4 bytes for catch offset, 4 bytes for finally offset)
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Emit an ENDTRY_L instruction with placeholder offset
fn emit_endtry_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    let endtry_l = opcodes::lookup("ENDTRY_L")
        .ok_or_else(|| anyhow!("ENDTRY_L opcode not found"))?
        .byte;

    script.push(endtry_l);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder for end offset
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Patch a TRY_L instruction with catch and finally offsets
fn patch_try_catch(script: &mut Vec<u8>, position: usize, catch_offset: usize) -> Result<()> {
    if position + 8 > script.len() {
        bail!(
            "invalid try patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // Calculate relative offset for catch block
    let catch_rel = (catch_offset as i32) - (position as i32 + 4);
    let catch_bytes = catch_rel.to_le_bytes();

    // Patch catch offset (first 4 bytes)
    script[position..position + 4].copy_from_slice(&catch_bytes);

    // Leave finally offset as 0 (no finally block) - last 4 bytes
    script[position + 4..position + 8].copy_from_slice(&[0, 0, 0, 0]);

    Ok(())
}

/// Patch an ENDTRY_L instruction with the end offset
fn patch_endtry(script: &mut Vec<u8>, position: usize, end_offset: usize) -> Result<()> {
    if position + 4 > script.len() {
        bail!(
            "invalid endtry patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // Calculate relative offset to end
    let offset = (end_offset as i32) - (position as i32 + 4);
    let bytes = offset.to_le_bytes();

    // Patch the 4-byte placeholder
    script[position..position + 4].copy_from_slice(&bytes);

    Ok(())
}

// ============================================================================
// Runtime Helpers and Supporting Types
// ============================================================================

struct RuntimeHelpers {
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

struct StartDescriptor {
    function_index: u32,
    kind: StartKind,
}

enum StartKind {
    Defined { offset: usize },
    Import,
}

struct StartHelper<'a> {
    slot: usize,
    descriptor: &'a StartDescriptor,
}

struct FunctionRegistry {
    offsets: Vec<Option<usize>>,
    fixups: Vec<Vec<usize>>,
}

impl FunctionRegistry {
    fn new(total_functions: usize) -> Self {
        FunctionRegistry {
            offsets: vec![None; total_functions],
            fixups: vec![Vec::new(); total_functions],
        }
    }

    fn register_offset(
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

    fn emit_call(&mut self, script: &mut Vec<u8>, function_index: usize) -> Result<()> {
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
    fn helper_record_mut(&mut self, kind: MemoryHelperKind) -> &mut HelperRecord {
        self.memory_helpers.entry(kind).or_default()
    }
    fn bit_helper_record_mut(&mut self, kind: BitHelperKind) -> &mut HelperRecord {
        self.bit_helpers.entry(kind).or_default()
    }
    fn table_helper_record_mut(&mut self, kind: TableHelperKind) -> &mut HelperRecord {
        self.table_helpers.entry(kind).or_default()
    }
    fn emit_memory_init_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        self.memory_init_calls.push(call_pos);
        Ok(())
    }

    fn finalize(
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
        let table_base = 4 + global_count;

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
            slot: 4
                + global_layouts.len()
                + table_layouts.len()
                + passive_layout_vec.len() * 2
                + passive_element_layouts.len() * 2,
            descriptor,
        });

        let static_slot_count = 4
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

    fn set_memory_config(&mut self, initial_pages: u32, maximum_pages: Option<u32>) -> Result<()> {
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

    fn memory_defined(&self) -> bool {
        self.memory_defined
    }

    fn emit_memory_load_call(&mut self, script: &mut Vec<u8>, bytes: u32) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Load(bytes));
        record.calls.push(call_pos);
        Ok(())
    }

    fn emit_memory_store_call(&mut self, script: &mut Vec<u8>, bytes: u32) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Store(bytes));
        record.calls.push(call_pos);
        Ok(())
    }

    fn emit_memory_grow_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Grow);
        record.calls.push(call_pos);
        Ok(())
    }

    fn emit_memory_fill_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Fill);
        record.calls.push(call_pos);
        Ok(())
    }

    fn emit_memory_copy_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Copy);
        record.calls.push(call_pos);
        Ok(())
    }

    fn emit_bit_helper(&mut self, script: &mut Vec<u8>, kind: BitHelperKind) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.bit_helper_record_mut(kind);
        record.calls.push(call_pos);
        Ok(())
    }

    fn emit_table_helper(&mut self, script: &mut Vec<u8>, kind: TableHelperKind) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.table_helper_record_mut(kind);
        record.calls.push(call_pos);
        Ok(())
    }

    fn emit_data_init_call(&mut self, script: &mut Vec<u8>, segment_index: u32) -> Result<()> {
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

    fn emit_data_drop_call(&mut self, script: &mut Vec<u8>, segment_index: u32) -> Result<()> {
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

    fn register_passive_segment(&mut self, bytes: Vec<u8>) -> Result<()> {
        let index = self.next_data_index;
        self.next_data_index += 1;
        let segment = self.ensure_passive_segment(index)?;
        segment.bytes = bytes;
        segment.defined = true;
        Ok(())
    }

    fn register_active_segment(&mut self, _memory: u32, offset: u64, bytes: Vec<u8>) -> Result<()> {
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

    fn ensure_passive_segment(&mut self, index: usize) -> Result<&mut DataSegmentInfo> {
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

    fn register_table(&mut self, initial_len: usize, maximum: Option<u32>) -> usize {
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

    fn table_descriptor_mut(&mut self, index: usize) -> Result<&mut TableDescriptor> {
        self.tables
            .get_mut(index)
            .ok_or_else(|| anyhow!("table index {} out of range", index))
    }

    fn table_descriptor_const(&self, index: usize) -> Result<&TableDescriptor> {
        self.tables
            .get(index)
            .ok_or_else(|| anyhow!("table index {} out of range", index))
    }

    fn passive_element_slots_const(&self, index: usize) -> Result<(usize, usize)> {
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

    fn passive_element_drop_slot_const(&self, index: usize) -> Result<usize> {
        let (_, drop_slot) = self.passive_element_slots_const(index)?;
        Ok(drop_slot)
    }

    fn table_slot(&mut self, index: usize) -> Result<usize> {
        let base = 4 + self.globals.len();
        let table = self.table_descriptor_mut(index)?;
        if table.slot.is_none() {
            table.slot = Some(base + index);
        }
        Ok(table.slot.expect("table slot should be assigned"))
    }

    fn register_active_element(
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

    fn register_passive_element(&mut self, values: Vec<i32>) -> usize {
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

    fn ensure_passive_element(&mut self, index: usize) -> Result<&mut ElementSegmentInfo> {
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

    fn apply_active_element(
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

    fn register_global(&mut self, mutable: bool, initial_value: i128) -> usize {
        let slot = 4 + self.globals.len();
        let const_value = if mutable { None } else { Some(initial_value) };
        self.globals.push(GlobalDescriptor {
            slot,
            mutable,
            initial_value,
            const_value,
        });
        self.globals.len() - 1
    }

    fn global_slot(&self, index: usize) -> Result<usize> {
        self.globals
            .get(index)
            .map(|g| g.slot)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    fn global_mutable(&self, index: usize) -> Result<bool> {
        self.globals
            .get(index)
            .map(|g| g.mutable)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    fn global_const_value(&self, index: usize) -> Result<Option<i128>> {
        self.globals
            .get(index)
            .map(|g| g.const_value)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    fn clear_global_const(&mut self, index: usize) -> Result<()> {
        let global = self
            .globals
            .get_mut(index)
            .ok_or_else(|| anyhow!("global index {} out of range", index))?;
        global.const_value = None;
        Ok(())
    }
}

#[derive(Clone)]
struct MemoryConfig {
    initial_pages: u32,
    maximum_pages: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MemoryHelperKind {
    Load(u32),
    Store(u32),
    Grow,
    Fill,
    Copy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BitHelperKind {
    Clz(u32),
    Ctz(u32),
    Popcnt(u32),
}

impl BitHelperKind {
    fn bits(self) -> u32 {
        match self {
            BitHelperKind::Clz(bits) | BitHelperKind::Ctz(bits) | BitHelperKind::Popcnt(bits) => {
                bits
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TableHelperKind {
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
enum CallTarget {
    Import(u32),
    Defined(usize),
}

#[derive(Default)]
struct HelperRecord {
    offset: Option<usize>,
    calls: Vec<usize>,
}

struct DataSegmentInfo {
    bytes: Vec<u8>,
    kind: DataSegmentKind,
    defined: bool,
}

enum DataSegmentKind {
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

struct GlobalDescriptor {
    slot: usize,
    mutable: bool,
    initial_value: i128,
    const_value: Option<i128>,
}

struct TableDescriptor {
    initial_entries: Vec<i32>,
    maximum: Option<usize>,
    slot: Option<usize>,
}

enum ElementSegmentKind {
    Passive {
        value_slot: Option<usize>,
        drop_slot: Option<usize>,
    },
    Active {
        _table_index: usize,
        _offset: usize,
    },
}

struct ElementSegmentInfo {
    values: Vec<i32>,
    kind: ElementSegmentKind,
    defined: bool,
}

struct TableInfo {
    entries: Vec<Option<u32>>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            initial_pages: 0,
            maximum_pages: None,
        }
    }
}

pub fn translate_module(bytes: &[u8], contract_name: &str) -> Result<Translation> {
    translate_module_with_safe(bytes, contract_name, &[])
}

pub fn translate_module_with_safe(
    bytes: &[u8],
    contract_name: &str,
    safe_methods: &[&str],
) -> Result<Translation> {
    translate_module_internal(bytes, contract_name, safe_methods)
}

fn translate_module_internal(
    bytes: &[u8],
    contract_name: &str,
    safe_methods: &[&str],
) -> Result<Translation> {
    let parser = Parser::new(0);
    let mut types: Vec<FuncType> = Vec::new();
    let mut func_type_indices: Vec<u32> = Vec::new();
    let mut exported_funcs: BTreeMap<u32, (String, bool)> = BTreeMap::new();
    let mut tables: Vec<TableInfo> = Vec::new();
    let mut imports: Vec<FunctionImport> = Vec::new();
    let mut script: Vec<u8> = Vec::new();
    let mut runtime = RuntimeHelpers::default();
    let mut methods: Vec<ManifestMethod> = Vec::new();
    let mut remaining_safe: HashSet<&str> = safe_methods.iter().copied().collect();
    let mut manifest_overlay: Option<Value> = None;
    let mut section_method_tokens: Vec<MethodToken> = Vec::new();
    let mut section_source: Option<String> = None;
    let mut saw_code_section = false;
    let mut next_defined_index: usize = 0;
    let mut function_registry: Option<FunctionRegistry> = None;
    let mut start_function: Option<u32> = None;
    let mut start_defined_offset: Option<usize> = None;

    for payload in parser.parse_all(bytes) {
        match payload? {
            Payload::Version { .. } => {}
            Payload::TypeSection(reader) => {
                for group in reader {
                    let group = group?;
                    for (_, subtype) in group.into_types_and_offsets() {
                        match subtype.composite_type.inner {
                            CompositeInnerType::Func(func) => types.push(func),
                            _ => {}
                        }
                    }
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    match import.ty {
                        TypeRef::Func(type_index) => {
                            imports.push(FunctionImport {
                                module: import.module.to_string(),
                                name: import.name.to_string(),
                                type_index,
                            });
                        }
                        TypeRef::Global(_) => {
                            bail!(
                                "global imports are not supported ({}::{})",
                                import.module,
                                import.name
                            );
                        }
                        _ => {
                            bail!("only function imports are supported (found non-function import {})", import.name);
                        }
                    }
                }
            }
            Payload::FunctionSection(reader) => {
                for idx in reader {
                    func_type_indices.push(idx?);
                }
            }
            Payload::TableSection(reader) => {
                for table in reader {
                    let table = table?;
                    if table.ty.table64 {
                        bail!("table64 is not supported");
                    }
                    if table.ty.shared {
                        bail!("shared tables are not supported");
                    }
                    if table.ty.element_type != RefType::FUNCREF {
                        bail!(
                            "reference type {:?} tables are not supported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                            table.ty.element_type
                        );
                    }
                    let initial_len = usize::try_from(table.ty.initial)
                        .context("table initial size exceeds host limits")?;
                    let maximum = match table.ty.maximum {
                        Some(max) => {
                            Some(u32::try_from(max).context("table maximum exceeds 32-bit range")?)
                        }
                        None => None,
                    };
                    runtime.register_table(initial_len, maximum);
                    tables.push(TableInfo {
                        entries: vec![None; initial_len],
                    });
                }
            }
            Payload::GlobalSection(reader) => {
                for entry in reader {
                    let entry = entry?;
                    let value_type = entry.ty.content_type;
                    match value_type {
                        ValType::I32 | ValType::I64 => {}
                        other => bail!("only i32/i64 globals are supported (found {:?})", other),
                    }
                    let initial = evaluate_global_init(entry.init_expr, value_type)
                        .context("failed to evaluate global initialiser")?;
                    runtime.register_global(entry.ty.mutable, initial);
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    if export.kind == ExternalKind::Func {
                        exported_funcs.insert(export.index, (export.name.to_string(), false));
                    }
                }
            }
            Payload::MemorySection(reader) => {
                for mem in reader {
                    let mem = mem?;
                    if mem.memory64 {
                        bail!("memory64 is not supported");
                    }
                    if mem.shared {
                        bail!("shared memories are not supported");
                    }
                    let initial = u32::try_from(mem.initial)
                        .context("memory initial size exceeds 32-bit range")?;
                    let maximum = match mem.maximum {
                        Some(max) => Some(
                            u32::try_from(max).context("memory maximum exceeds 32-bit range")?,
                        ),
                        None => None,
                    };
                    runtime
                        .set_memory_config(initial, maximum)
                        .context("failed to register memory section")?;
                }
            }
            Payload::CodeSectionStart { .. } => {
                saw_code_section = true;
                next_defined_index = 0;
                let total_functions = imports.len() + func_type_indices.len();
                function_registry = Some(FunctionRegistry::new(total_functions));
            }
            Payload::CodeSectionEntry(body) => {
                let functions = function_registry
                    .as_mut()
                    .ok_or_else(|| anyhow!("code section encountered without initialisation"))?;
                let defined_index = next_defined_index;
                next_defined_index += 1;

                let func_index = imports.len() + defined_index;
                let func_index_u32 = func_index as u32;
                let maybe_export = exported_funcs.get_mut(&func_index_u32);
                if let Some(entry) = maybe_export.as_ref() {
                    if entry.1 {
                        bail!("function index {} exported multiple times", func_index_u32);
                    }
                }

                let function_name = maybe_export
                    .as_ref()
                    .map(|(name, _)| name.as_str())
                    .unwrap_or("<internal>");

                let type_index =
                    func_type_indices
                        .get(defined_index)
                        .copied()
                        .ok_or_else(|| {
                            anyhow!(
                                "no type index recorded for function '{}' (defined index {})",
                                function_name,
                                defined_index
                            )
                        })?;

                let func_type = types.get(type_index as usize).ok_or_else(|| {
                    anyhow!(
                        "type index {} referenced by function '{}' out of bounds",
                        type_index,
                        function_name
                    )
                })?;

                let offset = script.len();
                functions
                    .register_offset(&mut script, func_index, offset)
                    .context("failed to register function offset")?;

                if start_function == Some(func_index as u32) {
                    start_defined_offset = Some(offset);
                }

                let return_kind = translate_function(
                    func_type,
                    &body,
                    &mut script,
                    &imports,
                    &types,
                    &func_type_indices,
                    &mut runtime,
                    &tables,
                    functions,
                    func_index,
                    start_function,
                )
                .with_context(|| {
                    format!(
                        "failed to translate function '{}'",
                        maybe_export
                            .as_ref()
                            .map(|(name, _)| name.as_str())
                            .unwrap_or("<internal>")
                    )
                })?;

                if let Some(entry) = maybe_export {
                    let parameters: Vec<ManifestParameter> = func_type
                        .params()
                        .iter()
                        .enumerate()
                        .map(|(idx, param)| ManifestParameter {
                            name: format!("arg{}", idx),
                            kind: wasm_val_type_to_manifest(param)
                                .unwrap_or_else(|_| "Any".to_string()),
                        })
                        .collect();

                    let is_safe = remaining_safe.remove(entry.0.as_str());
                    let method = ManifestMethod {
                        name: entry.0.clone(),
                        parameters,
                        return_type: return_kind,
                        offset: offset as u32,
                        safe: is_safe,
                    };
                    methods.push(method);
                    entry.1 = true;
                }
            }
            Payload::ElementSection(reader) => {
                for element in reader {
                    let element = element?;

                    let (table_index_opt, offset_opt) = match element.kind {
                        wasmparser::ElementKind::Active {
                            table_index,
                            offset_expr,
                        } => {
                            let table_idx = table_index.unwrap_or(0) as usize;
                            let offset = evaluate_offset_expr(offset_expr)
                                .context("failed to evaluate element offset")?;
                            if offset < 0 {
                                bail!("element segment offset must be non-negative");
                            }
                            (Some(table_idx), Some(offset as usize))
                        }
                        wasmparser::ElementKind::Passive => (None, None),
                        wasmparser::ElementKind::Declared => {
                            bail!("declared element segments are not supported")
                        }
                    };

                    let mut func_refs: Vec<Option<u32>> = Vec::new();
                    match element.items {
                        wasmparser::ElementItems::Functions(funcs) => {
                            for func in funcs {
                                func_refs.push(Some(func?));
                            }
                        }
                        wasmparser::ElementItems::Expressions(ref_ty, exprs) => {
                            if ref_ty != RefType::FUNCREF {
                                bail!(
                                    "element expressions for reference type {:?} are not supported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                                    ref_ty
                                );
                            }
                            for expr in exprs {
                                let expr = expr?;
                                let mut reader = expr.get_operators_reader();
                                let mut value: Option<Option<u32>> = None;
                                while !reader.eof() {
                                    let op = reader.read()?;
                                    match op {
                                        Operator::RefNull { hty } => {
                                            if hty != HeapType::FUNC {
                                                bail!(
                                                    "element expression uses unsupported heap type {:?}",
                                                    hty
                                                );
                                            }
                                            value = Some(None);
                                        }
                                        Operator::RefFunc { function_index } => {
                                            value = Some(Some(function_index));
                                        }
                                        Operator::End => break,
                                        other => {
                                            bail!(
                                                "unsupported instruction {:?} in element segment expression",
                                                other
                                            );
                                        }
                                    }
                                }
                                let parsed = value.ok_or_else(|| {
                                    anyhow!("element expression did not yield a value")
                                })?;
                                func_refs.push(parsed);
                            }
                        }
                    }

                    let values_i32: Vec<i32> = func_refs
                        .iter()
                        .map(|opt| opt.map(|v| v as i32).unwrap_or(FUNCREF_NULL as i32))
                        .collect();

                    if let Some(table_index) = table_index_opt {
                        let offset = offset_opt.expect("active element requires offset");
                        let table = tables.get_mut(table_index).ok_or_else(|| {
                            anyhow!("element references missing table index {}", table_index)
                        })?;

                        for (i, func_ref) in func_refs.iter().enumerate() {
                            let slot = offset
                                .checked_add(i)
                                .ok_or_else(|| anyhow!("element segment offset overflow"))?;
                            if slot >= table.entries.len() {
                                bail!(
                                    "element segment writes past table bounds (index {}, table length {})",
                                    slot,
                                    table.entries.len()
                                );
                            }
                            table.entries[slot] = *func_ref;
                        }

                        runtime
                            .register_active_element(table_index, offset, values_i32)
                            .context("failed to register active element segment")?;
                    } else {
                        runtime.register_passive_element(values_i32);
                    }
                }
            }
            Payload::DataSection(reader) => {
                for entry in reader {
                    let entry = entry?;
                    let data_bytes = entry.data.to_vec();
                    match entry.kind {
                        DataKind::Passive => {
                            runtime
                                .register_passive_segment(data_bytes)
                                .context("failed to register passive data segment")?;
                        }
                        DataKind::Active {
                            memory_index,
                            offset_expr,
                        } => {
                            if memory_index != 0 {
                                bail!("only default memory index 0 is supported for active data segments");
                            }
                            let offset = evaluate_offset_expr(offset_expr)
                                .context("failed to evaluate active data segment offset")?;
                            if offset < 0 {
                                bail!("active data segment offset must be non-negative");
                            }
                            runtime
                                .register_active_segment(memory_index, offset as u64, data_bytes)
                                .context("failed to register active data segment")?;
                        }
                    }
                }
            }
            Payload::CustomSection(section) => match classify_custom_section(section.name()) {
                Some(CustomSectionKind::Manifest) => {
                    for overlay in parse_concatenated_json(section.data(), "neo.manifest")? {
                        if let Some(existing) = manifest_overlay.as_mut() {
                            merge_manifest(existing, &overlay);
                        } else {
                            manifest_overlay = Some(overlay);
                        }
                    }
                }
                Some(CustomSectionKind::MethodTokens) => {
                    let fragments = parse_concatenated_json(section.data(), "neo.methodtokens")?;
                    for fragment in fragments {
                        let bytes = serde_json::to_vec(&fragment)
                            .context("failed to serialize neo.methodtokens fragment")?;
                        let metadata = parse_method_token_section(&bytes)
                            .context("failed to parse neo.methodtokens custom section fragment")?;
                        if section_source.is_none() {
                            section_source = metadata.source.clone();
                        }
                        section_method_tokens.extend(metadata.method_tokens);
                    }
                }
                None => {}
            },
            Payload::StartSection { func, .. } => {
                if start_function.is_some() {
                    bail!("module contains multiple start sections");
                }
                start_function = Some(func);
            }
            Payload::TagSection(_)
            | Payload::DataCountSection { .. }
            | Payload::UnknownSection { .. } => {}
            Payload::End(_) => break,
            _ => {}
        }
    }

    if !saw_code_section {
        bail!("input module does not contain a code section");
    }

    let missing: Vec<String> = exported_funcs
        .into_iter()
        .filter_map(|(_, (name, processed))| if processed { None } else { Some(name) })
        .collect();

    if !missing.is_empty() {
        bail!(
            "did not translate exported functions: {}",
            missing.join(", ")
        );
    }

    if methods.is_empty() {
        bail!(
            "no exportable functions were translated – ensure functions are exported and meet translation constraints"
        );
    }

    if !remaining_safe.is_empty() {
        let mut missing: Vec<&str> = remaining_safe.iter().copied().collect();
        missing.sort_unstable();
        bail!(
            "the following safe methods were not found among exported functions: {}",
            missing.join(", ")
        );
    }

    let start_descriptor = if let Some(start_idx) = start_function {
        if (start_idx as usize) < imports.len() {
            let import = imports
                .get(start_idx as usize)
                .ok_or_else(|| anyhow!("start section references missing import {}", start_idx))?;
            let type_index = get_import_type_index(import)?;
            let func_type = types.get(type_index as usize).ok_or_else(|| {
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
            Some(StartDescriptor {
                function_index: start_idx,
                kind: StartKind::Import,
            })
        } else {
            let defined_index = (start_idx as usize)
                .checked_sub(imports.len())
                .ok_or_else(|| anyhow!("start function index underflow"))?;
            let type_index = func_type_indices
                .get(defined_index)
                .copied()
                .ok_or_else(|| anyhow!("no type index recorded for start function"))?;
            let func_type = types
                .get(type_index as usize)
                .ok_or_else(|| anyhow!("invalid type index {} for start function", type_index))?;
            if !func_type.params().is_empty() {
                bail!("start function must not take parameters");
            }
            if !func_type.results().is_empty() {
                bail!("start function must not return values");
            }
            let offset = start_defined_offset.ok_or_else(|| {
                anyhow!(
                    "failed to record offset for start function; ensure code section is present"
                )
            })?;
            Some(StartDescriptor {
                function_index: start_idx,
                kind: StartKind::Defined { offset },
            })
        }
    } else {
        None
    };

    runtime.finalize(&mut script, start_descriptor.as_ref(), &imports, &types)?;

    let mut manifest = build_manifest(contract_name, &methods);
    if let Some(overlay) = manifest_overlay {
        merge_manifest(&mut manifest.value, &overlay);
    }

    let mut metadata = extract_nef_metadata(&manifest.value)?;
    metadata.method_tokens.extend(section_method_tokens);
    let inferred_tokens = infer_contract_tokens(&script);
    metadata.method_tokens.extend(inferred_tokens);
    dedup_method_tokens(&mut metadata.method_tokens);
    if metadata.source.is_none() {
        metadata.source = section_source;
    }

    update_manifest_metadata(
        &mut manifest.value,
        metadata.source.as_deref(),
        &metadata.method_tokens,
    )?;

    Ok(Translation {
        script,
        manifest,
        method_tokens: metadata.method_tokens.clone(),
        source_url: metadata.source.clone(),
    })
}

fn translate_function(
    func_type: &FuncType,
    body: &wasmparser::FunctionBody,
    script: &mut Vec<u8>,
    imports: &[FunctionImport],
    types: &[FuncType],
    func_type_indices: &[u32],
    runtime: &mut RuntimeHelpers,
    tables: &[TableInfo],
    functions: &mut FunctionRegistry,
    function_index: usize,
    start_function: Option<u32>,
) -> Result<String> {
    let params = func_type.params();
    for ty in params {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!("only i32/i64 parameters are supported (found {:?})", other),
        }
    }
    let param_count = params.len();

    let returns = func_type.results();
    if returns.len() > 1 {
        bail!("multi-value returns are not supported");
    }

    if let Some(start_idx) = start_function {
        if start_idx as usize != function_index {
            runtime.emit_memory_init_call(script)?;
        }
    }

    let return_kind = returns
        .first()
        .map(|ty| wasm_val_type_to_manifest(ty))
        .transpose()?;

    let locals_reader = body.get_locals_reader()?;
    let mut local_states: Vec<LocalState> = Vec::new();
    for i in 0..param_count {
        local_states.push(LocalState {
            kind: LocalKind::Param(i as u32),
            const_value: None,
        });
    }

    let mut next_local_slot: u32 = 0;
    for entry in locals_reader {
        let (count, ty) = entry?;
        if ty != ValType::I32 && ty != ValType::I64 {
            bail!("only i32/i64 locals are supported (found {:?})", ty);
        }
        for _ in 0..count {
            local_states.push(LocalState {
                kind: LocalKind::Local(next_local_slot),
                const_value: Some(0),
            });
            next_local_slot += 1;
        }
    }

    let mut emitted_ret = false;
    let op_reader = body.get_operators_reader()?;
    let mut value_stack: Vec<StackValue> = Vec::new();
    let mut control_stack: Vec<ControlFrame> = Vec::new();
    let mut is_unreachable = false;

    // Push implicit function-level control frame
    // In WASM, the function body itself is an implicit block that can be targeted by branches
    // stack_height is 0 because branches to the function can occur at any point
    // result_count tracks how many values must be on stack when branching to function exit
    control_stack.push(ControlFrame {
        kind: ControlKind::Function,
        stack_height: 0,
        result_count: returns.len(), // Function expects return values
        start_offset: script.len(),
        end_fixups: Vec::new(),
        if_false_fixup: None,
        has_else: false,
    });

    // Ensure the current function offset is known to callers (already registered before entry).
    // This assertion helps catch internal misuse during development.
    if function_index >= functions.offsets.len() {
        bail!(
            "function index {} out of range for translation",
            function_index
        );
    }

    for op in op_reader {
        let op = op?;
        match op {
            Operator::Nop => {}
            Operator::I32Const { value } => {
                let entry = emit_push_int(script, value as i128);
                value_stack.push(entry);
            }
            Operator::I64Const { value } => {
                let entry = emit_push_int(script, value as i128);
                value_stack.push(entry);
            }
            Operator::I32Clz => {
                let value = pop_value(&mut value_stack, "i32.clz operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Clz(32))?;
                value_stack.push(result);
            }
            Operator::I32Ctz => {
                let value = pop_value(&mut value_stack, "i32.ctz operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Ctz(32))?;
                value_stack.push(result);
            }
            Operator::I32Popcnt => {
                let value = pop_value(&mut value_stack, "i32.popcnt operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Popcnt(32))?;
                value_stack.push(result);
            }
            Operator::I64Clz => {
                let value = pop_value(&mut value_stack, "i64.clz operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Clz(64))?;
                value_stack.push(result);
            }
            Operator::I64Ctz => {
                let value = pop_value(&mut value_stack, "i64.ctz operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Ctz(64))?;
                value_stack.push(result);
            }
            Operator::I64Popcnt => {
                let value = pop_value(&mut value_stack, "i64.popcnt operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Popcnt(64))?;
                value_stack.push(result);
            }
            Operator::I32Add => {
                let rhs = pop_value(&mut value_stack, "i32.add rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.add lhs")?;
                let result = emit_binary_op(script, "ADD", lhs, rhs, |a, b| Some(a + b))?;
                value_stack.push(result);
            }
            Operator::I64Add => {
                let rhs = pop_value(&mut value_stack, "i64.add rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.add lhs")?;
                let result = emit_binary_op(script, "ADD", lhs, rhs, |a, b| Some(a + b))?;
                value_stack.push(result);
            }
            Operator::I32Sub => {
                let rhs = pop_value(&mut value_stack, "i32.sub rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.sub lhs")?;
                let result = emit_binary_op(script, "SUB", lhs, rhs, |a, b| Some(a - b))?;
                value_stack.push(result);
            }
            Operator::I64Sub => {
                let rhs = pop_value(&mut value_stack, "i64.sub rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.sub lhs")?;
                let result = emit_binary_op(script, "SUB", lhs, rhs, |a, b| Some(a - b))?;
                value_stack.push(result);
            }
            Operator::I32Mul => {
                let rhs = pop_value(&mut value_stack, "i32.mul rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.mul lhs")?;
                let result = emit_binary_op(script, "MUL", lhs, rhs, |a, b| Some(a * b))?;
                value_stack.push(result);
            }
            Operator::I32And => {
                let rhs = pop_value(&mut value_stack, "i32.and rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.and lhs")?;
                let result = emit_binary_op(script, "AND", lhs, rhs, |a, b| {
                    Some(((a as i32) & (b as i32)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32Or => {
                let rhs = pop_value(&mut value_stack, "i32.or rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.or lhs")?;
                let result = emit_binary_op(script, "OR", lhs, rhs, |a, b| {
                    Some(((a as i32) | (b as i32)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32Xor => {
                let rhs = pop_value(&mut value_stack, "i32.xor rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.xor lhs")?;
                let result = emit_binary_op(script, "XOR", lhs, rhs, |a, b| {
                    Some(((a as i32) ^ (b as i32)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32Shl => {
                let rhs = pop_value(&mut value_stack, "i32.shl rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.shl lhs")?;
                let result = emit_binary_op(script, "SHL", lhs, rhs, |a, b| {
                    let shift = (b as u32) & 31;
                    Some(((a as i32) << shift) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32ShrS => {
                let rhs = pop_value(&mut value_stack, "i32.shr_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.shr_s lhs")?;
                let result = emit_shift_right(script, lhs, rhs, 32, ShiftKind::Arithmetic)?;
                value_stack.push(result);
            }
            Operator::I32ShrU => {
                let rhs = pop_value(&mut value_stack, "i32.shr_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.shr_u lhs")?;
                let result = emit_shift_right(script, lhs, rhs, 32, ShiftKind::Logical)?;
                value_stack.push(result);
            }
            Operator::I32Rotl => {
                let rhs = pop_value(&mut value_stack, "i32.rotl rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.rotl lhs")?;
                let result = emit_rotate(script, lhs, rhs, 32, true)?;
                value_stack.push(result);
            }
            Operator::I32Rotr => {
                let rhs = pop_value(&mut value_stack, "i32.rotr rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.rotr lhs")?;
                let result = emit_rotate(script, lhs, rhs, 32, false)?;
                value_stack.push(result);
            }
            Operator::I64Mul => {
                let rhs = pop_value(&mut value_stack, "i64.mul rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.mul lhs")?;
                let result = emit_binary_op(script, "MUL", lhs, rhs, |a, b| Some(a * b))?;
                value_stack.push(result);
            }
            Operator::I64And => {
                let rhs = pop_value(&mut value_stack, "i64.and rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.and lhs")?;
                let result = emit_binary_op(script, "AND", lhs, rhs, |a, b| {
                    Some(((a as i64) & (b as i64)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64Or => {
                let rhs = pop_value(&mut value_stack, "i64.or rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.or lhs")?;
                let result = emit_binary_op(script, "OR", lhs, rhs, |a, b| {
                    Some(((a as i64) | (b as i64)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64Xor => {
                let rhs = pop_value(&mut value_stack, "i64.xor rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.xor lhs")?;
                let result = emit_binary_op(script, "XOR", lhs, rhs, |a, b| {
                    Some(((a as i64) ^ (b as i64)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64Shl => {
                let rhs = pop_value(&mut value_stack, "i64.shl rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.shl lhs")?;
                let result = emit_binary_op(script, "SHL", lhs, rhs, |a, b| {
                    let shift = (b as u32) & 63;
                    Some(((a as i64) << shift) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64ShrS => {
                let rhs = pop_value(&mut value_stack, "i64.shr_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.shr_s lhs")?;
                let result = emit_shift_right(script, lhs, rhs, 64, ShiftKind::Arithmetic)?;
                value_stack.push(result);
            }
            Operator::I64ShrU => {
                let rhs = pop_value(&mut value_stack, "i64.shr_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.shr_u lhs")?;
                let result = emit_shift_right(script, lhs, rhs, 64, ShiftKind::Logical)?;
                value_stack.push(result);
            }
            Operator::I64Rotl => {
                let rhs = pop_value(&mut value_stack, "i64.rotl rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.rotl lhs")?;
                let result = emit_rotate(script, lhs, rhs, 64, true)?;
                value_stack.push(result);
            }
            Operator::I64Rotr => {
                let rhs = pop_value(&mut value_stack, "i64.rotr rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.rotr lhs")?;
                let result = emit_rotate(script, lhs, rhs, 64, false)?;
                value_stack.push(result);
            }
            Operator::I32DivS => {
                let rhs = pop_value(&mut value_stack, "i32.div_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.div_s lhs")?;
                let result = emit_binary_op(script, "DIV", lhs, rhs, |a, b| {
                    let dividend = a as i32;
                    let divisor = b as i32;
                    if divisor == 0 {
                        return None;
                    }
                    if dividend == i32::MIN && divisor == -1 {
                        return None;
                    }
                    Some((dividend / divisor) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32DivU => {
                let rhs = pop_value(&mut value_stack, "i32.div_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.div_u lhs")?;
                let result = emit_unsigned_binary_op(script, UnsignedOp::Div, lhs, rhs, 32)?;
                value_stack.push(result);
            }
            Operator::I32RemS => {
                let rhs = pop_value(&mut value_stack, "i32.rem_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.rem_s lhs")?;
                let result = emit_binary_op(script, "MOD", lhs, rhs, |a, b| {
                    let dividend = a as i32;
                    let divisor = b as i32;
                    if divisor == 0 {
                        return None;
                    }
                    Some((dividend % divisor) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32RemU => {
                let rhs = pop_value(&mut value_stack, "i32.rem_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.rem_u lhs")?;
                let result = emit_unsigned_binary_op(script, UnsignedOp::Rem, lhs, rhs, 32)?;
                value_stack.push(result);
            }
            Operator::I64DivS => {
                let rhs = pop_value(&mut value_stack, "i64.div_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.div_s lhs")?;
                let result = emit_binary_op(script, "DIV", lhs, rhs, |a, b| {
                    let dividend = a as i64;
                    let divisor = b as i64;
                    if divisor == 0 {
                        return None;
                    }
                    if dividend == i64::MIN && divisor == -1 {
                        return None;
                    }
                    Some((dividend / divisor) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64DivU => {
                let rhs = pop_value(&mut value_stack, "i64.div_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.div_u lhs")?;
                let result = emit_unsigned_binary_op(script, UnsignedOp::Div, lhs, rhs, 64)?;
                value_stack.push(result);
            }
            Operator::I64RemS => {
                let rhs = pop_value(&mut value_stack, "i64.rem_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.rem_s lhs")?;
                let result = emit_binary_op(script, "MOD", lhs, rhs, |a, b| {
                    let dividend = a as i64;
                    let divisor = b as i64;
                    if divisor == 0 {
                        return None;
                    }
                    Some((dividend % divisor) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64RemU => {
                let rhs = pop_value(&mut value_stack, "i64.rem_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.rem_u lhs")?;
                let result = emit_unsigned_binary_op(script, UnsignedOp::Rem, lhs, rhs, 64)?;
                value_stack.push(result);
            }
            Operator::I32WrapI64 => {
                let value = pop_value(&mut value_stack, "i32.wrap_i64 operand")?;
                let result = emit_sign_extend(script, value, 32, 32)?;
                value_stack.push(result);
            }
            Operator::I64ExtendI32U => {
                let value = pop_value(&mut value_stack, "i64.extend_i32_u operand")?;
                let result = emit_zero_extend(script, value, 32)?;
                value_stack.push(result);
            }
            Operator::I64ExtendI32S => {
                let value = pop_value(&mut value_stack, "i64.extend_i32_s operand")?;
                let result = emit_sign_extend(script, value, 32, 64)?;
                value_stack.push(result);
            }
            Operator::I32Extend8S => {
                let value = pop_value(&mut value_stack, "i32.extend8_s operand")?;
                let result = emit_sign_extend(script, value, 8, 32)?;
                value_stack.push(result);
            }
            Operator::I32Extend16S => {
                let value = pop_value(&mut value_stack, "i32.extend16_s operand")?;
                let result = emit_sign_extend(script, value, 16, 32)?;
                value_stack.push(result);
            }
            Operator::I64Extend8S => {
                let value = pop_value(&mut value_stack, "i64.extend8_s operand")?;
                let result = emit_sign_extend(script, value, 8, 64)?;
                value_stack.push(result);
            }
            Operator::I64Extend16S => {
                let value = pop_value(&mut value_stack, "i64.extend16_s operand")?;
                let result = emit_sign_extend(script, value, 16, 64)?;
                value_stack.push(result);
            }
            Operator::I64Extend32S => {
                let value = pop_value(&mut value_stack, "i64.extend32_s operand")?;
                let result = emit_sign_extend(script, value, 32, 64)?;
                value_stack.push(result);
            }
            Operator::Block { .. } => {
                control_stack.push(ControlFrame {
                    kind: ControlKind::Block,
                    stack_height: value_stack.len(),
                    result_count: 0,  // Blocks don't affect function return
                    start_offset: script.len(),
                    end_fixups: Vec::new(),
                    if_false_fixup: None,
                    has_else: false,
                });
                // Entering a new block resets unreachable state
                is_unreachable = false;
            }
            Operator::Loop { .. } => {
                control_stack.push(ControlFrame {
                    kind: ControlKind::Loop,
                    stack_height: value_stack.len(),
                    result_count: 0,  // Loops don't affect function return
                    start_offset: script.len(),
                    end_fixups: Vec::new(),
                    if_false_fixup: None,
                    has_else: false,
                });
                // Entering a new loop resets unreachable state
                is_unreachable = false;
            }
            Operator::If { .. } => {
                let _cond = pop_value(&mut value_stack, "if condition")?;
                // Condition already materialised on stack
                let jump_pos = emit_jump_placeholder(script, "JMPIFNOT_L")?;
                control_stack.push(ControlFrame {
                    kind: ControlKind::If,
                    stack_height: value_stack.len(),
                    result_count: 0,  // If blocks don't affect function return
                    start_offset: script.len(),
                    end_fixups: Vec::new(),
                    if_false_fixup: Some(jump_pos),
                    has_else: false,
                });
                // Entering IF resets unreachable state
                is_unreachable = false;
            }
            Operator::Else => {
                let frame = control_stack
                    .last_mut()
                    .ok_or_else(|| anyhow!("ELSE without matching IF"))?;
                if !matches!(frame.kind, ControlKind::If) {
                    bail!("ELSE can only appear within an IF block");
                }
                if let Some(pos) = frame.if_false_fixup.take() {
                    patch_jump(script, pos, script.len())?;
                }
                // Jump over else body when the THEN branch executes
                let jump_end = emit_jump_placeholder(script, "JMP_L")?;
                frame.end_fixups.push(jump_end);
                frame.has_else = true;
                frame.start_offset = script.len();
                value_stack.truncate(frame.stack_height);
                // Entering ELSE resets unreachable state
                is_unreachable = false;
            }
            Operator::End => {
                let frame = control_stack
                    .pop()
                    .ok_or_else(|| anyhow!("END without matching block"))?;

                match frame.kind {
                    ControlKind::If => {
                        if let Some(pos) = frame.if_false_fixup {
                            patch_jump(script, pos, script.len())?;
                        }
                    }
                    ControlKind::Loop | ControlKind::Block => {}
                    ControlKind::Function => {
                        // This is the final END of the function
                        // Don't truncate value stack or patch fixups for function-level frame
                        continue;
                    }
                }
                for fixup in frame.end_fixups {
                    patch_jump(script, fixup, script.len())?;
                }
                value_stack.truncate(frame.stack_height);
                // Code after END is reachable again
                is_unreachable = false;
            }
            Operator::Br { relative_depth } => {
                handle_branch(
                    script,
                    &mut value_stack,
                    &mut control_stack,
                    relative_depth as usize,
                    false,
                    &mut is_unreachable,
                )?;
            }
            Operator::BrIf { relative_depth } => {
                let _cond = pop_value(&mut value_stack, "br_if condition")?;
                handle_branch(
                    script,
                    &mut value_stack,
                    &mut control_stack,
                    relative_depth as usize,
                    true,
                    &mut is_unreachable,
                )?;
            }
            Operator::BrTable { targets } => {
                let index = pop_value(&mut value_stack, "br_table index")?;
                let mut target_depths: Vec<usize> = Vec::with_capacity(targets.len() as usize);
                for target in targets.targets() {
                    target_depths.push(target? as usize);
                }
                let default_depth = targets.default() as usize;
                handle_br_table(
                    script,
                    &mut value_stack,
                    &mut control_stack,
                    index,
                    &target_depths,
                    default_depth,
                    &mut is_unreachable,
                )?;
            }
            Operator::MemorySize { mem, .. } => {
                ensure_memory_access(runtime, mem)?;
                runtime.emit_memory_init_call(script)?;
                script.push(lookup_opcode("LDSFLD2")?.byte);
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::I32Load { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    4,
                    None,
                    32,
                    "i32.load",
                )?;
            }
            Operator::I64Load { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    8,
                    None,
                    64,
                    "i64.load",
                )?;
            }
            Operator::I32Load8S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load8_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    1,
                    Some((8, 32)),
                    32,
                    "i32.load8_s",
                )?;
            }
            Operator::I32Load8U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load8_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    1,
                    None,
                    32,
                    "i32.load8_u",
                )?;
            }
            Operator::I32Load16S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load16_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    2,
                    Some((16, 32)),
                    32,
                    "i32.load16_s",
                )?;
            }
            Operator::I32Load16U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load16_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    2,
                    None,
                    32,
                    "i32.load16_u",
                )?;
            }
            Operator::I64Load8S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load8_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    1,
                    Some((8, 64)),
                    64,
                    "i64.load8_s",
                )?;
            }
            Operator::I64Load8U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load8_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    1,
                    None,
                    64,
                    "i64.load8_u",
                )?;
            }
            Operator::I64Load16S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load16_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    2,
                    Some((16, 64)),
                    64,
                    "i64.load16_s",
                )?;
            }
            Operator::I64Load16U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load16_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    2,
                    None,
                    64,
                    "i64.load16_u",
                )?;
            }
            Operator::I64Load32S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load32_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    4,
                    Some((32, 64)),
                    64,
                    "i64.load32_s",
                )?;
            }
            Operator::I64Load32U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load32_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    4,
                    None,
                    64,
                    "i64.load32_u",
                )?;
            }
            Operator::I32Store { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i32.store value")?;
                let addr = pop_value(&mut value_stack, "i32.store address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    4,
                    "i32.store",
                )?;
            }
            Operator::I64Store { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i64.store value")?;
                let addr = pop_value(&mut value_stack, "i64.store address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    8,
                    "i64.store",
                )?;
            }
            Operator::I32Store8 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i32.store8 value")?;
                let addr = pop_value(&mut value_stack, "i32.store8 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    1,
                    "i32.store8",
                )?;
            }
            Operator::I32Store16 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i32.store16 value")?;
                let addr = pop_value(&mut value_stack, "i32.store16 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    2,
                    "i32.store16",
                )?;
            }
            Operator::I64Store8 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i64.store8 value")?;
                let addr = pop_value(&mut value_stack, "i64.store8 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    1,
                    "i64.store8",
                )?;
            }
            Operator::I64Store16 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i64.store16 value")?;
                let addr = pop_value(&mut value_stack, "i64.store16 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    2,
                    "i64.store16",
                )?;
            }
            Operator::I64Store32 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i64.store32 value")?;
                let addr = pop_value(&mut value_stack, "i64.store32 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    4,
                    "i64.store32",
                )?;
            }
            Operator::MemoryGrow { mem, .. } => {
                ensure_memory_access(runtime, mem)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_memory_grow_call(script)?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::MemoryFill { mem, .. } => {
                let len = pop_value(&mut value_stack, "memory.fill len")?;
                let value = pop_value(&mut value_stack, "memory.fill value")?;
                let dest = pop_value(&mut value_stack, "memory.fill dest")?;
                translate_memory_fill(script, runtime, dest, value, len, mem)
                    .context("failed to translate memory.fill")?;
            }
            Operator::MemoryCopy {
                dst_mem, src_mem, ..
            } => {
                let len = pop_value(&mut value_stack, "memory.copy len")?;
                let src = pop_value(&mut value_stack, "memory.copy src")?;
                let dest = pop_value(&mut value_stack, "memory.copy dest")?;
                translate_memory_copy(script, runtime, dest, src, len, dst_mem, src_mem)
                    .context("failed to translate memory.copy")?;
            }
            Operator::MemoryInit {
                data_index, mem, ..
            } => {
                let len = pop_value(&mut value_stack, "memory.init len")?;
                let src = pop_value(&mut value_stack, "memory.init offset")?;
                let dest = pop_value(&mut value_stack, "memory.init dest")?;
                translate_memory_init(script, runtime, dest, src, len, data_index, mem)
                    .context("failed to translate memory.init")?;
            }
            Operator::DataDrop { data_index } => {
                translate_data_drop(script, runtime, data_index)
                    .context("failed to translate data.drop")?;
            }
            Operator::TableGet { table } => {
                let _ = pop_value(&mut value_stack, "table.get index")?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Get(table as usize))?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::TableSet { table } => {
                let _ = pop_value(&mut value_stack, "table.set value")?;
                let _ = pop_value(&mut value_stack, "table.set index")?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Set(table as usize))?;
            }
            Operator::TableSize { table } => {
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Size(table as usize))?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::TableGrow { table } => {
                let _delta = pop_value(&mut value_stack, "table.grow delta")?;
                let _value = pop_value(&mut value_stack, "table.grow value")?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Grow(table as usize))?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::TableFill { table } => {
                let _len = pop_value(&mut value_stack, "table.fill len")?;
                let _value = pop_value(&mut value_stack, "table.fill value")?;
                let _dst = pop_value(&mut value_stack, "table.fill dest")?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Fill(table as usize))?;
            }
            Operator::TableCopy {
                dst_table,
                src_table,
            } => {
                let _len = pop_value(&mut value_stack, "table.copy len")?;
                let _src = pop_value(&mut value_stack, "table.copy src")?;
                let _dst = pop_value(&mut value_stack, "table.copy dest")?;
                runtime.table_slot(dst_table as usize)?;
                runtime.table_slot(src_table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(
                    script,
                    TableHelperKind::Copy {
                        dst: dst_table as usize,
                        src: src_table as usize,
                    },
                )?;
            }
            Operator::TableInit { table, elem_index } => {
                let _len = pop_value(&mut value_stack, "table.init len")?;
                let _src = pop_value(&mut value_stack, "table.init offset")?;
                let _dst = pop_value(&mut value_stack, "table.init dest")?;
                runtime.ensure_passive_element(elem_index as usize)?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(
                    script,
                    TableHelperKind::InitFromPassive {
                        table: table as usize,
                        segment: elem_index as usize,
                    },
                )?;
            }
            Operator::ElemDrop { elem_index } => {
                runtime.ensure_passive_element(elem_index as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime
                    .emit_table_helper(script, TableHelperKind::ElemDrop(elem_index as usize))?;
            }
            Operator::I32Eq => {
                let rhs = pop_value(&mut value_stack, "i32.eq rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.eq lhs")?;
                let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                    Some(if a == b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::I32Ne => {
                let rhs = pop_value(&mut value_stack, "i32.ne rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.ne lhs")?;
                let result = emit_binary_op(script, "NOTEQUAL", lhs, rhs, |a, b| {
                    Some(if a != b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::I32LtS => {
                let rhs = pop_value(&mut value_stack, "i32.lt_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.lt_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Lt)?;
                value_stack.push(result);
            }
            Operator::I32LtU => {
                let rhs = pop_value(&mut value_stack, "i32.lt_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.lt_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Lt)?;
                value_stack.push(result);
            }
            Operator::I32LeS => {
                let rhs = pop_value(&mut value_stack, "i32.le_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.le_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Le)?;
                value_stack.push(result);
            }
            Operator::I32LeU => {
                let rhs = pop_value(&mut value_stack, "i32.le_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.le_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Le)?;
                value_stack.push(result);
            }
            Operator::I32GtS => {
                let rhs = pop_value(&mut value_stack, "i32.gt_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.gt_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Gt)?;
                value_stack.push(result);
            }
            Operator::I32GtU => {
                let rhs = pop_value(&mut value_stack, "i32.gt_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.gt_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Gt)?;
                value_stack.push(result);
            }
            Operator::I32GeS => {
                let rhs = pop_value(&mut value_stack, "i32.ge_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.ge_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Ge)?;
                value_stack.push(result);
            }
            Operator::I32GeU => {
                let rhs = pop_value(&mut value_stack, "i32.ge_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.ge_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Ge)?;
                value_stack.push(result);
            }
            Operator::I64Eq => {
                let rhs = pop_value(&mut value_stack, "i64.eq rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.eq lhs")?;
                let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                    Some(if a == b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::I64Ne => {
                let rhs = pop_value(&mut value_stack, "i64.ne rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.ne lhs")?;
                let result = emit_binary_op(script, "NOTEQUAL", lhs, rhs, |a, b| {
                    Some(if a != b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::I64LtS => {
                let rhs = pop_value(&mut value_stack, "i64.lt_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.lt_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Lt)?;
                value_stack.push(result);
            }
            Operator::I64LtU => {
                let rhs = pop_value(&mut value_stack, "i64.lt_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.lt_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Lt)?;
                value_stack.push(result);
            }
            Operator::I64LeS => {
                let rhs = pop_value(&mut value_stack, "i64.le_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.le_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Le)?;
                value_stack.push(result);
            }
            Operator::I64LeU => {
                let rhs = pop_value(&mut value_stack, "i64.le_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.le_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Le)?;
                value_stack.push(result);
            }
            Operator::I64GtS => {
                let rhs = pop_value(&mut value_stack, "i64.gt_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.gt_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Gt)?;
                value_stack.push(result);
            }
            Operator::I64GtU => {
                let rhs = pop_value(&mut value_stack, "i64.gt_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.gt_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Gt)?;
                value_stack.push(result);
            }
            Operator::I64GeS => {
                let rhs = pop_value(&mut value_stack, "i64.ge_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.ge_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Ge)?;
                value_stack.push(result);
            }
            Operator::I64GeU => {
                let rhs = pop_value(&mut value_stack, "i64.ge_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.ge_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Ge)?;
                value_stack.push(result);
            }
            Operator::Select => {
                let condition = pop_value(&mut value_stack, "select condition")?;
                let false_value = pop_value(&mut value_stack, "select false value")?;
                let true_value = pop_value(&mut value_stack, "select true value")?;
                let result = emit_select(script, true_value, false_value, condition)?;
                value_stack.push(result);
            }
            Operator::TypedSelect { ty } => {
                ensure_select_type_supported(&[ty])?;
                let condition = pop_value(&mut value_stack, "typed select condition")?;
                let false_value = pop_value(&mut value_stack, "typed select false value")?;
                let true_value = pop_value(&mut value_stack, "typed select true value")?;
                let result = emit_select(script, true_value, false_value, condition)?;
                value_stack.push(result);
            }
            Operator::TypedSelectMulti { tys } => {
                ensure_select_type_supported(&tys)?;
                let condition = pop_value(&mut value_stack, "typed select condition")?;
                let false_value = pop_value(&mut value_stack, "typed select false value")?;
                let true_value = pop_value(&mut value_stack, "typed select true value")?;
                let result = emit_select(script, true_value, false_value, condition)?;
                value_stack.push(result);
            }
            Operator::I32Eqz => {
                let value = pop_value(&mut value_stack, "i32.eqz operand")?;
                let result = emit_eqz(script, value)?;
                value_stack.push(result);
            }
            Operator::I64Eqz => {
                let value = pop_value(&mut value_stack, "i64.eqz operand")?;
                let result = emit_eqz(script, value)?;
                value_stack.push(result);
            }
            Operator::LocalGet { local_index } => {
                let state = local_states
                    .get(local_index as usize)
                    .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
                let value = emit_local_get(script, state)?;
                value_stack.push(value);
            }
            Operator::LocalSet { local_index } => {
                let value = pop_value(&mut value_stack, "local.set operand")?;
                let state = local_states
                    .get_mut(local_index as usize)
                    .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
                emit_local_set(script, state, &value)?;
            }
            Operator::LocalTee { local_index } => {
                let value = pop_value(&mut value_stack, "local.tee operand")?;
                let state = local_states
                    .get_mut(local_index as usize)
                    .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
                emit_local_set(script, state, &value)?;
                let value = emit_local_get(script, state)?;
                value_stack.push(value);
            }
            Operator::GlobalGet { global_index } => {
                let idx = global_index as usize;
                let const_value = runtime.global_const_value(idx)?;
                if let Some(value) = const_value {
                    let entry = emit_push_int(script, value);
                    value_stack.push(entry);
                } else {
                    runtime.emit_memory_init_call(script)?;
                    let slot = runtime.global_slot(idx)?;
                    emit_load_static(script, slot)?;
                    value_stack.push(StackValue {
                        const_value: None,
                        bytecode_start: None,
                    });
                }
            }
            Operator::GlobalSet { global_index } => {
                let idx = global_index as usize;
                if !runtime.global_mutable(idx)? {
                    bail!("global {} is immutable", idx);
                }
                let _value = pop_value(&mut value_stack, "global.set operand")?;
                runtime.emit_memory_init_call(script)?;
                let slot = runtime.global_slot(idx)?;
                emit_store_static(script, slot)?;
                runtime.clear_global_const(idx)?;
            }
            Operator::Return => {
                script.push(RET);
                emitted_ret = true;
                value_stack.clear();
            }
            Operator::Call { function_index } => {
                let param_count = if let Some(import) = imports.get(function_index as usize) {
                    let type_index = get_import_type_index(import)?;
                    types
                        .get(type_index as usize)
                        .ok_or_else(|| {
                            anyhow!(
                                "invalid type index {} for import {}",
                                type_index,
                                import.name
                            )
                        })?
                        .params()
                        .len()
                } else {
                    let defined_index = (function_index as usize)
                        .checked_sub(imports.len())
                        .ok_or_else(|| anyhow!("function index underflow"))?;
                    let type_index =
                        func_type_indices
                            .get(defined_index)
                            .copied()
                            .ok_or_else(|| {
                                anyhow!("no type index recorded for function {}", function_index)
                            })?;
                    types
                        .get(type_index as usize)
                        .ok_or_else(|| {
                            anyhow!(
                                "invalid type index {} for function {}",
                                type_index,
                                function_index
                            )
                        })?
                        .params()
                        .len()
                };

                let mut params = Vec::with_capacity(param_count);
                for _ in 0..param_count {
                    params.push(pop_value(&mut value_stack, "call argument")?);
                }
                params.reverse();

                if let Some(import) = imports.get(function_index as usize) {
                    handle_import_call(function_index, script, imports, types, &params)?;
                    let type_index = get_import_type_index(import)?;
                    let func_sig = types.get(type_index as usize).ok_or_else(|| {
                        anyhow!(
                            "invalid type index {} for import {}",
                            type_index,
                            import.name
                        )
                    })?;
                    if !func_sig.results().is_empty() {
                        value_stack.push(StackValue {
                            const_value: None,
                            bytecode_start: None,
                        });
                    }
                } else {
                    let defined_index = (function_index as usize) - imports.len();
                    let type_index =
                        func_type_indices
                            .get(defined_index)
                            .copied()
                            .ok_or_else(|| {
                                anyhow!("no type index recorded for function {}", function_index)
                            })?;
                    let func_sig = types.get(type_index as usize).ok_or_else(|| {
                        anyhow!(
                            "invalid type index {} for function {}",
                            type_index,
                            function_index
                        )
                    })?;
                    if func_sig.params().len() != params.len() {
                        bail!(
                            "function {} expects {} argument(s) but {} were provided",
                            function_index,
                            func_sig.params().len(),
                            params.len()
                        );
                    }
                    if func_sig.results().len() > 1 {
                        bail!(
                            "multi-value returns are not supported (function {} returns {} values)",
                            function_index,
                            func_sig.results().len()
                        );
                    }
                    functions.emit_call(script, function_index as usize)?;
                    if !func_sig.results().is_empty() {
                        value_stack.push(StackValue {
                            const_value: None,
                            bytecode_start: None,
                        });
                    }
                }
            }
            Operator::CallIndirect {
                table_index,
                type_index,
            } => {
                tables.get(table_index as usize).ok_or_else(|| {
                    anyhow!("call_indirect references missing table {}", table_index)
                })?;

                let func_sig = types.get(type_index as usize).ok_or_else(|| {
                    anyhow!("type index {} out of bounds for call_indirect", type_index)
                })?;

                for ty in func_sig.params() {
                    match ty {
                        ValType::I32 | ValType::I64 => {}
                        other => bail!(
                            "call_indirect with unsupported parameter type {:?}; only i32/i64 are supported",
                            other
                        ),
                    }
                }
                if func_sig.results().len() > 1 {
                    bail!("call_indirect returning multiple values is not supported");
                }

                let _table_index_value = pop_value(&mut value_stack, "call_indirect table index")?;

                let mut params = Vec::with_capacity(func_sig.params().len());
                for _ in 0..func_sig.params().len() {
                    params.push(pop_value(&mut value_stack, "call_indirect argument")?);
                }
                params.reverse();

                runtime.emit_memory_init_call(script)?;
                runtime.table_slot(table_index as usize)?;
                runtime.emit_table_helper(script, TableHelperKind::Get(table_index as usize))?;

                script.push(lookup_opcode("DUP")?.byte);
                let _ = emit_push_int(script, FUNCREF_NULL);
                script.push(lookup_opcode("EQUAL")?.byte);
                let trap_null = emit_jump_placeholder(script, "JMPIF_L")?;
                script.push(lookup_opcode("DROP")?.byte);

                let total_functions = imports.len() + func_type_indices.len();
                let mut case_fixups: Vec<(usize, CallTarget)> = Vec::new();
                for fn_index in 0..total_functions {
                    let candidate_type_index = if fn_index < imports.len() {
                        get_import_type_index(&imports[fn_index])?
                    } else {
                        let defined_index = fn_index - imports.len();
                        *func_type_indices.get(defined_index).ok_or_else(|| {
                            anyhow!(
                                "call_indirect target function {} missing type entry",
                                fn_index
                            )
                        })?
                    };

                    if candidate_type_index != type_index {
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

                let mut end_fixups: Vec<usize> = Vec::new();
                for (jump, target) in case_fixups {
                    let label = script.len();
                    patch_jump(script, jump, label)?;
                    script.push(lookup_opcode("DROP")?.byte);
                    match target {
                        CallTarget::Import(idx) => {
                            handle_import_call(idx, script, imports, types, &params)?;
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

                if !func_sig.results().is_empty() {
                    value_stack.push(StackValue {
                        const_value: None,
                        bytecode_start: None,
                    });
                }
            }
            Operator::Drop => {
                let value = pop_value(&mut value_stack, "drop operand")?;
                if let Some(start) = value.bytecode_start {
                    script.truncate(start);
                } else {
                    let drop = lookup_opcode("DROP")?;
                    script.push(drop.byte);
                }
            }
            Operator::RefEq => {
                let rhs = pop_value(&mut value_stack, "ref.eq rhs")?;
                let lhs = pop_value(&mut value_stack, "ref.eq lhs")?;
                let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                    Some(if a == b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::RefAsNonNull => {
                let value = pop_value(&mut value_stack, "ref.as_non_null operand")?;
                if let Some(constant) = value.const_value {
                    if constant == FUNCREF_NULL {
                        let abort = lookup_opcode("ABORT")?;
                        script.push(abort.byte);
                        value_stack.clear();
                    } else {
                        value_stack.push(value);
                    }
                } else {
                    let dup = lookup_opcode("DUP")?;
                    script.push(dup.byte);
                    let _ = emit_push_int(script, FUNCREF_NULL);
                    script.push(lookup_opcode("EQUAL")?.byte);
                    let skip_trap = emit_jump_placeholder(script, "JMPIFNOT_L")?;
                    script.push(lookup_opcode("DROP")?.byte);
                    script.push(lookup_opcode("ABORT")?.byte);
                    let continue_label = script.len();
                    patch_jump(script, skip_trap, continue_label)?;
                    value_stack.push(value);
                }
            }
            Operator::RefNull { hty } => match hty {
                HeapType::FUNC => {
                    let entry = emit_push_int(script, FUNCREF_NULL);
                    value_stack.push(entry);
                }
                other => bail!(
                    "ref.null with heap type {:?} is unsupported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                    other
                ),
            },
            Operator::RefIsNull => {
                let value = pop_value(&mut value_stack, "ref.is_null operand")?;
                let sentinel = emit_push_int(script, FUNCREF_NULL);
                let result = emit_binary_op(script, "EQUAL", value, sentinel, |a, b| {
                    Some(if a == b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::RefFunc { function_index } => {
                let total_functions = imports.len() + func_type_indices.len();
                if function_index as usize >= total_functions {
                    bail!(
                        "ref.func references unknown function index {} (total functions: {})",
                        function_index,
                        total_functions
                    );
                }
                let entry = emit_push_int(script, function_index as i128);
                value_stack.push(entry);
            }
            Operator::Unreachable => {
                let abort = lookup_opcode("ABORT")?;
                script.push(abort.byte);
                value_stack.clear();
            }
            _ => {
                if let Some(desc) = describe_float_op(&op) {
                    return numeric::unsupported_float(&desc);
                }
                if let Some(desc) = describe_simd_op(&op) {
                    return numeric::unsupported_simd(&desc);
                }
                bail!(format!("unsupported Wasm operator {:?} ({}).", op, UNSUPPORTED_FEATURE_DOC));
            }
        }
    }

    if !emitted_ret {
        script.push(RET);
    }

    if let Some(frame) = control_stack.last() {
        bail!(
            "unclosed block detected at end of function (kind: {:?})",
            frame.kind
        );
    }

    Ok(return_kind.unwrap_or_else(|| "Void".to_string()))
}

fn pop_value(stack: &mut Vec<StackValue>, context: &str) -> Result<StackValue> {
    stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow while processing {}", context))
}

fn pop_value_polymorphic(
    stack: &mut Vec<StackValue>,
    is_unreachable: bool,
    context: &str,
) -> Result<StackValue> {
    if is_unreachable {
        // In unreachable code, stack operations are polymorphic
        // Return a dummy value that represents "any type"
        Ok(stack.pop().unwrap_or(StackValue {
            const_value: None,
            bytecode_start: None,
        }))
    } else {
        stack
            .pop()
            .ok_or_else(|| anyhow!("stack underflow while processing {}", context))
    }
}

fn lookup_opcode(name: &str) -> Result<&'static opcodes::OpcodeInfo> {
    opcodes::lookup(name).ok_or_else(|| anyhow!("unknown NeoVM opcode '{}'", name))
}

fn emit_binary_op(
    script: &mut Vec<u8>,
    opcode_name: &str,
    lhs: StackValue,
    rhs: StackValue,
    combine: impl FnOnce(i128, i128) -> Option<i128>,
) -> Result<StackValue> {
    let opcode = lookup_opcode(opcode_name)?;
    script.push(opcode.byte);
    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => combine(a, b),
        _ => None,
    };
    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

fn emit_eqz(script: &mut Vec<u8>, value: StackValue) -> Result<StackValue> {
    if let (Some(constant), Some(start)) = (value.const_value, value.bytecode_start) {
        script.truncate(start);
        let result = if constant == 0 { 1 } else { 0 };
        return Ok(emit_push_int(script, result));
    }

    let push0 = lookup_opcode("PUSH0")?;
    script.push(push0.byte);
    let equal = lookup_opcode("EQUAL")?;
    script.push(equal.byte);
    Ok(StackValue {
        const_value: None,
        bytecode_start: None,
    })
}

#[derive(Clone, Copy)]
enum ShiftKind {
    Arithmetic,
    Logical,
}

fn emit_shift_right(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: ShiftKind,
) -> Result<StackValue> {
    mask_shift_amount(script, bits)?;
    match kind {
        ShiftKind::Arithmetic => {
            script.push(lookup_opcode("SHR")?.byte);
        }
        ShiftKind::Logical => {
            let swap = lookup_opcode("SWAP")?;
            script.push(swap.byte);
            let mask = ((1u128 << bits) - 1) as i128;
            let _ = emit_push_int(script, mask);
            script.push(lookup_opcode("AND")?.byte);
            script.push(swap.byte);
            script.push(lookup_opcode("SHR")?.byte);
        }
    }

    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => {
            let shift = (b as u32) & (bits - 1);
            match kind {
                ShiftKind::Arithmetic => {
                    if bits == 32 {
                        Some(((a as i32) >> shift) as i128)
                    } else {
                        Some(((a as i64) >> shift) as i128)
                    }
                }
                ShiftKind::Logical => {
                    let mask = (1u128 << bits) - 1;
                    let unsigned = (a as u128) & mask;
                    Some((unsigned >> shift) as i128)
                }
            }
        }
        _ => None,
    };

    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

fn emit_push_int(buffer: &mut Vec<u8>, value: i128) -> StackValue {
    let start = buffer.len();
    match value {
        -1 => buffer.push(PUSHM1),
        0 => buffer.push(PUSH0),
        1..=16 => buffer.push(PUSH_BASE + value as u8),
        v if v >= i8::MIN as i128 && v <= i8::MAX as i128 => {
            buffer.push(PUSHINT8);
            buffer.push(v as i8 as u8);
        }
        v if v >= i16::MIN as i128 && v <= i16::MAX as i128 => {
            buffer.push(PUSHINT16);
            buffer.extend_from_slice(&(v as i16).to_le_bytes());
        }
        v if v >= i32::MIN as i128 && v <= i32::MAX as i128 => {
            buffer.push(PUSHINT32);
            buffer.extend_from_slice(&(v as i32).to_le_bytes());
        }
        v if v >= i64::MIN as i128 && v <= i64::MAX as i128 => {
            buffer.push(PUSHINT64);
            buffer.extend_from_slice(&(v as i64).to_le_bytes());
        }
        _ => {
            buffer.push(PUSHINT128);
            buffer.extend_from_slice(&value.to_le_bytes());
        }
    }

    StackValue {
        const_value: Some(value),
        bytecode_start: Some(start),
    }
}

fn emit_rotate(
    script: &mut Vec<u8>,
    value: StackValue,
    shift: StackValue,
    bits: u32,
    left: bool,
) -> Result<StackValue> {
    match (
        value.const_value,
        shift.const_value,
        value.bytecode_start,
        shift.bytecode_start,
    ) {
        (Some(v), Some(s), Some(value_start), Some(shift_start)) => {
            let mask = match bits {
                32 => 31,
                64 => 63,
                _ => unreachable!(),
            };
            let rotate = match bits {
                32 => {
                    let val = v as i32;
                    let amt = (s as u32) & mask;
                    if left {
                        val.rotate_left(amt) as i128
                    } else {
                        val.rotate_right(amt) as i128
                    }
                }
                64 => {
                    let val = v as i64;
                    let amt = (s as u32) & mask;
                    if left {
                        val.rotate_left(amt) as i128
                    } else {
                        val.rotate_right(amt) as i128
                    }
                }
                _ => unreachable!(),
            };
            script.truncate(value_start.min(shift_start));
            Ok(emit_push_int(script, rotate))
        }
        _ => emit_rotate_dynamic(script, value, shift, bits, left),
    }
}

fn emit_rotate_dynamic(
    script: &mut Vec<u8>,
    value: StackValue,
    shift: StackValue,
    bits: u32,
    left: bool,
) -> Result<StackValue> {
    let mut stack = vec![value, shift];

    let mask_sv = emit_push_int(script, (bits - 1) as i128);
    stack.push(mask_sv);
    apply_binary(script, &mut stack, "AND", |a, b| Some(a & b))?;

    stack_pick(script, &mut stack, 1)?;
    stack_pick(script, &mut stack, 1)?;

    if left {
        apply_binary(script, &mut stack, "SHL", |a, b| {
            let shift = (b as u32) & (bits - 1);
            if bits == 32 {
                Some(((a as i32) << shift) as i128)
            } else {
                Some(((a as i64) << shift) as i128)
            }
        })?
    } else {
        apply_shift_right(script, &mut stack, bits, ShiftKind::Logical)?
    };

    stack_pick(script, &mut stack, 2)?;
    stack_pick(script, &mut stack, 2)?;

    let bits_sv = emit_push_int(script, bits as i128);
    stack.push(bits_sv);
    stack_swap(script, &mut stack)?;
    apply_binary(script, &mut stack, "SUB", |a, b| Some(a - b))?;

    if left {
        apply_shift_right(script, &mut stack, bits, ShiftKind::Logical)?
    } else {
        apply_binary(script, &mut stack, "SHL", |a, b| {
            let shift = (b as u32) & (bits - 1);
            if bits == 32 {
                Some(((a as i32) << shift) as i128)
            } else {
                Some(((a as i64) << shift) as i128)
            }
        })?
    };

    let result = apply_binary(script, &mut stack, "OR", |a, b| Some(a | b))?;

    stack_swap(script, &mut stack)?;
    stack_drop(script, &mut stack)?;
    stack_swap(script, &mut stack)?;
    stack_drop(script, &mut stack)?;

    Ok(result)
}

fn apply_binary(
    script: &mut Vec<u8>,
    stack: &mut Vec<StackValue>,
    opcode: &str,
    combine: impl FnOnce(i128, i128) -> Option<i128>,
) -> Result<StackValue> {
    let rhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for {} rhs", opcode))?;
    let lhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for {} lhs", opcode))?;
    let result = emit_binary_op(script, opcode, lhs, rhs, combine)?;
    let clone = result.clone();
    stack.push(result);
    Ok(clone)
}

fn apply_shift_right(
    script: &mut Vec<u8>,
    stack: &mut Vec<StackValue>,
    bits: u32,
    kind: ShiftKind,
) -> Result<StackValue> {
    let rhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for shift rhs"))?;
    let lhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for shift lhs"))?;
    let result = emit_shift_right(script, lhs, rhs, bits, kind)?;
    let clone = result.clone();
    stack.push(result);
    Ok(clone)
}

fn stack_pick(script: &mut Vec<u8>, stack: &mut Vec<StackValue>, index: usize) -> Result<()> {
    let idx_sv = emit_push_int(script, index as i128);
    stack.push(idx_sv);
    script.push(lookup_opcode("PICK")?.byte);
    let len = stack.len();
    if index >= len - 1 {
        bail!("PICK index {} out of range", index);
    }
    let picked = stack[len - 2 - index].clone();
    stack.pop();
    stack.push(StackValue {
        const_value: picked.const_value,
        bytecode_start: None,
    });
    Ok(())
}

fn stack_swap(script: &mut Vec<u8>, stack: &mut Vec<StackValue>) -> Result<()> {
    if stack.len() < 2 {
        bail!("SWAP requires at least two stack values");
    }
    script.push(lookup_opcode("SWAP")?.byte);
    let len = stack.len();
    stack.swap(len - 1, len - 2);
    Ok(())
}

fn stack_drop(script: &mut Vec<u8>, stack: &mut Vec<StackValue>) -> Result<()> {
    if stack.is_empty() {
        bail!("DROP requires at least one stack value");
    }
    script.push(lookup_opcode("DROP")?.byte);
    stack.pop();
    Ok(())
}

fn mask_shift_amount(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    if bits == 0 {
        return Ok(());
    }
    let mask = (bits - 1) as i128;
    let _ = emit_push_int(script, mask);
    script.push(lookup_opcode("AND")?.byte);
    Ok(())
}

fn emit_local_get(script: &mut Vec<u8>, state: &LocalState) -> Result<StackValue> {
    if let Some(value) = state.const_value {
        return Ok(emit_push_int(script, value));
    }

    match state.kind {
        LocalKind::Param(index) => emit_load_arg(script, index)?,
        LocalKind::Local(slot) => emit_load_local_slot(script, slot)?,
    }

    Ok(StackValue {
        const_value: None,
        bytecode_start: None,
    })
}

fn emit_local_set(script: &mut Vec<u8>, state: &mut LocalState, value: &StackValue) -> Result<()> {
    match state.kind {
        LocalKind::Param(index) => emit_store_arg(script, index)?,
        LocalKind::Local(slot) => emit_store_local_slot(script, slot)?,
    }
    state.const_value = value.const_value;
    Ok(())
}

/// Extract type index from Import's TypeRef
fn get_import_type_index(import: &FunctionImport) -> Result<u32> {
    Ok(import.type_index)
}

/// Helper to emit indexed opcodes (LDARG, STARG, LDLOC, STLOC)
/// NeoVM has optimized opcodes for indices 0-6 (e.g., LDARG0-LDARG6)
fn emit_indexed_opcode(
    script: &mut Vec<u8>,
    base_opcode: &str,
    _indexed_base: &str,
    index: u32,
) -> Result<()> {
    // Try optimized indexed opcode first (0-6)
    if index <= 6 {
        let indexed_name = format!("{}{}", base_opcode, index);
        if let Some(opcode) = opcodes::lookup(&indexed_name) {
            script.push(opcode.byte);
            return Ok(());
        }
    }

    // Fall back to base opcode with explicit index
    let opcode =
        opcodes::lookup(base_opcode).ok_or_else(|| anyhow!("unknown opcode: {}", base_opcode))?;

    script.push(opcode.byte);
    script.push(index as u8);
    Ok(())
}

fn emit_load_arg(script: &mut Vec<u8>, index: u32) -> Result<()> {
    emit_indexed_opcode(script, "LDARG", "LDARG", index)
}

fn emit_store_arg(script: &mut Vec<u8>, index: u32) -> Result<()> {
    emit_indexed_opcode(script, "STARG", "STARG", index)
}

fn emit_load_local_slot(script: &mut Vec<u8>, slot: u32) -> Result<()> {
    emit_indexed_opcode(script, "LDLOC", "LDLOC", slot)
}

fn emit_store_local_slot(script: &mut Vec<u8>, slot: u32) -> Result<()> {
    emit_indexed_opcode(script, "STLOC", "STLOC", slot)
}

fn wasm_val_type_to_manifest(ty: &ValType) -> Result<String> {
    let repr = match ty {
        ValType::I32 => "Integer",
        ValType::I64 => "Integer",
        ValType::F32 | ValType::F64 => return numeric::unsupported_float("manifest numeric type"),
        ValType::V128 => return numeric::unsupported_simd("manifest v128"),
        ValType::Ref(_) => return numeric::unsupported_reference_type("manifest reference type"),
    };
    Ok(repr.to_string())
}

fn handle_branch(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut Vec<ControlFrame>,
    relative_depth: usize,
    conditional: bool,
    is_unreachable: &mut bool,
) -> Result<()> {
    if relative_depth >= control_stack.len() {
        bail!(
            "branch depth {} exceeds current control stack",
            relative_depth
        );
    }
    let target_index = control_stack.len() - 1 - relative_depth;
    let (prefix, _) = control_stack.split_at_mut(target_index + 1);
    let frame = &mut prefix[target_index];

    // Only validate stack height if we're not already in unreachable code
    // For Function frames: branching means providing return values, validate against result_count
    // For other frames: branching means jumping to end of block, validate against stack_height
    if !*is_unreachable {
        let expected = if frame.kind == ControlKind::Function {
            frame.result_count
        } else {
            frame.stack_height
        };
        if value_stack.len() != expected {
            bail!(
                "branch requires {} values but current stack has {}",
                expected,
                value_stack.len()
            );
        }
    }

    match frame.kind {
        ControlKind::Loop => {
            let opcode = if conditional { "JMPIF_L" } else { "JMP_L" };
            emit_jump_to(script, opcode, frame.start_offset)?;
        }
        _ => {
            let opcode = if conditional { "JMPIF_L" } else { "JMP_L" };
            let pos = emit_jump_placeholder(script, opcode)?;
            frame.end_fixups.push(pos);
        }
    }

    if !conditional {
        // For Function frames, keep result_count values on stack for return
        // For other frames, truncate to stack_height
        let target_size = if frame.kind == ControlKind::Function {
            frame.result_count
        } else {
            frame.stack_height
        };
        value_stack.truncate(target_size);
        // Unconditional branch makes subsequent code unreachable
        *is_unreachable = true;
    }

    Ok(())
}

fn handle_import_call(
    function_index: u32,
    script: &mut Vec<u8>,
    imports: &[FunctionImport],
    types: &[FuncType],
    params: &[StackValue],
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
    match module.as_str() {
        "opcode" => emit_opcode_call(import, func_type, params, script),
        "syscall" => {
            emit_syscall_call(import, script)?;
            Ok(())
        }
        "neo" => {
            emit_neo_syscall(import, script)?;
            Ok(())
        }
        other => bail!("unsupported import module '{}::{}'", other, import.name),
    }
}

fn emit_opcode_call(
    import: &FunctionImport,
    func_type: &FuncType,
    params: &[StackValue],
    script: &mut Vec<u8>,
) -> Result<()> {
    if import.name.eq_ignore_ascii_case("raw") {
        ensure_param_count(import, params, 1)?;
        let value = truncate_literal(params.last().unwrap(), script, 1)? as u8;
        script.push(value);
        return Ok(());
    }

    if import.name.eq_ignore_ascii_case("raw4") {
        ensure_param_count(import, params, 1)?;
        let value = truncate_literal(params.last().unwrap(), script, 4)? as i64;
        script.extend_from_slice(&(value as u32).to_le_bytes());
        return Ok(());
    }

    let info = opcodes::lookup(&import.name)
        .ok_or_else(|| anyhow!("unknown NeoVM opcode '{}'", import.name))?;

    if info.operand_size_prefix != 0 {
        bail!(
            "opcode '{}' has a variable-size operand; emit it manually via opcode.raw/raw4",
            import.name
        );
    }

    if !func_type.results().is_empty() {
        bail!(
            "imported opcode '{}' must have signature (param ..., result void)",
            import.name
        );
    }

    let expected_params = func_type.params().len();
    if expected_params != params.len() {
        bail!(
            "imported opcode '{}' expects {} parameter(s) but {} were provided",
            import.name,
            expected_params,
            params.len()
        );
    }

    if info.operand_size == 0 {
        if !params.is_empty() {
            bail!("opcode '{}' does not take immediate operands", import.name);
        }
        script.push(info.byte);
        return Ok(());
    }

    ensure_param_count(import, params, 1)?;
    let immediate = truncate_literal(params.last().unwrap(), script, info.operand_size as usize)?;

    script.push(info.byte);
    match info.operand_size {
        1 => script.push(immediate as u8),
        2 => script.extend_from_slice(&(immediate as i16).to_le_bytes()),
        4 => script.extend_from_slice(&(immediate as i32).to_le_bytes()),
        8 => script.extend_from_slice(&(immediate as i64).to_le_bytes()),
        other => {
            bail!(
                "unsupported operand size {} for opcode '{}'; use opcode.raw/raw4",
                other,
                import.name
            );
        }
    }

    Ok(())
}

fn emit_syscall_call(import: &FunctionImport, script: &mut Vec<u8>) -> Result<()> {
    let syscall = syscalls::lookup(&import.name)
        .ok_or_else(|| anyhow!("unknown syscall '{}'", import.name))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;

    // SYSCALL has a 4-byte immediate hash.
    if opcode.operand_size != 4 || opcode.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }

    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(())
}

fn emit_neo_syscall(import: &FunctionImport, script: &mut Vec<u8>) -> Result<()> {
    let syscall_name = neo_syscalls::lookup_neo_syscall(&import.name)
        .ok_or_else(|| anyhow!("unknown Neo syscall import '{}'", import.name))?;
    let syscall = syscalls::lookup(syscall_name)
        .ok_or_else(|| anyhow!("syscall '{}' not found", syscall_name))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;

    if opcode.operand_size != 4 || opcode.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }

    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(())
}

fn ensure_param_count(
    import: &FunctionImport,
    params: &[StackValue],
    expected: usize,
) -> Result<()> {
    if params.len() != expected {
        bail!(
            "import '{}' expects {} parameter(s) but {} were provided",
            import.name,
            expected,
            params.len()
        );
    }
    Ok(())
}

fn truncate_literal(param: &StackValue, script: &mut Vec<u8>, max_bytes: usize) -> Result<i128> {
    let value = param
        .const_value
        .ok_or_else(|| anyhow!("import argument must be a compile-time constant"))?;
    if let Some(start) = param.bytecode_start {
        if start > script.len() {
            bail!("internal error: literal start beyond current script length");
        }
        script.truncate(start);
    } else {
        bail!("import argument cannot be materialised as an immediate; ensure it is a literal");
    }

    // Treat the literal as signed; validate it fits within the requested bytes.
    let bits = max_bytes * 8;
    if bits == 0 || bits > 64 {
        bail!("unsupported immediate width {} bytes", max_bytes);
    }
    let min = -(1i128 << (bits - 1));
    let max_signed = (1i128 << (bits - 1)) - 1;
    let max_unsigned = (1i128 << bits) - 1;
    if !(min <= value && value <= max_signed) && !(0 <= value && value <= max_unsigned) {
        bail!(
            "literal value {} does not fit in {} byte(s) for opcode immediate",
            value,
            max_bytes
        );
    }

    Ok(value)
}

#[derive(Debug, Clone)]
struct LocalState {
    kind: LocalKind,
    const_value: Option<i128>,
}

#[derive(Debug, Clone)]
enum LocalKind {
    Param(u32),
    Local(u32),
}

enum UnsignedOp {
    Div,
    Rem,
}

#[derive(Clone, Copy)]
enum CompareOp {
    Lt,
    Le,
    Gt,
    Ge,
}

impl CompareOp {
    fn opcode_name(self) -> &'static str {
        match self {
            CompareOp::Lt => "LT",
            CompareOp::Le => "LE",
            CompareOp::Gt => "GT",
            CompareOp::Ge => "GE",
        }
    }

    fn evaluate_signed(self, lhs: i128, rhs: i128, bits: u32) -> bool {
        match bits {
            32 => {
                let lhs = lhs as i32;
                let rhs = rhs as i32;
                self.evaluate_order(lhs, rhs)
            }
            64 => {
                let lhs = lhs as i64;
                let rhs = rhs as i64;
                self.evaluate_order(lhs, rhs)
            }
            other => unreachable!("unsupported signed comparison width {}", other),
        }
    }

    fn evaluate_unsigned(self, lhs: u128, rhs: u128) -> bool {
        self.evaluate_order(lhs, rhs)
    }

    fn evaluate_order<T: PartialOrd>(self, lhs: T, rhs: T) -> bool {
        match self {
            CompareOp::Lt => lhs < rhs,
            CompareOp::Le => lhs <= rhs,
            CompareOp::Gt => lhs > rhs,
            CompareOp::Ge => lhs >= rhs,
        }
    }
}

fn emit_unsigned_binary_op(
    script: &mut Vec<u8>,
    op: UnsignedOp,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
) -> Result<StackValue> {
    mask_unsigned_operands(script, bits)?;

    let opcode_name = match op {
        UnsignedOp::Div => "DIV",
        UnsignedOp::Rem => "MOD",
    };
    script.push(lookup_opcode(opcode_name)?.byte);

    let mask = (1u128 << bits) - 1;
    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => {
            let dividend = (a as u128) & mask;
            let divisor = (b as u128) & mask;
            if divisor == 0 {
                None
            } else {
                let value = match op {
                    UnsignedOp::Div => dividend / divisor,
                    UnsignedOp::Rem => dividend % divisor,
                };
                Some(value as i128)
            }
        }
        _ => None,
    };

    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

fn emit_signed_compare(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: CompareOp,
) -> Result<StackValue> {
    emit_binary_op(script, kind.opcode_name(), lhs, rhs, |a, b| {
        let cmp = kind.evaluate_signed(a, b, bits);
        Some(if cmp { 1 } else { 0 })
    })
}

fn emit_unsigned_compare(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: CompareOp,
) -> Result<StackValue> {
    mask_unsigned_operands(script, bits)?;
    let mask = (1u128 << bits) - 1;
    emit_binary_op(script, kind.opcode_name(), lhs, rhs, |a, b| {
        let lhs = (a as u128) & mask;
        let rhs = (b as u128) & mask;
        let cmp = kind.evaluate_unsigned(lhs, rhs);
        Some(if cmp { 1 } else { 0 })
    })
}

fn mask_unsigned_operands(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    let mask_value = ((1u128 << bits) - 1) as i128;
    let and = lookup_opcode("AND")?;
    let swap = lookup_opcode("SWAP")?;

    let _ = emit_push_int(script, mask_value);
    script.push(and.byte);
    script.push(swap.byte);
    let _ = emit_push_int(script, mask_value);
    script.push(and.byte);
    script.push(swap.byte);

    Ok(())
}

fn ensure_select_type_supported(tys: &[ValType]) -> Result<()> {
    if tys.len() > 1 {
        bail!("typed select results with more than one value are not supported");
    }
    for ty in tys {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!(
                "typed select with unsupported value type {:?}; only i32/i64 are supported",
                other
            ),
        }
    }
    Ok(())
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

    script.push(RET);

    Ok(())
}

fn ensure_memory_access(runtime: &RuntimeHelpers, mem_index: u32) -> Result<()> {
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

fn evaluate_offset_expr(expr: ConstExpr<'_>) -> Result<i64> {
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

fn evaluate_global_init(expr: ConstExpr<'_>, value_type: ValType) -> Result<i128> {
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

fn translate_memory_load(
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

    let raw_value = StackValue {
        const_value: None,
        bytecode_start: None,
    };

    let result = if let Some((from_bits, to_bits)) = sign_extend {
        emit_sign_extend(script, raw_value, from_bits, to_bits)?
    } else {
        if result_bits < bytes * 8 {
            bail!(
                "result bit-width {} smaller than load width {}",
                result_bits,
                bytes * 8
            );
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

fn translate_memory_store(
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

fn translate_memory_fill(
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

fn translate_memory_copy(
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

fn translate_memory_init(
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

fn translate_data_drop(
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

fn infer_contract_tokens(script: &[u8]) -> Vec<MethodToken> {
    use crate::opcodes;
    use crate::syscalls;
    use MethodToken;

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

fn describe_float_op(op: &Operator) -> Option<String> {
    let name = format!("{:?}", op);
    if name.starts_with("F32") || name.starts_with("F64") {
        return Some(name.to_lowercase());
    }
    None
}

fn describe_simd_op(op: &Operator) -> Option<String> {
    const PREFIXES: &[&str] = &["I8x16", "I16x8", "I32x4", "I64x2", "F32x4", "F64x2", "V128"];
    let name = format!("{:?}", op);
    if PREFIXES.iter().any(|prefix| name.starts_with(prefix)) {
        return Some(name.to_lowercase());
    }
    None
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

fn emit_select(
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

fn emit_zero_extend(script: &mut Vec<u8>, value: StackValue, bits: u32) -> Result<StackValue> {
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

fn emit_bit_count(
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

fn emit_sign_extend(
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

fn handle_br_table(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut Vec<ControlFrame>,
    index: StackValue,
    targets: &[usize],
    default_depth: usize,
    is_unreachable: &mut bool,
) -> Result<()> {
    if let Some(const_idx) = index.const_value {
        if let Some(start) = index.bytecode_start {
            script.truncate(start);
        } else {
            script.push(lookup_opcode("DROP")?.byte);
        }
        let idx = if const_idx < 0 || const_idx > usize::MAX as i128 {
            usize::MAX
        } else {
            const_idx as usize
        };
        let depth = targets.get(idx).copied().unwrap_or(default_depth);
        handle_branch(
            script,
            value_stack,
            control_stack,
            depth,
            false,
            is_unreachable,
        )?;
        return Ok(());
    }

    emit_br_table_dynamic(
        script,
        value_stack,
        control_stack,
        targets,
        default_depth,
        is_unreachable,
    )
}

fn emit_br_table_dynamic(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut Vec<ControlFrame>,
    targets: &[usize],
    default_depth: usize,
    is_unreachable: &mut bool,
) -> Result<()> {
    let dup = lookup_opcode("DUP")?.byte;
    let equal = lookup_opcode("EQUAL")?.byte;
    let drop = lookup_opcode("DROP")?.byte;

    let mut case_fixups: Vec<(usize, usize)> = Vec::with_capacity(targets.len());

    for (case_index, &depth) in targets.iter().enumerate() {
        script.push(dup);
        let _ = emit_push_int(script, case_index as i128);
        script.push(equal);
        let fixup = emit_jump_placeholder(script, "JMPIF_L")?;
        case_fixups.push((fixup, depth));
    }

    script.push(drop);
    handle_branch(
        script,
        value_stack,
        control_stack,
        default_depth,
        false,
        is_unreachable,
    )?;

    for (fixup, depth) in case_fixups {
        let label_pos = script.len();
        patch_jump(script, fixup, label_pos)?;
        script.push(drop);
        handle_branch(
            script,
            value_stack,
            control_stack,
            depth,
            false,
            is_unreachable,
        )?;
    }

    Ok(())
}
