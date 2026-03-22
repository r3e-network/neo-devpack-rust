// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    imports: &[FunctionImport],
    func_type_indices: &[u32],
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
    is_unreachable: &mut bool,
) -> Result<bool> {
    match op {
        Operator::RefEq => {
            let rhs = super::pop_value(value_stack, "ref.eq rhs")?;
            let lhs = super::pop_value(value_stack, "ref.eq lhs")?;
            let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                Some(if a == b { 1 } else { 0 })
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::RefAsNonNull => {
            let value = super::pop_value(value_stack, "ref.as_non_null operand")?;
            if let Some(constant) = value.const_value {
                if constant == FUNCREF_NULL {
                    let abort = lookup_opcode("ABORT")?;
                    script.push(abort.byte);
                    value_stack.clear();
                } else {
                    value_stack.push(value);
                }
            } else {
                let dup = lookup_opcode("DUP")?;
                script.push(dup.byte);
                let _ = emit_push_int(script, FUNCREF_NULL);
                script.push(lookup_opcode("EQUAL")?.byte);
                let skip_trap = emit_jump_placeholder(script, "JMPIFNOT_L")?;
                script.push(lookup_opcode("DROP")?.byte);
                script.push(lookup_opcode("ABORT")?.byte);
                let continue_label = script.len();
                patch_jump(script, skip_trap, continue_label)?;
                value_stack.push(value);
            }
            Ok(true)
        }
        Operator::RefNull { hty } => match *hty {
            HeapType::FUNC => {
                let entry = emit_push_int(script, FUNCREF_NULL);
                value_stack.push(entry);
                Ok(true)
            }
            other => bail!(
                "ref.null with heap type {:?} is unsupported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                other
            ),
        },
        Operator::RefIsNull => {
            let value = super::pop_value(value_stack, "ref.is_null operand")?;
            let sentinel = emit_push_int(script, FUNCREF_NULL);
            let result = emit_binary_op(script, "EQUAL", value, sentinel, |a, b| {
                Some(if a == b { 1 } else { 0 })
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::RefFunc { function_index } => {
            let total_functions = imports.len() + func_type_indices.len();
            if (*function_index as usize) >= total_functions {
                bail!(
                    "ref.func references unknown function index {} (total functions: {})",
                    function_index,
                    total_functions
                );
            }
            runtime.register_ref_func_constant(*function_index);
            let entry = emit_push_int(script, (*function_index) as i128);
            value_stack.push(entry);
            Ok(true)
        }
        Operator::Unreachable => {
            let abort = lookup_opcode("ABORT")?;
            script.push(abort.byte);
            *is_unreachable = true;
            value_stack.clear();
            Ok(true)
        }
        _ => Ok(false),
    }
}
