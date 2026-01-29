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
    });
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

            let mut params = Vec::with_capacity(func_sig.params().len());
            for _ in 0..func_sig.params().len() {
                params.push(super::pop_value(value_stack, "call_indirect argument")?);
            }
            params.reverse();

            runtime.emit_memory_init_call(script)?;
            runtime.table_slot(table_index as usize)?;
            runtime.emit_table_helper(script, TableHelperKind::Get(table_index as usize))?;

            script.push(lookup_opcode("DUP")?.byte);
            let _ = emit_push_int(script, FUNCREF_NULL);
            script.push(lookup_opcode("EQUAL")?.byte);
            let trap_null = emit_jump_placeholder(script, "JMPIF_L")?;

            let total_functions = imports.len() + func_type_indices.len();
            // Pre-allocate case_fixups based on actual matches found (Round 62 optimization)
            let estimated_matches = total_functions.min(32);
            let mut case_fixups: Vec<(usize, CallTarget)> = Vec::with_capacity(estimated_matches);
            for fn_index in 0..total_functions {
                let candidate_type_index = if fn_index < imports.len() {
                    get_import_type_index(&imports[fn_index])?
                } else {
                    let defined_index = fn_index - imports.len();
                    *func_type_indices.get(defined_index).ok_or_else(|| {
                        anyhow!(
                            "call_indirect target function {} missing type entry",
                            fn_index
                        )
                    })?
                };

                if candidate_type_index != type_index {
                    continue;
                }

                script.push(lookup_opcode("DUP")?.byte);
                let _ = emit_push_int(script, fn_index as i128);
                script.push(lookup_opcode("EQUAL")?.byte);
                let jump = emit_jump_placeholder(script, "JMPIF_L")?;

                let target = if fn_index < imports.len() {
                    CallTarget::Import(fn_index as u32)
                } else {
                    CallTarget::Defined(fn_index)
                };
                case_fixups.push((jump, target));
            }

            let trap_label = script.len();
            script.push(lookup_opcode("DROP")?.byte);
            script.push(lookup_opcode("ABORT")?.byte);
            patch_jump(script, trap_null, trap_label)?;

            // Pre-allocate end_fixups with same capacity as case_fixups (Round 62 optimization)
            let mut end_fixups: Vec<usize> = Vec::with_capacity(estimated_matches);
            for (jump, target) in case_fixups {
                let label = script.len();
                patch_jump(script, jump, label)?;
                script.push(lookup_opcode("DROP")?.byte);
                match target {
                    CallTarget::Import(idx) => {
                        handle_import_call(
                            idx, script, imports, types, &params, features, adapter,
                        )?;
                    }
                    CallTarget::Defined(idx) => {
                        functions.emit_call(script, idx)?;
                    }
                }
                let end_jump = emit_jump_placeholder(script, "JMP_L")?;
                end_fixups.push(end_jump);
            }

            let end_label = script.len();
            for fixup in end_fixups {
                patch_jump(script, fixup, end_label)?;
            }

            if !func_sig.results().is_empty() {
                push_placeholder_value(value_stack);
            }

            Ok(true)
        }
        _ => Ok(false),
    }
}
