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
        Operator::I32Store { memarg, .. } => {
            let value = super::super::pop_value(value_stack, "i32.store value")?;
            let addr = super::super::pop_value(value_stack, "i32.store address")?;
            translate_memory_store(
                script,
                runtime,
                value,
                addr,
                memarg.memory,
                memarg.offset,
                4,
                "i32.store",
            )?;
            Ok(true)
        }
        Operator::I64Store { memarg, .. } => {
            let value = super::super::pop_value(value_stack, "i64.store value")?;
            let addr = super::super::pop_value(value_stack, "i64.store address")?;
            translate_memory_store(
                script,
                runtime,
                value,
                addr,
                memarg.memory,
                memarg.offset,
                8,
                "i64.store",
            )?;
            Ok(true)
        }
        Operator::I32Store8 { memarg, .. } => {
            let value = super::super::pop_value(value_stack, "i32.store8 value")?;
            let addr = super::super::pop_value(value_stack, "i32.store8 address")?;
            translate_memory_store(
                script,
                runtime,
                value,
                addr,
                memarg.memory,
                memarg.offset,
                1,
                "i32.store8",
            )?;
            Ok(true)
        }
        Operator::I32Store16 { memarg, .. } => {
            let value = super::super::pop_value(value_stack, "i32.store16 value")?;
            let addr = super::super::pop_value(value_stack, "i32.store16 address")?;
            translate_memory_store(
                script,
                runtime,
                value,
                addr,
                memarg.memory,
                memarg.offset,
                2,
                "i32.store16",
            )?;
            Ok(true)
        }
        Operator::I64Store8 { memarg, .. } => {
            let value = super::super::pop_value(value_stack, "i64.store8 value")?;
            let addr = super::super::pop_value(value_stack, "i64.store8 address")?;
            translate_memory_store(
                script,
                runtime,
                value,
                addr,
                memarg.memory,
                memarg.offset,
                1,
                "i64.store8",
            )?;
            Ok(true)
        }
        Operator::I64Store16 { memarg, .. } => {
            let value = super::super::pop_value(value_stack, "i64.store16 value")?;
            let addr = super::super::pop_value(value_stack, "i64.store16 address")?;
            translate_memory_store(
                script,
                runtime,
                value,
                addr,
                memarg.memory,
                memarg.offset,
                2,
                "i64.store16",
            )?;
            Ok(true)
        }
        Operator::I64Store32 { memarg, .. } => {
            let value = super::super::pop_value(value_stack, "i64.store32 value")?;
            let addr = super::super::pop_value(value_stack, "i64.store32 address")?;
            translate_memory_store(
                script,
                runtime,
                value,
                addr,
                memarg.memory,
                memarg.offset,
                4,
                "i64.store32",
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
