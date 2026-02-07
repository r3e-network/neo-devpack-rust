use super::*;

mod op_calls;
mod op_control;
mod op_locals;
mod op_memory;
mod op_numeric;
mod op_refs;
mod op_tables;

/// Context for function translation to reduce parameter count
///
/// This struct groups together all the context needed for translating
/// a WebAssembly function to NeoVM bytecode.
pub struct TranslationContext<'a> {
    pub func_type: &'a FuncType,
    pub body: &'a wasmparser::FunctionBody<'a>,
    pub script: &'a mut Vec<u8>,
    pub imports: &'a [FunctionImport],
    pub types: &'a [FuncType],
    pub func_type_indices: &'a [u32],
    pub runtime: &'a mut RuntimeHelpers,
    pub tables: &'a [TableInfo],
    pub functions: &'a mut FunctionRegistry,
    pub function_index: usize,
    pub start_function: Option<u32>,
    pub function_name: &'a str,
    pub features: &'a mut FeatureTracker,
    pub adapter: &'a dyn ChainAdapter,
}

pub(super) fn translate_function(ctx: &mut TranslationContext<'_>) -> Result<String> {
    let params = ctx.func_type.params();
    for ty in params {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!("only i32/i64 parameters are supported (found {:?})", other),
        }
    }
    let param_count = params.len();

    let returns = ctx.func_type.results();
    if returns.len() > 1 {
        bail!("multi-value returns are not supported");
    }

    if let Some(start_idx) = ctx.start_function {
        if start_idx as usize != ctx.function_index {
            ctx.runtime.emit_memory_init_call(ctx.script)?;
        }
    }

    let return_kind = returns.first().map(wasm_val_type_to_manifest).transpose()?;

    let locals_reader = ctx.body.get_locals_reader()?;
    let mut local_states: Vec<LocalState> = Vec::new();
    for i in 0..param_count {
        local_states.push(LocalState {
            kind: LocalKind::Param(i as u32),
            const_value: None,
        });
    }

    let mut local_count: u32 = 0;
    for entry in ctx.body.get_locals_reader()? {
        let (count, ty) = entry?;
        if ty != ValType::I32 && ty != ValType::I64 {
            bail!("only i32/i64 locals are supported (found {:?})", ty);
        }
        local_count = local_count
            .checked_add(count)
            .ok_or_else(|| anyhow!("function {} local count overflow", ctx.function_name))?;
    }

    if param_count > u8::MAX as usize {
        bail!(
            "function {} has too many parameters ({}) for NeoVM INITSLOT",
            ctx.function_name,
            param_count
        );
    }
    if local_count > u8::MAX as u32 {
        bail!(
            "function {} has too many locals ({}) for NeoVM INITSLOT",
            ctx.function_name,
            local_count
        );
    }

    if local_count > 0 || param_count > 0 {
        ctx.script.push(lookup_opcode("INITSLOT")?.byte);
        ctx.script.push(local_count as u8);
        ctx.script.push(param_count as u8);
    }

    // NeoVM parameters are arbitrary-precision integers. Normalise them to the Wasm bit-width
    // so arithmetic, comparisons, and shifts observe WebAssembly's i32/i64 semantics.
    // `_deploy` is invoked with (Any, Boolean) by Neo and must not force integer coercions.
    if !ctx.function_name.eq_ignore_ascii_case("_deploy") {
        for (index, ty) in params.iter().enumerate() {
            emit_load_arg(ctx.script, index as u32)?;
            let value = StackValue {
                const_value: None,
                bytecode_start: None,
            };
            match ty {
                ValType::I32 => {
                    let _ = emit_sign_extend(ctx.script, value, 32, 32)?;
                }
                ValType::I64 => {
                    let _ = emit_sign_extend(ctx.script, value, 64, 64)?;
                }
                _ => unreachable!("parameter types validated earlier"),
            }
            emit_store_arg(ctx.script, index as u32)?;
        }
    }

    let mut next_local_slot: u32 = 0;
    for entry in locals_reader {
        let (count, ty) = entry?;
        if ty != ValType::I32 && ty != ValType::I64 {
            bail!("only i32/i64 locals are supported (found {:?})", ty);
        }
        for _ in 0..count {
            local_states.push(LocalState {
                kind: LocalKind::Local(next_local_slot),
                const_value: Some(0),
            });
            next_local_slot += 1;
        }
    }

    let op_reader = ctx.body.get_operators_reader()?;
    let mut value_stack: Vec<StackValue> = Vec::new();
    let mut control_stack: Vec<ControlFrame> = Vec::new();
    let mut is_unreachable = false;

    // Push implicit function-level control frame
    // In WASM, the function body itself is an implicit block that can be targeted by branches
    // stack_height is 0 because branches to the function can occur at any point
    // result_count tracks how many values must be on stack when branching to function exit
    control_stack.push(ControlFrame {
        kind: ControlKind::Function,
        stack_height: 0,
        result_count: returns.len(), // Function expects return values
        start_offset: ctx.script.len(),
        end_fixups: Vec::new(),
        if_false_fixup: None,
        has_else: false,
        entry_reachable: true,
        end_reachable_from_branch: false,
        if_then_end_reachable: None,
    });

    // Ensure the current function offset is known to callers (already registered before entry).
    // This assertion helps catch internal misuse during development.
    if !ctx.functions.contains_index(ctx.function_index) {
        bail!(
            "function index {} out of range for translation",
            ctx.function_index
        );
    }

    for op in op_reader {
        let op = op?;

        // In WASM, code after an unconditional branch/return/unreachable is unreachable with a
        // polymorphic stack. We still must translate structured control operators to keep the
        // control stack balanced and patch jump fixups, but can skip translating other operators.
        if is_unreachable {
            if op_control::try_handle(
                &op,
                ctx.script,
                ctx.types,
                &mut value_stack,
                &mut control_stack,
                &mut is_unreachable,
            )? {
                continue;
            }
            continue;
        }

        if op_numeric::try_handle(&op, ctx.script, ctx.runtime, &mut value_stack)? {
            continue;
        }

        if op_control::try_handle(
            &op,
            ctx.script,
            ctx.types,
            &mut value_stack,
            &mut control_stack,
            &mut is_unreachable,
        )? {
            continue;
        }

        if op_memory::try_handle(&op, ctx.script, ctx.runtime, &mut value_stack)? {
            continue;
        }

        if op_tables::try_handle(&op, ctx.script, ctx.runtime, &mut value_stack)? {
            continue;
        }

        if op_locals::try_handle(
            &op,
            ctx.script,
            ctx.runtime,
            &mut value_stack,
            &mut local_states,
        )? {
            continue;
        }

        if op_calls::try_handle(
            &op,
            ctx.script,
            ctx.imports,
            ctx.types,
            ctx.func_type_indices,
            ctx.runtime,
            ctx.tables,
            ctx.functions,
            &mut value_stack,
            ctx.features,
            ctx.adapter,
            &mut is_unreachable,
        )? {
            continue;
        }

        if op_refs::try_handle(
            &op,
            ctx.script,
            ctx.imports,
            ctx.func_type_indices,
            ctx.runtime,
            &mut value_stack,
            &mut is_unreachable,
        )? {
            continue;
        }

        if let Some(desc) = describe_float_op(&op) {
            let context = format!("{} in function {}", desc, ctx.function_name);
            return numeric::unsupported_float(&context);
        }
        if let Some(desc) = describe_simd_op(&op) {
            let context = format!("{} in function {}", desc, ctx.function_name);
            return numeric::unsupported_simd(&context);
        }
        bail!(format!(
            "unsupported Wasm operator {:?} ({}).",
            op, UNSUPPORTED_FEATURE_DOC
        ));
    }

    // Always end with an epilogue RET so `br` to the function-level implicit block has a
    // well-defined jump target.
    ctx.script.push(RET);

    if let Some(frame) = control_stack.last() {
        bail!(
            "unclosed block detected at end of function (kind: {:?})",
            frame.kind
        );
    }

    Ok(return_kind.unwrap_or_else(|| "Void".to_string()))
}

fn pop_value(stack: &mut Vec<StackValue>, context: &str) -> Result<StackValue> {
    stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow while processing {}", context))
}

fn pop_value_maybe_unreachable(
    stack: &mut Vec<StackValue>,
    context: &str,
    is_unreachable: bool,
) -> Result<StackValue> {
    if let Some(value) = stack.pop() {
        return Ok(value);
    }
    if is_unreachable {
        return Ok(StackValue {
            const_value: None,
            bytecode_start: None,
        });
    }
    Err(anyhow!("stack underflow while processing {}", context))
}

fn set_stack_height_polymorphic(stack: &mut Vec<StackValue>, height: usize) {
    while stack.len() < height {
        stack.push(StackValue {
            const_value: None,
            bytecode_start: None,
        });
    }
    stack.truncate(height);
}
