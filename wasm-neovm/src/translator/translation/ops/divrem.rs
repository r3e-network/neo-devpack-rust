use super::*;

/// Branch prediction macros (Round 85)
#[allow(unused_macros)]
macro_rules! likely {
    ($e:expr) => {
        $e
    };
}
#[allow(unused_macros)]
macro_rules! unlikely {
    ($e:expr) => {
        $e
    };
}

/// Round 82: Const-evaluated INT_MIN values for common bit widths
const INT_MIN_32: i128 = i32::MIN as i128;
const INT_MIN_64: i128 = i64::MIN as i128;

/// Emit abort if divisor is zero (Round 81 - inline)
#[inline]
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

/// Emit abort on signed division overflow (Round 82 - const eval, Round 81 - inline)
///
/// WebAssembly traps on signed division overflow: INT_MIN / -1.
#[inline]
pub(in super::super) fn emit_abort_on_signed_div_overflow(
    script: &mut Vec<u8>,
    bits: u32,
) -> Result<()> {
    // Round 82: Pre-computed MIN values
    let min = match bits {
        32 => INT_MIN_32,
        64 => INT_MIN_64,
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

/// Unsigned division operations (Round 82 - const eval)
#[derive(Clone, Copy)]
pub(in super::super) enum UnsignedOp {
    Div,
    Rem,
}

impl UnsignedOp {
    /// Get opcode name (Round 81 - inline)
    #[inline(always)]
    fn opcode_name(self) -> &'static str {
        match self {
            UnsignedOp::Div => "DIV",
            UnsignedOp::Rem => "MOD",
        }
    }

    /// Round 82: Const-evaluate unsigned operation
    #[inline]
    fn eval_const(self, dividend: u128, divisor: u128) -> Option<i128> {
        if divisor == 0 {
            return None;
        }
        let result = match self {
            UnsignedOp::Div => dividend / divisor,
            UnsignedOp::Rem => dividend % divisor,
        };
        Some(result as i128)
    }
}

/// Round 82: Pre-computed masks for common bit widths
const MASK_32: u128 = (1u128 << 32) - 1; // 0xFFFFFFFF
const MASK_64: u128 = (1u128 << 64) - 1; // 0xFFFFFFFFFFFFFFFF

/// Emit unsigned binary operation (Rounds 81, 82, 85, 87)
#[inline]
pub(in super::super) fn emit_unsigned_binary_op(
    script: &mut Vec<u8>,
    op: UnsignedOp,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
) -> Result<StackValue> {
    mask_unsigned_operands(script, bits)?;
    emit_abort_on_zero_divisor(script)?;

    script.push(lookup_opcode(op.opcode_name())?.byte);

    // Round 82: Use const-evaluated mask
    let mask = match bits {
        32 => MASK_32,
        64 => MASK_64,
        _ => (1u128 << bits) - 1,
    };

    // Round 85: Constant folding is common
    let const_value = if likely!(lhs.const_value.is_some() && rhs.const_value.is_some()) {
        let dividend = (lhs.const_value.unwrap() as u128) & mask;
        let divisor = (rhs.const_value.unwrap() as u128) & mask;
        op.eval_const(dividend, divisor)
    } else {
        None
    };

    let result = StackValue {
        const_value,
        bytecode_start: None,
    };
    emit_sign_extend(script, result, bits, bits)
}

/// Mask unsigned operands (Rounds 81, 82, 87)
///
/// Round 87: Use bit manipulation for masking
#[inline]
pub(super) fn mask_unsigned_operands(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    // Round 82: Const-evaluated masks
    let mask_value = match bits {
        32 => MASK_32 as i128,
        64 => MASK_64 as i128,
        _ => ((1u128 << bits) - 1) as i128,
    };

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
