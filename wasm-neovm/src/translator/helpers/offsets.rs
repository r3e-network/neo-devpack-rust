// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Unified offset handling for jump and call instructions
//!
//! This module provides common functionality for emitting and patching
//! instruction offsets, eliminating code duplication between jumps and calls.

use anyhow::{bail, Result};

use super::lookup_opcode;

fn long_to_short_opcode(opcode: &str) -> Option<&'static str> {
    match opcode {
        "JMP_L" => Some("JMP"),
        "JMPIF_L" => Some("JMPIF"),
        "JMPIFNOT_L" => Some("JMPIFNOT"),
        "JMPEQ_L" => Some("JMPEQ"),
        "JMPNE_L" => Some("JMPNE"),
        "JMPGT_L" => Some("JMPGT"),
        "JMPGE_L" => Some("JMPGE"),
        "JMPLT_L" => Some("JMPLT"),
        "JMPLE_L" => Some("JMPLE"),
        "ENDTRY_L" => Some("ENDTRY"),
        _ => None,
    }
}

/// Offset size constants
pub const LONG_OFFSET_SIZE: usize = 4; // 4-byte offset for long jumps/calls
pub const PLACEHOLDER_BYTE: u8 = 0xFF;

/// Emit an instruction with a placeholder offset that will be patched later
///
/// # Arguments
/// * `script` - The bytecode buffer to write to
/// * `opcode` - The opcode name (e.g., "JMP_L", "CALL_L")
///
/// # Returns
/// The position of the placeholder (to be used with `patch_offset`)
pub fn emit_placeholder(script: &mut Vec<u8>, opcode: &str) -> Result<usize> {
    let opcode_byte = lookup_opcode(opcode)?.byte;
    script.push(opcode_byte);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder (will be patched later with actual offset)
    script.extend_from_slice(&[PLACEHOLDER_BYTE; LONG_OFFSET_SIZE]);

    Ok(placeholder_pos)
}

/// Patch a previously emitted instruction with the actual target offset
///
/// # Arguments
/// * `script` - The bytecode buffer containing the instruction
/// * `position` - The position returned by `emit_placeholder`
/// * `target` - The target position in the script
///
/// # Notes
/// NeoVM offsets are relative to the beginning of the current instruction.
/// `position` points at the first byte of the 4-byte operand, so the opcode is at `position - 1`.
pub fn patch_offset(script: &mut [u8], position: usize, target: usize) -> Result<()> {
    if position == 0 || position + LONG_OFFSET_SIZE > script.len() {
        bail!(
            "invalid patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

    let opcode_pos = position as i32 - 1;
    let offset = (target as i32) - opcode_pos;
    let bytes = offset.to_le_bytes();

    // Patch the 4-byte placeholder
    script[position..position + LONG_OFFSET_SIZE].copy_from_slice(&bytes);

    Ok(())
}

/// Emit a short jump instruction (1-byte relative offset) with a placeholder target
pub fn emit_placeholder_short(script: &mut Vec<u8>, opcode: &str) -> Result<usize> {
    let opcode_byte = lookup_opcode(opcode)?.byte;
    script.push(opcode_byte);
    let placeholder_pos = script.len();
    script.push(0);
    Ok(placeholder_pos)
}

/// Patch a short jump instruction (1-byte relative offset)
pub fn patch_offset_short(script: &mut [u8], position: usize, target: usize) -> Result<()> {
    if position == 0 || position >= script.len() {
        bail!(
            "invalid short patch position: {} exceeds script length {}",
            position,
            script.len()
        );
    }

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

/// Emit a jump to a specific target offset immediately
///
/// This is used when the target is already known at emission time
pub fn emit_jump_to(script: &mut Vec<u8>, opcode: &str, target: usize) -> Result<()> {
    let opcode_pos = script.len() as i32;
    let offset_i64 = i64::try_from(target)
        .map_err(|_| anyhow::anyhow!("target offset {} exceeds i64 range", target))?
        - i64::from(opcode_pos);

    if let Some(short_opcode) = long_to_short_opcode(opcode) {
        if (i8::MIN as i64..=i8::MAX as i64).contains(&offset_i64) {
            script.push(lookup_opcode(short_opcode)?.byte);
            script.push(offset_i64 as i8 as u8);
            return Ok(());
        }
    }

    let offset_i32 = i32::try_from(offset_i64)
        .map_err(|_| anyhow::anyhow!("jump delta {} exceeds i32 range", offset_i64))?;
    script.push(lookup_opcode(opcode)?.byte);
    script.extend_from_slice(&offset_i32.to_le_bytes());
    Ok(())
}
