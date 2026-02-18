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

    mask_top_bits(script, from_bits)?;

    if from_bits == 0 || from_bits >= 128 {
        return Ok(StackValue {
            const_value: const_result,
            bytecode_start: None,
        });
    }

    // Convert the masked unsigned value into a signed two's-complement integer:
    // if value >= 2^(from_bits-1) then value -= 2^from_bits.
    script.push(lookup_opcode("DUP")?.byte);
    super::util::emit_pow2(script, from_bits - 1)?;
    script.push(lookup_opcode("GE")?.byte);
    let skip_subtract = emit_jump_placeholder_short(script, "JMPIFNOT")?;
    super::util::emit_pow2(script, from_bits)?;
    script.push(lookup_opcode("SUB")?.byte);
    let end = script.len();
    patch_jump_short(script, skip_subtract, end)?;

    Ok(StackValue {
        const_value: const_result,
        bytecode_start: None,
    })
}
