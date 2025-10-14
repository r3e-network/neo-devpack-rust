use anyhow::{anyhow, bail, Result};

use crate::opcodes;

use super::constants::{
    PUSH0, PUSHINT128, PUSHINT16, PUSHINT32, PUSHINT64, PUSHINT8, PUSHM1, PUSH_BASE,
};
use super::types::StackValue;

pub(crate) fn lookup_opcode(name: &str) -> Result<&'static opcodes::OpcodeInfo> {
    opcodes::lookup(name).ok_or_else(|| anyhow!("unknown NeoVM opcode '{}'", name))
}

pub(crate) fn emit_push_int(buffer: &mut Vec<u8>, value: i128) -> StackValue {
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

/// Emit a jump instruction with a placeholder target that will be patched later
pub(crate) fn emit_jump_placeholder(script: &mut Vec<u8>, opcode: &str) -> Result<usize> {
    let opcode_byte = lookup_opcode(opcode)?.byte;

    script.push(opcode_byte);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder (will be patched later with actual offset)
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Patch a previously emitted jump instruction with the actual target offset
pub(crate) fn patch_jump(script: &mut Vec<u8>, position: usize, target: usize) -> Result<()> {
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
pub(crate) fn emit_call_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    script.push(lookup_opcode("CALL_L")?.byte);
    let placeholder_pos = script.len();

    // Emit 4-byte placeholder for call target
    script.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    Ok(placeholder_pos)
}

/// Patch a previously emitted call instruction with the actual function offset
pub(crate) fn patch_call(script: &mut Vec<u8>, position: usize, target: usize) -> Result<()> {
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
pub(crate) fn emit_jump_to(script: &mut Vec<u8>, opcode: &str, target: usize) -> Result<()> {
    script.push(lookup_opcode(opcode)?.byte);
    let current_pos = script.len();

    // Calculate relative offset
    let offset = (target as i32) - (current_pos as i32 + 4);
    let bytes = offset.to_le_bytes();

    // Emit the offset
    script.extend_from_slice(&bytes);

    Ok(())
}

/// Emit a load from a static field slot
pub(crate) fn emit_load_static(script: &mut Vec<u8>, slot: usize) -> Result<()> {
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
            script.push(lookup_opcode("LDSFLD")?.byte);
            script.push(slot as u8);
            return Ok(());
        }
    };

    script.push(lookup_opcode(opcode)?.byte);
    Ok(())
}

/// Emit a store to a static field slot
pub(crate) fn emit_store_static(script: &mut Vec<u8>, slot: usize) -> Result<()> {
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
            script.push(lookup_opcode("STSFLD")?.byte);
            script.push(slot as u8);
            return Ok(());
        }
    };

    script.push(lookup_opcode(opcode)?.byte);
    Ok(())
}

/// Emit a push data instruction with byte array data
pub(crate) fn emit_push_data(script: &mut Vec<u8>, data: &[u8]) -> Result<()> {
    let len = data.len();

    if len <= 75 {
        // For data <= 75 bytes, use direct push with length prefix
        script.push(len as u8);
        script.extend_from_slice(data);
    } else if len <= 255 {
        // For data <= 255 bytes, use PUSHDATA1
        script.push(lookup_opcode("PUSHDATA1")?.byte);
        script.push(len as u8);
        script.extend_from_slice(data);
    } else if len <= 65535 {
        // For data <= 65535 bytes, use PUSHDATA2
        script.push(lookup_opcode("PUSHDATA2")?.byte);
        script.extend_from_slice(&(len as u16).to_le_bytes());
        script.extend_from_slice(data);
    } else {
        // For larger data, use PUSHDATA4
        script.push(lookup_opcode("PUSHDATA4")?.byte);
        script.extend_from_slice(&(len as u32).to_le_bytes());
        script.extend_from_slice(data);
    }

    Ok(())
}

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
    script: &mut Vec<u8>,
    position: usize,
    catch_offset: usize,
) -> Result<()> {
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
pub(crate) fn patch_endtry(script: &mut Vec<u8>, position: usize, end_offset: usize) -> Result<()> {
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
