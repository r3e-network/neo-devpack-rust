use anyhow::{bail, Result};

use super::lookup_opcode;

/// Emit a call instruction with a placeholder that will be patched later
pub(crate) fn emit_call_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    script.push(lookup_opcode("CALL_L")?.byte);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder for call target
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Patch a previously emitted call instruction with the actual function offset
pub(crate) fn patch_call(script: &mut [u8], position: usize, target: usize) -> Result<()> {
    if position == 0 || position + 4 > script.len() {
        bail!(
            "invalid call patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // NeoVM CALL offsets are relative to the beginning of the current instruction.
    // `position` points at the first byte of the 4-byte operand, so the opcode is at `position - 1`.
    let opcode_pos = position as i32 - 1;
    let offset = (target as i32) - opcode_pos;
    let bytes = offset.to_le_bytes();

    // Patch the 4-byte placeholder
    script[position..position + 4].copy_from_slice(&bytes);

    Ok(())
}
