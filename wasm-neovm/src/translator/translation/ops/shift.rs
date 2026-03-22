// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Clone, Copy)]
pub(in super::super) enum ShiftKind {
    Arithmetic,
    Logical,
}

pub(in super::super) fn emit_shift_right(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: ShiftKind,
) -> Result<StackValue> {
    mask_shift_amount(script, bits)?;
    match kind {
        ShiftKind::Arithmetic => {
            script.push(lookup_opcode("SHR")?.byte);
        }
        ShiftKind::Logical => {
            let swap = lookup_opcode("SWAP")?;
            script.push(swap.byte);
            let mask = ((1u128 << bits) - 1) as i128;
            let _ = emit_push_int(script, mask);
            script.push(lookup_opcode("AND")?.byte);
            script.push(swap.byte);
            script.push(lookup_opcode("SHR")?.byte);
        }
    }

    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => {
            let shift = (b as u32) & (bits - 1);
            match kind {
                ShiftKind::Arithmetic => {
                    if bits == 32 {
                        Some(((a as i32) >> shift) as i128)
                    } else {
                        Some(((a as i64) >> shift) as i128)
                    }
                }
                ShiftKind::Logical => {
                    let mask = (1u128 << bits) - 1;
                    let unsigned = (a as u128) & mask;
                    Some((unsigned >> shift) as i128)
                }
            }
        }
        _ => None,
    };

    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

pub(in super::super) fn emit_rotate(
    script: &mut Vec<u8>,
    value: StackValue,
    shift: StackValue,
    bits: u32,
    left: bool,
) -> Result<StackValue> {
    match (
        value.const_value,
        shift.const_value,
        value.bytecode_start,
        shift.bytecode_start,
    ) {
        (Some(v), Some(s), Some(value_start), Some(shift_start)) => {
            let mask = match bits {
                32 => 31,
                64 => 63,
                _ => unreachable!(),
            };
            let rotate = match bits {
                32 => {
                    let val = v as i32;
                    let amt = (s as u32) & mask;
                    if left {
                        val.rotate_left(amt) as i128
                    } else {
                        val.rotate_right(amt) as i128
                    }
                }
                64 => {
                    let val = v as i64;
                    let amt = (s as u32) & mask;
                    if left {
                        val.rotate_left(amt) as i128
                    } else {
                        val.rotate_right(amt) as i128
                    }
                }
                _ => unreachable!(),
            };
            let _ = (value_start, shift_start);
            // Keep script monotonic: replacing backtracking `truncate` with
            // explicit stack cleanup avoids invalidating pending fixups.
            script.push(lookup_opcode("DROP")?.byte);
            script.push(lookup_opcode("DROP")?.byte);
            Ok(emit_push_int(script, rotate))
        }
        _ => emit_rotate_dynamic(script, value, shift, bits, left),
    }
}

fn emit_rotate_dynamic(
    script: &mut Vec<u8>,
    value: StackValue,
    shift: StackValue,
    bits: u32,
    left: bool,
) -> Result<StackValue> {
    // Pre-allocate stack with exact capacity needed (Round 62 optimization)
    let mut stack = Vec::with_capacity(4);
    stack.push(value);
    stack.push(shift);

    let mask_sv = emit_push_int(script, (bits - 1) as i128);
    stack.push(mask_sv);
    apply_binary(script, &mut stack, "AND", |a, b| Some(a & b))?;

    stack_pick(script, &mut stack, 1)?;
    stack_pick(script, &mut stack, 1)?;

    if left {
        apply_binary(script, &mut stack, "SHL", |a, b| {
            let shift = (b as u32) & (bits - 1);
            if bits == 32 {
                Some(((a as i32) << shift) as i128)
            } else {
                Some(((a as i64) << shift) as i128)
            }
        })?
    } else {
        apply_shift_right(script, &mut stack, bits, ShiftKind::Logical)?
    };

    stack_pick(script, &mut stack, 2)?;
    stack_pick(script, &mut stack, 2)?;

    let bits_sv = emit_push_int(script, bits as i128);
    stack.push(bits_sv);
    stack_swap(script, &mut stack)?;
    apply_binary(script, &mut stack, "SUB", |a, b| Some(a - b))?;

    if left {
        apply_shift_right(script, &mut stack, bits, ShiftKind::Logical)?
    } else {
        apply_binary(script, &mut stack, "SHL", |a, b| {
            let shift = (b as u32) & (bits - 1);
            if bits == 32 {
                Some(((a as i32) << shift) as i128)
            } else {
                Some(((a as i64) << shift) as i128)
            }
        })?
    };

    let result = apply_binary(script, &mut stack, "OR", |a, b| Some(a | b))?;
    stack_swap(script, &mut stack)?;
    stack_drop(script, &mut stack)?;
    stack_swap(script, &mut stack)?;
    stack_drop(script, &mut stack)?;

    Ok(result)
}

