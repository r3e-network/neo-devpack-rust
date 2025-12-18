use anyhow::Result;

use super::super::constants::{
    PUSH0, PUSHINT128, PUSHINT16, PUSHINT32, PUSHINT64, PUSHINT8, PUSHM1, PUSH_BASE,
};
use super::super::types::StackValue;
use super::lookup_opcode;

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
