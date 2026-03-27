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
        Operator::I32WrapI64 => {
            let value = super::pop_value(value_stack, "i32.wrap_i64 operand")?;
            let result = emit_sign_extend_via_helper(script, runtime, value, 32, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64ExtendI32U => {
            let value = super::pop_value(value_stack, "i64.extend_i32_u operand")?;
            let result = emit_zero_extend(script, value, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64ExtendI32S => {
            let value = super::pop_value(value_stack, "i64.extend_i32_s operand")?;
            let result = emit_sign_extend(script, value, 32, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Extend8S => {
            let value = super::pop_value(value_stack, "i32.extend8_s operand")?;
            let result = emit_sign_extend(script, value, 8, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Extend16S => {
            let value = super::pop_value(value_stack, "i32.extend16_s operand")?;
            let result = emit_sign_extend(script, value, 16, 32)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Extend8S => {
            let value = super::pop_value(value_stack, "i64.extend8_s operand")?;
            let result = emit_sign_extend(script, value, 8, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Extend16S => {
            let value = super::pop_value(value_stack, "i64.extend16_s operand")?;
            let result = emit_sign_extend(script, value, 16, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Extend32S => {
            let value = super::pop_value(value_stack, "i64.extend32_s operand")?;
            let result = emit_sign_extend(script, value, 32, 64)?;
            value_stack.push(result);
            Ok(true)
        }
        _ => Ok(false),
    }
}
