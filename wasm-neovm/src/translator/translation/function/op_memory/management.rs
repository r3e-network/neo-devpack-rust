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
        Operator::MemorySize { mem, .. } => {
            ensure_memory_access(runtime, *mem)?;
            runtime.emit_memory_init_call(script)?;
            script.push(lookup_opcode("LDSFLD2")?.byte);
            value_stack.push(StackValue {
                const_value: None,
                bytecode_start: None,
            });
            Ok(true)
        }
        Operator::MemoryGrow { mem, .. } => {
            let _delta = super::pop_value(value_stack, "memory.grow delta")?;
            ensure_memory_access(runtime, *mem)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_memory_grow_call(script)?;
            value_stack.push(StackValue {
                const_value: None,
                bytecode_start: None,
            });
            Ok(true)
        }
        _ => Ok(false),
    }
}
