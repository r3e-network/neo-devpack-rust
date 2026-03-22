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
        Operator::I32DivS => {
            let rhs = super::pop_value(value_stack, "i32.div_s rhs")?;
            let lhs = super::pop_value(value_stack, "i32.div_s lhs")?;
            emit_abort_on_zero_divisor(script)?;
            emit_abort_on_signed_div_overflow(script, 32)?;
            let result = emit_binary_op(script, "DIV", lhs, rhs, |a, b| {
                let dividend = a as i32;
                let divisor = b as i32;
                if divisor == 0 {
                    return None;
                }
                if dividend == i32::MIN && divisor == -1 {
                    return None;
                }
                Some((dividend / divisor) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32DivU => {
            let rhs = super::pop_value(value_stack, "i32.div_u rhs")?;
            let lhs = super::pop_value(value_stack, "i32.div_u lhs")?;
            let result = emit_unsigned_binary_op(script, UnsignedOp::Div, lhs, rhs, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32RemS => {
            let rhs = super::pop_value(value_stack, "i32.rem_s rhs")?;
            let lhs = super::pop_value(value_stack, "i32.rem_s lhs")?;
            emit_abort_on_zero_divisor(script)?;
            let result = emit_binary_op(script, "MOD", lhs, rhs, |a, b| {
                let dividend = a as i32;
                let divisor = b as i32;
                if divisor == 0 {
                    return None;
                }
                Some((dividend % divisor) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32RemU => {
            let rhs = super::pop_value(value_stack, "i32.rem_u rhs")?;
            let lhs = super::pop_value(value_stack, "i32.rem_u lhs")?;
            let result = emit_unsigned_binary_op(script, UnsignedOp::Rem, lhs, rhs, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64DivS => {
            let rhs = super::pop_value(value_stack, "i64.div_s rhs")?;
            let lhs = super::pop_value(value_stack, "i64.div_s lhs")?;
            emit_abort_on_zero_divisor(script)?;
            emit_abort_on_signed_div_overflow(script, 64)?;
            let result = emit_binary_op(script, "DIV", lhs, rhs, |a, b| {
                let dividend = a as i64;
                let divisor = b as i64;
                if divisor == 0 {
                    return None;
                }
                if dividend == i64::MIN && divisor == -1 {
                    return None;
                }
                Some((dividend / divisor) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64DivU => {
            let rhs = super::pop_value(value_stack, "i64.div_u rhs")?;
            let lhs = super::pop_value(value_stack, "i64.div_u lhs")?;
            let result = emit_unsigned_binary_op(script, UnsignedOp::Div, lhs, rhs, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64RemS => {
            let rhs = super::pop_value(value_stack, "i64.rem_s rhs")?;
            let lhs = super::pop_value(value_stack, "i64.rem_s lhs")?;
            emit_abort_on_zero_divisor(script)?;
            let result = emit_binary_op(script, "MOD", lhs, rhs, |a, b| {
                let dividend = a as i64;
                let divisor = b as i64;
                if divisor == 0 {
                    return None;
                }
                Some((dividend % divisor) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64RemU => {
            let rhs = super::pop_value(value_stack, "i64.rem_u rhs")?;
            let lhs = super::pop_value(value_stack, "i64.rem_u lhs")?;
            let result = emit_unsigned_binary_op(script, UnsignedOp::Rem, lhs, rhs, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        _ => Ok(false),
    }
}
