// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};

use super::lookup_opcode;

/// Emit a TRY_L instruction with placeholder catch offset
pub(crate) fn emit_try_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    script.push(lookup_opcode("TRY_L")?.byte);
    let placeholder_pos = script.len();

    // Emit 8-byte placeholder (4 bytes for catch offset, 4 bytes for finally offset)
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Emit an ENDTRY_L instruction with placeholder offset
pub(crate) fn emit_endtry_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    script.push(lookup_opcode("ENDTRY_L")?.byte);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder for end offset
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Patch a TRY_L instruction with catch and finally offsets
pub(crate) fn patch_try_catch(
    script: &mut [u8],
    position: usize,
    catch_offset: usize,
) -> Result<()> {
    if position == 0 || position + 8 > script.len() {
        bail!(
            "invalid try patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // TRY_L offsets are relative to the beginning of the current instruction.
    let opcode_pos = position as i32 - 1;
    let catch_rel = (catch_offset as i32) - opcode_pos;
    let catch_bytes = catch_rel.to_le_bytes();

    // Patch catch offset (first 4 bytes)
    script[position..position + 4].copy_from_slice(&catch_bytes);

    // Leave finally offset as 0 (no finally block) - last 4 bytes
    script[position + 4..position + 8].copy_from_slice(&[0, 0, 0, 0]);

    Ok(())
}

/// Patch an ENDTRY_L instruction with the end offset
pub(crate) fn patch_endtry(script: &mut [u8], position: usize, end_offset: usize) -> Result<()> {
    if position == 0 || position + 4 > script.len() {
        bail!(
            "invalid endtry patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    // ENDTRY_L offsets are relative to the beginning of the current instruction.
    let opcode_pos = position as i32 - 1;
    let offset = (end_offset as i32) - opcode_pos;
    let bytes = offset.to_le_bytes();

    // Patch the 4-byte placeholder
    script[position..position + 4].copy_from_slice(&bytes);

    Ok(())
}
