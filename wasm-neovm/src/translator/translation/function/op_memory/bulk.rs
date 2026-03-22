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
        Operator::MemoryFill { mem, .. } => {
            let len = super::pop_value(value_stack, "memory.fill len")?;
            let value = super::pop_value(value_stack, "memory.fill value")?;
            let dest = super::pop_value(value_stack, "memory.fill dest")?;
            translate_memory_fill(script, runtime, dest, value, len, *mem)
                .context("failed to translate memory.fill")?;
            Ok(true)
        }
        Operator::MemoryCopy {
            dst_mem, src_mem, ..
        } => {
            let len = super::pop_value(value_stack, "memory.copy len")?;
            let src = super::pop_value(value_stack, "memory.copy src")?;
            let dest = super::pop_value(value_stack, "memory.copy dest")?;
            translate_memory_copy(script, runtime, dest, src, len, *dst_mem, *src_mem)
                .context("failed to translate memory.copy")?;
            Ok(true)
        }
        Operator::MemoryInit {
            data_index, mem, ..
        } => {
            let len = super::pop_value(value_stack, "memory.init len")?;
            let src = super::pop_value(value_stack, "memory.init offset")?;
            let dest = super::pop_value(value_stack, "memory.init dest")?;
            translate_memory_init(script, runtime, dest, src, len, *data_index, *mem)
                .context("failed to translate memory.init")?;
            Ok(true)
        }
        Operator::DataDrop { data_index } => {
            translate_data_drop(script, runtime, *data_index)
                .context("failed to translate data.drop")?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
