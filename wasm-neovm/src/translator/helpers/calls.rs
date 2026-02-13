use anyhow::Result;

use super::offsets::emit_placeholder;

/// Emit a CALL_L placeholder for runtime helper calls
///
/// This is a convenience wrapper that defaults to CALL_L opcode
pub fn emit_call_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    emit_placeholder(script, "CALL_L")
}

/// Patch a previously emitted call with the actual target offset
///
/// Re-exported from offsets module for backward compatibility
pub use super::offsets::patch_offset as patch_call;
