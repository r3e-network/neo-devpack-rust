// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;
use super::util::{mask_top_bits, sign_extend_const, truncate_to_bits};

pub(crate) fn emit_select(
    script: &mut Vec<u8>,
    true_value: StackValue,
    false_value: StackValue,
    condition: StackValue,
) -> Result<StackValue> {
    let jmp_false = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("DROP")?.byte);
    let jmp_end = emit_jump_placeholder(script, "JMP_L")?;
    let else_target = script.len();
    patch_jump(script, jmp_false, else_target)?;
    script.push(lookup_opcode("NIP")?.byte);
    let end_target = script.len();
    patch_jump(script, jmp_end, end_target)?;

    let const_value = match condition.const_value {
        Some(value) if value != 0 => true_value.const_value,
        Some(_) => false_value.const_value,
        None => match (true_value.const_value, false_value.const_value) {
            (Some(a), Some(b)) if a == b => Some(a),
            _ => None,
        },
    };

    Ok(StackValue {
        const_value,
        bytecode_start: None,
        pending_sign_extend: None,
    })
}

pub(crate) fn emit_zero_extend(
    script: &mut Vec<u8>,
    value: StackValue,
    bits: u32,
) -> Result<StackValue> {
    let const_result = value.const_value.map(|c| truncate_to_bits(c, bits));

    if let (Some(result), Some(_start)) = (const_result, value.bytecode_start) {
        // Avoid bytecode backtracking (`truncate`) because it can invalidate
        // pending control-flow fixup positions captured earlier.
        script.push(lookup_opcode("DROP")?.byte);
        return Ok(emit_push_int(script, result));
    }

    mask_top_bits(script, bits)?;
    Ok(StackValue {
        const_value: const_result,
        bytecode_start: None,
        pending_sign_extend: None,
    })
}

pub(crate) fn emit_bit_count(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value: StackValue,
    kind: BitHelperKind,
) -> Result<StackValue> {
    let bits = kind.bits();
    if let Some(constant) = value.const_value {
        let result = match kind {
            BitHelperKind::Clz(_) => clz_const(constant, bits),
            BitHelperKind::Ctz(_) => ctz_const(constant, bits),
            BitHelperKind::Popcnt(_) => popcnt_const(constant, bits),
        };

        if let Some(_start) = value.bytecode_start {
            script.push(lookup_opcode("DROP")?.byte);
            return Ok(emit_push_int(script, result));
        }
    }

    runtime.emit_bit_helper(script, kind)?;
    Ok(StackValue {
        const_value: None,
        bytecode_start: None,
        pending_sign_extend: None,
    })
}

fn popcnt_const(value: i128, bits: u32) -> i128 {
    let masked = truncate_to_bits(value, bits);
    match bits {
        32 => (masked as u32).count_ones() as i128,
        64 => (masked as u64).count_ones() as i128,
        _ => unreachable!("unsupported bit-width {} for popcnt", bits),
    }
}

fn ctz_const(value: i128, bits: u32) -> i128 {
    let masked = truncate_to_bits(value, bits);
    if masked == 0 {
        return bits as i128;
    }
    match bits {
        32 => (masked as u32).trailing_zeros() as i128,
        64 => (masked as u64).trailing_zeros() as i128,
        _ => unreachable!("unsupported bit-width {} for ctz", bits),
    }
}

fn clz_const(value: i128, bits: u32) -> i128 {
    let masked = truncate_to_bits(value, bits);
    if masked == 0 {
        return bits as i128;
    }
    match bits {
        32 => (masked as u32).leading_zeros() as i128,
        64 => (masked as u64).leading_zeros() as i128,
        _ => unreachable!("unsupported bit-width {} for clz", bits),
    }
}

pub(crate) fn emit_sign_extend(
    script: &mut Vec<u8>,
    value: StackValue,
    from_bits: u32,
    _total_bits: u32,
) -> Result<StackValue> {
    let const_result = value
        .const_value
        .map(|c| sign_extend_const(truncate_to_bits(c, from_bits), from_bits));

    if let (Some(result), Some(_start)) = (const_result, value.bytecode_start) {
        script.push(lookup_opcode("DROP")?.byte);
        return Ok(emit_push_int(script, result));
    }

    if from_bits == 0 || from_bits >= 128 {
        mask_top_bits(script, from_bits)?;
        return Ok(StackValue {
            const_value: const_result,
            bytecode_start: None,
            pending_sign_extend: None,
        });
    }

    // For bit widths >= 9, use TUCK-based algorithm (compute sign_bit first, derive mask).
    // This is 1 byte smaller because mask_top_bits uses the 5-byte SHL+DEC+AND path for >= 9 bits.
    // For bit widths <= 8, use the old approach since mask_top_bits uses a shorter literal push.
    if from_bits >= 9 {
        super::util::emit_pow2(script, from_bits - 1)?;
        script.push(lookup_opcode("TUCK")?.byte);
        script.push(lookup_opcode("DUP")?.byte);
        let _ = emit_push_int(script, 1);
        script.push(lookup_opcode("SHL")?.byte);
        script.push(lookup_opcode("DEC")?.byte);
        script.push(lookup_opcode("ROT")?.byte);
        script.push(lookup_opcode("AND")?.byte);
        script.push(lookup_opcode("XOR")?.byte);
        script.push(lookup_opcode("SWAP")?.byte);
        script.push(lookup_opcode("SUB")?.byte);
    } else {
        // Old approach: mask first, then XOR-SUB with separate sign_bit push
        mask_top_bits(script, from_bits)?;
        super::util::emit_pow2(script, from_bits - 1)?;
        script.push(lookup_opcode("SWAP")?.byte);
        script.push(lookup_opcode("OVER")?.byte);
        script.push(lookup_opcode("XOR")?.byte);
        script.push(lookup_opcode("SWAP")?.byte);
        script.push(lookup_opcode("SUB")?.byte);
    }

    Ok(StackValue {
        const_value: const_result,
        bytecode_start: None,
        pending_sign_extend: None,
    })
}

/// Like `emit_sign_extend`, but emits a CALL to a shared helper instead of
/// inlining the mask+XOR-SUB sequence. Only supports from_bits == 32 or 64,
/// and only when the value is NOT a compile-time constant.
///
/// Falls back to inline `emit_sign_extend` for constants or unsupported bit widths.
pub(crate) fn emit_sign_extend_via_helper(
    script: &mut Vec<u8>,
    runtime: &mut super::super::RuntimeHelpers,
    value: StackValue,
    from_bits: u32,
    total_bits: u32,
) -> Result<StackValue> {
    // Constants are always folded inline (cheaper than a call)
    let const_result = value
        .const_value
        .map(|c| sign_extend_const(truncate_to_bits(c, from_bits), from_bits));

    if let (Some(result), Some(_start)) = (const_result, value.bytecode_start) {
        script.push(lookup_opcode("DROP")?.byte);
        return Ok(emit_push_int(script, result));
    }

    // Use helper for 32-bit or 64-bit full sign extension
    if from_bits == 32 && total_bits == 32 {
        runtime.emit_sign_extend_32_helper(script)?;
        return Ok(StackValue {
            const_value: None,
            bytecode_start: None,
            pending_sign_extend: None,
        });
    }
    if from_bits == 64 && total_bits == 64 {
        runtime.emit_sign_extend_64_helper(script)?;
        return Ok(StackValue {
            const_value: None,
            bytecode_start: None,
            pending_sign_extend: None,
        });
    }

    // Fall back to inline for other bit widths
    emit_sign_extend(script, value, from_bits, total_bits)
}
