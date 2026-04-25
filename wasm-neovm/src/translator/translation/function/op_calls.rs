// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

/// Helper to get function type information by index
fn get_function_type_info<'a>(
    function_index: u32,
    imports: &'a [FunctionImport],
    types: &'a [FuncType],
    func_type_indices: &'a [u32],
) -> Result<(Option<&'a FunctionImport>, usize, &'a FuncType)> {
    let is_import = (function_index as usize) < imports.len();

    let (type_index, import_ref) = if is_import {
        let import = imports
            .get(function_index as usize)
            .ok_or_else(|| anyhow!("function index {} out of bounds", function_index))?;
        (get_import_type_index(import)?, Some(import))
    } else {
        let defined_index = (function_index as usize)
            .checked_sub(imports.len())
            .ok_or_else(|| anyhow!("function index underflow"))?;
        let type_index = func_type_indices
            .get(defined_index)
            .copied()
            .ok_or_else(|| anyhow!("no type index recorded for function {}", function_index))?;
        (type_index, None)
    };

    let func_type = types
        .get(type_index as usize)
        .ok_or_else(|| anyhow!("invalid type index {}", type_index))?;

    Ok((import_ref, type_index as usize, func_type))
}

/// Push a placeholder value onto the stack
fn push_placeholder_value(value_stack: &mut Vec<StackValue>) {
    value_stack.push(StackValue {
        const_value: None,
        bytecode_start: None,
        pending_sign_extend: None,
    });
}

/// Reverse the top `n` items on the NeoVM evaluation stack.
///
/// Used before `CALL_L` to a wasm-defined function so that NeoVM `INITSLOT`
/// (which pops arguments top-first into `Arguments[0..N]`) ends up populating
/// slots in the same order WebAssembly's `local.get N` expects.
///
/// WebAssembly `call f(a, b, c)` pushes `a` then `b` then `c` (so `c` is on
/// top). Without reversing, `INITSLOT` would store `c` in `Arguments[0]` and
/// `a` in `Arguments[N-1]` — the opposite of `local.get 0 == a`. Reversing
/// flips the top to `[c, b, a]` so `INITSLOT` lands `a` in slot 0.
///
/// Imports are exempt: their helper bodies (e.g. `emit_storage_*_helper`)
/// already account for the top-first slot order in their own `INITSLOT`
/// dispatch, so adding a reversal here would re-flip them and break.
fn emit_reverse_args(script: &mut Vec<u8>, n: usize) -> Result<()> {
    match n {
        0 | 1 => Ok(()),
        2 => {
            script.push(lookup_opcode("SWAP")?.byte);
            Ok(())
        }
        3 => {
            script.push(lookup_opcode("REVERSE3")?.byte);
            Ok(())
        }
        4 => {
            script.push(lookup_opcode("REVERSE4")?.byte);
            Ok(())
        }
        _ => {
            let _ = emit_push_int(script, n as i128);
            script.push(lookup_opcode("REVERSEN")?.byte);
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    imports: &[FunctionImport],
    types: &[FuncType],
    func_type_indices: &[u32],
    runtime: &mut RuntimeHelpers,
    tables: &[TableInfo],
    functions: &mut FunctionRegistry,
    value_stack: &mut Vec<StackValue>,
    features: &mut FeatureTracker,
    adapter: &dyn ChainAdapter,
    is_unreachable: &mut bool,
) -> Result<bool> {
    match op {
        Operator::Return => {
            script.push(RET);
            *is_unreachable = true;
            value_stack.clear();
            Ok(true)
        }
        Operator::Call { function_index } => {
            let function_index = *function_index;
            let (_, _, func_sig) =
                get_function_type_info(function_index, imports, types, func_type_indices)?;
            let param_count = func_sig.params().len();

            let mut params = Vec::with_capacity(param_count);
            for _ in 0..param_count {
                params.push(super::pop_value(value_stack, "call argument")?);
            }
            params.reverse();

            if let Some(import) = imports.get(function_index as usize) {
                let type_index = get_import_type_index(import)?;
                let func_sig = types.get(type_index as usize).ok_or_else(|| {
                    anyhow!(
                        "invalid type index {} for import {}",
                        type_index,
                        import.name
                    )
                })?;
                if try_handle_env_import(import, func_sig, &params, runtime, script, value_stack)? {
                    return Ok(true);
                }
                if let Some(descriptor) =
                    try_handle_neo_import(import, func_sig, &params, runtime, script)?
                {
                    features.register_syscall(descriptor);
                    if !func_sig.results().is_empty() {
                        push_placeholder_value(value_stack);
                    }
                    return Ok(true);
                }

                handle_import_call(
                    function_index,
                    script,
                    imports,
                    types,
                    &params,
                    features,
                    adapter,
                )?;
                if !func_sig.results().is_empty() {
                    push_placeholder_value(value_stack);
                }
            } else {
                let defined_index = (function_index as usize) - imports.len();
                let type_index =
                    func_type_indices
                        .get(defined_index)
                        .copied()
                        .ok_or_else(|| {
                            anyhow!("no type index recorded for function {}", function_index)
                        })?;
                let func_sig = types.get(type_index as usize).ok_or_else(|| {
                    anyhow!(
                        "invalid type index {} for function {}",
                        type_index,
                        function_index
                    )
                })?;
                if func_sig.params().len() != params.len() {
                    bail!(
                        "function {} expects {} argument(s) but {} were provided",
                        function_index,
                        func_sig.params().len(),
                        params.len()
                    );
                }
                if func_sig.results().len() > 1 {
                    bail!(
                        "multi-value returns are not supported (function {} returns {} values)",
                        function_index,
                        func_sig.results().len()
                    );
                }
                emit_reverse_args(script, func_sig.params().len())?;
                functions.emit_call(script, function_index as usize)?;
                if !func_sig.results().is_empty() {
                    push_placeholder_value(value_stack);
                }
            }
            Ok(true)
        }
        Operator::CallIndirect {
            table_index,
            type_index,
        } => {
            let table_index = *table_index;
            let type_index = *type_index;

            tables
                .get(table_index as usize)
                .ok_or_else(|| anyhow!("call_indirect references missing table {}", table_index))?;

            let func_sig = types.get(type_index as usize).ok_or_else(|| {
                anyhow!("type index {} out of bounds for call_indirect", type_index)
            })?;

            for ty in func_sig.params() {
                match ty {
                    ValType::I32 | ValType::I64 => {}
                    other => bail!(
                        "call_indirect with unsupported parameter type {:?}; only i32/i64 are supported",
                        other
                    ),
                }
            }
            if func_sig.results().len() > 1 {
                bail!("call_indirect returning multiple values is not supported");
            }

            let _table_index_value = super::pop_value(value_stack, "call_indirect table index")?;

            for _ in 0..func_sig.params().len() {
                let _ = super::pop_value(value_stack, "call_indirect argument")?;
            }

            runtime.emit_memory_init_call(script)?;
            runtime.emit_call_indirect_helper(script, table_index as usize, type_index)?;

            if !func_sig.results().is_empty() {
                push_placeholder_value(value_stack);
            }

            Ok(true)
        }
        _ => Ok(false),
    }
}
