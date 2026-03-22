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
        Operator::TableGet { table } => {
            let _ = super::pop_value(value_stack, "table.get index")?;
            runtime.table_slot(*table as usize)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_table_helper(script, TableHelperKind::Get(*table as usize))?;
            value_stack.push(StackValue {
                const_value: None,
                bytecode_start: None,
            });
            Ok(true)
        }
        Operator::TableSet { table } => {
            let _ = super::pop_value(value_stack, "table.set value")?;
            let _ = super::pop_value(value_stack, "table.set index")?;
            runtime.table_slot(*table as usize)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_table_helper(script, TableHelperKind::Set(*table as usize))?;
            Ok(true)
        }
        Operator::TableSize { table } => {
            runtime.table_slot(*table as usize)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_table_helper(script, TableHelperKind::Size(*table as usize))?;
            value_stack.push(StackValue {
                const_value: None,
                bytecode_start: None,
            });
            Ok(true)
        }
        Operator::TableGrow { table } => {
            let _delta = super::pop_value(value_stack, "table.grow delta")?;
            let _value = super::pop_value(value_stack, "table.grow value")?;
            runtime.table_slot(*table as usize)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_table_helper(script, TableHelperKind::Grow(*table as usize))?;
            value_stack.push(StackValue {
                const_value: None,
                bytecode_start: None,
            });
            Ok(true)
        }
        Operator::TableFill { table } => {
            let _len = super::pop_value(value_stack, "table.fill len")?;
            let _value = super::pop_value(value_stack, "table.fill value")?;
            let _dst = super::pop_value(value_stack, "table.fill dest")?;
            runtime.table_slot(*table as usize)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_table_helper(script, TableHelperKind::Fill(*table as usize))?;
            Ok(true)
        }
        Operator::TableCopy {
            dst_table,
            src_table,
        } => {
            let _len = super::pop_value(value_stack, "table.copy len")?;
            let _src = super::pop_value(value_stack, "table.copy src")?;
            let _dst = super::pop_value(value_stack, "table.copy dest")?;
            runtime.table_slot(*dst_table as usize)?;
            runtime.table_slot(*src_table as usize)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_table_helper(
                script,
                TableHelperKind::Copy {
                    dst: *dst_table as usize,
                    src: *src_table as usize,
                },
            )?;
            Ok(true)
        }
        Operator::TableInit { table, elem_index } => {
            let _len = super::pop_value(value_stack, "table.init len")?;
            let _src = super::pop_value(value_stack, "table.init offset")?;
            let _dst = super::pop_value(value_stack, "table.init dest")?;
            runtime.ensure_passive_element(*elem_index as usize)?;
            runtime.table_slot(*table as usize)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_table_helper(
                script,
                TableHelperKind::InitFromPassive {
                    table: *table as usize,
                    segment: *elem_index as usize,
                },
            )?;
            Ok(true)
        }
        Operator::ElemDrop { elem_index } => {
            runtime.ensure_passive_element(*elem_index as usize)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_table_helper(script, TableHelperKind::ElemDrop(*elem_index as usize))?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
