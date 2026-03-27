// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    match op {
        Operator::I32Add => {
            let rhs = super::pop_value(value_stack, "i32.add rhs")?;
            let lhs = super::pop_value(value_stack, "i32.add lhs")?;
            let result = emit_binary_op(script, "ADD", lhs, rhs, |a, b| {
                let lhs = a as i32;
                let rhs = b as i32;
                Some(lhs.wrapping_add(rhs) as i128)
            })?;
            let result = emit_sign_extend_via_helper(script, runtime, result, 32, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Add => {
            let rhs = super::pop_value(value_stack, "i64.add rhs")?;
            let lhs = super::pop_value(value_stack, "i64.add lhs")?;
            let result = emit_binary_op(script, "ADD", lhs, rhs, |a, b| {
                let lhs = a as i64;
                let rhs = b as i64;
                Some(lhs.wrapping_add(rhs) as i128)
            })?;
            let result = emit_sign_extend_via_helper(script, runtime, result, 64, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Sub => {
            let rhs = super::pop_value(value_stack, "i32.sub rhs")?;
            let lhs = super::pop_value(value_stack, "i32.sub lhs")?;
            let result = emit_binary_op(script, "SUB", lhs, rhs, |a, b| {
                let lhs = a as i32;
                let rhs = b as i32;
                Some(lhs.wrapping_sub(rhs) as i128)
            })?;
            let result = emit_sign_extend_via_helper(script, runtime, result, 32, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Sub => {
            let rhs = super::pop_value(value_stack, "i64.sub rhs")?;
            let lhs = super::pop_value(value_stack, "i64.sub lhs")?;
            let result = emit_binary_op(script, "SUB", lhs, rhs, |a, b| {
                let lhs = a as i64;
                let rhs = b as i64;
                Some(lhs.wrapping_sub(rhs) as i128)
            })?;
            let result = emit_sign_extend_via_helper(script, runtime, result, 64, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Mul => {
            let rhs = super::pop_value(value_stack, "i32.mul rhs")?;
            let lhs = super::pop_value(value_stack, "i32.mul lhs")?;
            let result = emit_binary_op(script, "MUL", lhs, rhs, |a, b| {
                let lhs = a as i32;
                let rhs = b as i32;
                Some(lhs.wrapping_mul(rhs) as i128)
            })?;
            let result = emit_sign_extend_via_helper(script, runtime, result, 32, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Mul => {
            let rhs = super::pop_value(value_stack, "i64.mul rhs")?;
            let lhs = super::pop_value(value_stack, "i64.mul lhs")?;
            let result = emit_binary_op(script, "MUL", lhs, rhs, |a, b| {
                let lhs = a as i64;
                let rhs = b as i64;
                Some(lhs.wrapping_mul(rhs) as i128)
            })?;
            let result = emit_sign_extend_via_helper(script, runtime, result, 64, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        _ => Ok(false),
    }
}
