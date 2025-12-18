use anyhow::{bail, Result};

use super::lookup_opcode;

/// Emit a jump instruction with a placeholder target that will be patched later
pub(crate) fn emit_jump_placeholder(script: &mut Vec<u8>, opcode: &str) -> Result<usize> {
    let opcode_byte = lookup_opcode(opcode)?.byte;

    script.push(opcode_byte);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder (will be patched later with actual offset)
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Emit a short jump instruction (1-byte relative offset) with a placeholder target.
pub(crate) fn emit_jump_placeholder_short(script: &mut Vec<u8>, opcode: &str) -> Result<usize> {
    let opcode_byte = lookup_opcode(opcode)?.byte;
    script.push(opcode_byte);
    let placeholder_pos = script.len();
    script.push(0);
    Ok(placeholder_pos)
}

/// Patch a previously emitted jump instruction with the actual target offset
pub(crate) fn patch_jump(script: &mut [u8], position: usize, target: usize) -> Result<()> {
    if position == 0 || position + 4 > script.len() {
        bail!(
            "invalid jump patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // NeoVM jump offsets are relative to the beginning of the current instruction.
    // `position` points at the first byte of the 4-byte operand, so the opcode is at `position - 1`.
    let opcode_pos = position as i32 - 1;
    let offset = (target as i32) - opcode_pos;
    let bytes = offset.to_le_bytes();

    // Patch the 4-byte placeholder
    script[position..position + 4].copy_from_slice(&bytes);

    Ok(())
}

/// Patch a short jump instruction (1-byte relative offset).
pub(crate) fn patch_jump_short(script: &mut [u8], position: usize, target: usize) -> Result<()> {
    if position == 0 || position >= script.len() {
        bail!(
            "invalid short jump patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // NeoVM jump offsets are relative to the beginning of the current instruction.
    let opcode_pos = position as i32 - 1;
    let offset = (target as i32) - opcode_pos;
    if offset < i8::MIN as i32 || offset > i8::MAX as i32 {
        bail!(
            "short jump target {} out of range for opcode at {} (offset {})",
            target,
            opcode_pos,
            offset
        );
    }

    script[position] = offset as i8 as u8;
    Ok(())
}

/// Emit a jump to a specific target offset
pub(crate) fn emit_jump_to(script: &mut Vec<u8>, opcode: &str, target: usize) -> Result<()> {
    script.push(lookup_opcode(opcode)?.byte);
    let current_pos = script.len();

    // NeoVM jump offsets are relative to the beginning of the current instruction.
    let opcode_pos = current_pos as i32 - 1;
    let offset = (target as i32) - opcode_pos;
    let bytes = offset.to_le_bytes();

    // Emit the offset
    script.extend_from_slice(&bytes);

    Ok(())
}
