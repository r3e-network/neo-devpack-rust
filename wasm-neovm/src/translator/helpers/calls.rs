// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::Result;

use super::lookup_opcode;
use super::offsets::emit_placeholder;

/// Emit a CALL_L placeholder for runtime helper calls
///
/// This is a convenience wrapper that defaults to CALL_L opcode
pub fn emit_call_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    emit_placeholder(script, "CALL_L")
}

/// Emit a direct call to a known target.
///
/// Uses compact `CALL` when the relative displacement fits in an `i8`,
/// otherwise falls back to `CALL_L`.
pub fn emit_call_to(script: &mut Vec<u8>, target: usize) -> Result<()> {
    let opcode_pos = script.len();
    let opcode_pos_i64 = i64::try_from(opcode_pos)
        .map_err(|_| anyhow::anyhow!("script offset {} exceeds i64 range", opcode_pos))?;
    let target_i64 = i64::try_from(target)
        .map_err(|_| anyhow::anyhow!("target offset {} exceeds i64 range", target))?;
    let delta_i64 = target_i64 - opcode_pos_i64;

    if (i8::MIN as i64..=i8::MAX as i64).contains(&delta_i64) {
        script.push(lookup_opcode("CALL")?.byte);
        script.push(delta_i64 as i8 as u8);
        return Ok(());
    }

    let delta_i32 = i32::try_from(delta_i64)
        .map_err(|_| anyhow::anyhow!("call delta {} exceeds i32 range", delta_i64))?;
    script.push(lookup_opcode("CALL_L")?.byte);
    script.extend_from_slice(&delta_i32.to_le_bytes());
    Ok(())
}

/// Patch a previously emitted call with the actual target offset
///
/// Re-exported from offsets module for backward compatibility
pub use super::offsets::patch_offset as patch_call;
