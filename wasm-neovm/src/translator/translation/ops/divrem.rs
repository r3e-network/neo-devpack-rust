use super::*;

pub(in super::super) fn emit_abort_on_zero_divisor(script: &mut Vec<u8>) -> Result<()> {
    // Stack before: [..., dividend, divisor]
    // Trap if divisor == 0.
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let ok = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("ABORT")?.byte);
    let end = script.len();
    patch_jump(script, ok, end)?;
    Ok(())
}

pub(in super::super) fn emit_abort_on_signed_div_overflow(
    script: &mut Vec<u8>,
    bits: u32,
) -> Result<()> {
    // WebAssembly traps on signed division overflow: INT_MIN / -1.
    let min = match bits {
        32 => i32::MIN as i128,
        64 => i64::MIN as i128,
        other => bail!("unsupported signed division width {}", other),
    };

    // Stack before: [..., dividend, divisor]
    // Compute: (dividend == MIN) && (divisor == -1)
    script.push(lookup_opcode("OVER")?.byte);
    let _ = emit_push_int(script, min);
    script.push(lookup_opcode("EQUAL")?.byte);
    script.push(lookup_opcode("OVER")?.byte);
    script.push(lookup_opcode("PUSHM1")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    script.push(lookup_opcode("BOOLAND")?.byte);

    let ok = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("ABORT")?.byte);
    let end = script.len();
    patch_jump(script, ok, end)?;
    Ok(())
}

pub(in super::super) enum UnsignedOp {
    Div,
    Rem,
}

pub(in super::super) fn emit_unsigned_binary_op(
    script: &mut Vec<u8>,
    op: UnsignedOp,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
) -> Result<StackValue> {
    mask_unsigned_operands(script, bits)?;
    emit_abort_on_zero_divisor(script)?;

    let opcode_name = match op {
        UnsignedOp::Div => "DIV",
        UnsignedOp::Rem => "MOD",
    };
    script.push(lookup_opcode(opcode_name)?.byte);

    let mask = (1u128 << bits) - 1;
    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => {
            let dividend = (a as u128) & mask;
            let divisor = (b as u128) & mask;
            if divisor == 0 {
                None
            } else {
                let value = match op {
                    UnsignedOp::Div => dividend / divisor,
                    UnsignedOp::Rem => dividend % divisor,
                };
                Some(value as i128)
            }
        }
        _ => None,
    };

    let result = StackValue {
        const_value,
        bytecode_start: None,
    };
    emit_sign_extend(script, result, bits, bits)
}

pub(super) fn mask_unsigned_operands(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    let mask_value = ((1u128 << bits) - 1) as i128;
    let and = lookup_opcode("AND")?;
    let swap = lookup_opcode("SWAP")?;

    let _ = emit_push_int(script, mask_value);
    script.push(and.byte);
    script.push(swap.byte);
    let _ = emit_push_int(script, mask_value);
    script.push(and.byte);
    script.push(swap.byte);

    Ok(())
}
