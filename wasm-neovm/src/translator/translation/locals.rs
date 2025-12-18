use anyhow::{anyhow, bail, Result};

use crate::opcodes;

use crate::translator::helpers::emit_push_int;
use crate::translator::types::StackValue;

#[derive(Debug, Clone)]
pub(super) struct LocalState {
    pub(super) kind: LocalKind,
    pub(super) const_value: Option<i128>,
}

#[derive(Debug, Clone)]
pub(super) enum LocalKind {
    Param(u32),
    Local(u32),
}

pub(super) fn emit_local_get(script: &mut Vec<u8>, state: &LocalState) -> Result<StackValue> {
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

pub(super) fn emit_local_set(
    script: &mut Vec<u8>,
    state: &mut LocalState,
    value: &StackValue,
) -> Result<()> {
    match state.kind {
        LocalKind::Param(index) => emit_store_arg(script, index)?,
        LocalKind::Local(slot) => emit_store_local_slot(script, slot)?,
    }
    state.const_value = value.const_value;
    Ok(())
}

/// Helper to emit indexed opcodes (LDARG, STARG, LDLOC, STLOC)
/// NeoVM has optimized opcodes for indices 0-6 (e.g., LDARG0-LDARG6)
fn emit_indexed_opcode(script: &mut Vec<u8>, base_opcode: &str, index: u32) -> Result<()> {
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

    if index > u8::MAX as u32 {
        bail!(
            "{} index {} exceeds NeoVM operand limit (0-255)",
            base_opcode,
            index
        );
    }

    script.push(opcode.byte);
    script.push(index as u8);
    Ok(())
}

pub(super) fn emit_load_arg(script: &mut Vec<u8>, index: u32) -> Result<()> {
    emit_indexed_opcode(script, "LDARG", index)
}

pub(super) fn emit_store_arg(script: &mut Vec<u8>, index: u32) -> Result<()> {
    emit_indexed_opcode(script, "STARG", index)
}

fn emit_load_local_slot(script: &mut Vec<u8>, slot: u32) -> Result<()> {
    emit_indexed_opcode(script, "LDLOC", slot)
}

fn emit_store_local_slot(script: &mut Vec<u8>, slot: u32) -> Result<()> {
    emit_indexed_opcode(script, "STLOC", slot)
}
