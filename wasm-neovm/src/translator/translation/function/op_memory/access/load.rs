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
        Operator::I32Load { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i32.load address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                4,
                None,
                32,
                "i32.load",
            )?;
            Ok(true)
        }
        Operator::I64Load { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i64.load address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                8,
                None,
                64,
                "i64.load",
            )?;
            Ok(true)
        }
        Operator::I32Load8S { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i32.load8_s address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                1,
                Some((8, 32)),
                32,
                "i32.load8_s",
            )?;
            Ok(true)
        }
        Operator::I32Load8U { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i32.load8_u address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                1,
                None,
                32,
                "i32.load8_u",
            )?;
            Ok(true)
        }
        Operator::I32Load16S { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i32.load16_s address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                2,
                Some((16, 32)),
                32,
                "i32.load16_s",
            )?;
            Ok(true)
        }
        Operator::I32Load16U { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i32.load16_u address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                2,
                None,
                32,
                "i32.load16_u",
            )?;
            Ok(true)
        }
        Operator::I64Load8S { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i64.load8_s address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                1,
                Some((8, 64)),
                64,
                "i64.load8_s",
            )?;
            Ok(true)
        }
        Operator::I64Load8U { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i64.load8_u address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                1,
                None,
                64,
                "i64.load8_u",
            )?;
            Ok(true)
        }
        Operator::I64Load16S { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i64.load16_s address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                2,
                Some((16, 64)),
                64,
                "i64.load16_s",
            )?;
            Ok(true)
        }
        Operator::I64Load16U { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i64.load16_u address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                2,
                None,
                64,
                "i64.load16_u",
            )?;
            Ok(true)
        }
        Operator::I64Load32S { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i64.load32_s address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                4,
                Some((32, 64)),
                64,
                "i64.load32_s",
            )?;
            Ok(true)
        }
        Operator::I64Load32U { memarg, .. } => {
            let mem = memarg.memory;
            let offset = memarg.offset;
            let addr = super::super::pop_value(value_stack, "i64.load32_u address")?;
            translate_memory_load(
                script,
                runtime,
                value_stack,
                addr,
                mem,
                offset,
                4,
                None,
                64,
                "i64.load32_u",
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
