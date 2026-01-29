use anyhow::Result;

use super::super::constants::{
    PUSH0, PUSHINT128, PUSHINT16, PUSHINT32, PUSHINT64, PUSHINT8, PUSHM1, PUSH_BASE,
};
use super::super::types::StackValue;
use super::lookup_opcode;

// Small value cache for common constants (Round 64 optimization)
// These are the most frequently pushed values in WASM contracts
const SMALL_VALUES: [u8; 18] = [
    PUSHM1,         // -1
    PUSH0,          // 0
    PUSH_BASE + 1,  // 1
    PUSH_BASE + 2,  // 2
    PUSH_BASE + 3,  // 3
    PUSH_BASE + 4,  // 4
    PUSH_BASE + 5,  // 5
    PUSH_BASE + 6,  // 6
    PUSH_BASE + 7,  // 7
    PUSH_BASE + 8,  // 8
    PUSH_BASE + 9,  // 9
    PUSH_BASE + 10, // 10
    PUSH_BASE + 11, // 11
    PUSH_BASE + 12, // 12
    PUSH_BASE + 13, // 13
    PUSH_BASE + 14, // 14
    PUSH_BASE + 15, // 15
    PUSH_BASE + 16, // 16
];

/// Emit a push int instruction with optimized fast paths (Rounds 61, 64, 67 optimizations)
pub(crate) fn emit_push_int(buffer: &mut Vec<u8>, value: i128) -> StackValue {
    let start = buffer.len();

    // Fast path for most common values: -1 to 16 (Round 64 optimization)
    // These account for ~80% of integer constants in typical WASM
    #[allow(clippy::manual_range_contains)]
    if value >= -1 && value <= 16 {
        buffer.push(SMALL_VALUES[(value + 1) as usize]);
        return StackValue {
            const_value: Some(value),
            bytecode_start: Some(start),
        };
    }

    match value {
        v if v >= i8::MIN as i128 && v <= i8::MAX as i128 => {
            buffer.push(PUSHINT8);
            buffer.push(v as i8 as u8);
        }
        v if v >= i16::MIN as i128 && v <= i16::MAX as i128 => {
            buffer.push(PUSHINT16);
            // Use SIMD-friendly unaligned write when available (Round 67 hint)
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

/// Emit a push data instruction with byte array data
pub(crate) fn emit_push_data(script: &mut Vec<u8>, data: &[u8]) -> Result<()> {
    let len = data.len();

    // Neo N3 uses PUSHDATA{1,2,4} for pushing raw byte strings.
    // (Neo Legacy supported PUSHBYTES1..75, but those byte values are not data pushes in N3.)
    if len <= 255 {
        // For data <= 255 bytes, use PUSHDATA1 (length encoded as u8)
        script.push(lookup_opcode("PUSHDATA1")?.byte);
        script.push(len as u8);
        script.extend_from_slice(data);
    } else if len <= 65535 {
        // For data <= 65535 bytes, use PUSHDATA2 (length encoded as u16)
        script.push(lookup_opcode("PUSHDATA2")?.byte);
        script.extend_from_slice(&(len as u16).to_le_bytes());
        script.extend_from_slice(data);
    } else {
        // For larger data, use PUSHDATA4 (length encoded as u32)
        script.push(lookup_opcode("PUSHDATA4")?.byte);
        script.extend_from_slice(&(len as u32).to_le_bytes());
        script.extend_from_slice(data);
    }

    Ok(())
}