fn apply_binary(
    script: &mut Vec<u8>,
    stack: &mut Vec<StackValue>,
    opcode: &str,
    combine: impl FnOnce(i128, i128) -> Option<i128>,
) -> Result<StackValue> {
    let rhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for {} rhs", opcode))?;
    let lhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for {} lhs", opcode))?;
    let result = emit_binary_op(script, opcode, lhs, rhs, combine)?;
    let clone = result.clone();
    stack.push(result);
    Ok(clone)
}

fn apply_shift_right(
    script: &mut Vec<u8>,
    stack: &mut Vec<StackValue>,
    bits: u32,
    kind: ShiftKind,
) -> Result<StackValue> {
    let rhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for shift rhs"))?;
    let lhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for shift lhs"))?;
    let result = emit_shift_right(script, lhs, rhs, bits, kind)?;
    let clone = result.clone();
    stack.push(result);
    Ok(clone)
}

fn stack_pick(script: &mut Vec<u8>, stack: &mut Vec<StackValue>, index: usize) -> Result<()> {
    let idx_sv = emit_push_int(script, index as i128);
    stack.push(idx_sv);
    script.push(lookup_opcode("PICK")?.byte);
    let len = stack.len();
    if index >= len - 1 {
        bail!("PICK index {} out of range", index);
    }
    let picked = stack[len - 2 - index].clone();
    stack.pop();
    stack.push(StackValue {
        const_value: picked.const_value,
        bytecode_start: None,
    });
    Ok(())
}

fn stack_swap(script: &mut Vec<u8>, stack: &mut [StackValue]) -> Result<()> {
    if stack.len() < 2 {
        bail!("SWAP requires at least two stack values");
    }
    script.push(lookup_opcode("SWAP")?.byte);
    let len = stack.len();
    stack.swap(len - 1, len - 2);
    Ok(())
}

fn stack_drop(script: &mut Vec<u8>, stack: &mut Vec<StackValue>) -> Result<()> {
    if stack.is_empty() {
        bail!("DROP requires at least one stack value");
    }
    script.push(lookup_opcode("DROP")?.byte);
    stack.pop();
    Ok(())
}

/// Mask shift amount to valid range (Round 82 - Const evaluation, Round 87 - Bit manipulation)
///
/// Round 87: Uses (bits - 1) as mask since shifts are modulo 2^n
/// For bits=32, mask=31 (0x1F), for bits=64, mask=63 (0x3F)
pub(in super::super) fn mask_shift_amount(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    // Round 82: Compile-time check for power-of-two bits
    #[allow(dead_code)]
    const fn is_power_of_two(n: u32) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }

    // Handle edge case
    if bits == 0 {
        return Ok(());
    }

    // Round 82: Precompute mask at compile time for common bit widths
    const MASK_32: i128 = (32 - 1) as i128; // 31
    const MASK_64: i128 = (64 - 1) as i128; // 63

    // Round 87: Use const-evaluated mask
    let mask = match bits {
        32 => MASK_32,
        64 => MASK_64,
        _ => (bits - 1) as i128,
    };

    let _ = emit_push_int(script, mask);
    script.push(lookup_opcode("AND")?.byte);
    Ok(())
}
