// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    _runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    match op {
        Operator::I32Shl => {
            let rhs = super::pop_value(value_stack, "i32.shl rhs")?;
            let lhs = super::pop_value(value_stack, "i32.shl lhs")?;
            mask_shift_amount(script, 32)?;
            let result = emit_binary_op(script, "SHL", lhs, rhs, |a, b| {
                let shift = (b as u32) & 31;
                Some(((a as i32) << shift) as i128)
            })?;
            let result = emit_sign_extend(script, result, 32, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32ShrS => {
            let rhs = super::pop_value(value_stack, "i32.shr_s rhs")?;
            let lhs = super::pop_value(value_stack, "i32.shr_s lhs")?;
            let result = emit_shift_right(script, lhs, rhs, 32, ShiftKind::Arithmetic)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32ShrU => {
            let rhs = super::pop_value(value_stack, "i32.shr_u rhs")?;
            let lhs = super::pop_value(value_stack, "i32.shr_u lhs")?;
            let result = emit_shift_right(script, lhs, rhs, 32, ShiftKind::Logical)?;
            let result = emit_sign_extend(script, result, 32, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Rotl => {
            let rhs = super::pop_value(value_stack, "i32.rotl rhs")?;
            let lhs = super::pop_value(value_stack, "i32.rotl lhs")?;
            let result = emit_rotate(script, lhs, rhs, 32, true)?;
            let result = emit_sign_extend(script, result, 32, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Rotr => {
            let rhs = super::pop_value(value_stack, "i32.rotr rhs")?;
            let lhs = super::pop_value(value_stack, "i32.rotr lhs")?;
            let result = emit_rotate(script, lhs, rhs, 32, false)?;
            let result = emit_sign_extend(script, result, 32, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Shl => {
            let rhs = super::pop_value(value_stack, "i64.shl rhs")?;
            let lhs = super::pop_value(value_stack, "i64.shl lhs")?;
            mask_shift_amount(script, 64)?;
            let result = emit_binary_op(script, "SHL", lhs, rhs, |a, b| {
                let shift = (b as u32) & 63;
                Some(((a as i64) << shift) as i128)
            })?;
            let result = emit_sign_extend(script, result, 64, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64ShrS => {
            let rhs = super::pop_value(value_stack, "i64.shr_s rhs")?;
            let lhs = super::pop_value(value_stack, "i64.shr_s lhs")?;
            let result = emit_shift_right(script, lhs, rhs, 64, ShiftKind::Arithmetic)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64ShrU => {
            let rhs = super::pop_value(value_stack, "i64.shr_u rhs")?;
            let lhs = super::pop_value(value_stack, "i64.shr_u lhs")?;
            let result = emit_shift_right(script, lhs, rhs, 64, ShiftKind::Logical)?;
            let result = emit_sign_extend(script, result, 64, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Rotl => {
            let rhs = super::pop_value(value_stack, "i64.rotl rhs")?;
            let lhs = super::pop_value(value_stack, "i64.rotl lhs")?;
            let result = emit_rotate(script, lhs, rhs, 64, true)?;
            let result = emit_sign_extend(script, result, 64, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Rotr => {
            let rhs = super::pop_value(value_stack, "i64.rotr rhs")?;
            let lhs = super::pop_value(value_stack, "i64.rotr lhs")?;
            let result = emit_rotate(script, lhs, rhs, 64, false)?;
            let result = emit_sign_extend(script, result, 64, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        _ => Ok(false),
    }
}
