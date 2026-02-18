use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
    local_states: &mut [LocalState],
) -> Result<bool> {
    match op {
        Operator::LocalGet { local_index } => {
            let state = local_states
                .get(*local_index as usize)
                .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
            let value = emit_local_get(script, state)?;
            value_stack.push(value);
            Ok(true)
        }
        Operator::LocalSet { local_index } => {
            let value = super::pop_value(value_stack, "local.set operand")?;
            let state = local_states
                .get_mut(*local_index as usize)
                .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
            emit_local_set(script, state, &value)?;
            Ok(true)
        }
        Operator::LocalTee { local_index } => {
            let value = super::pop_value(value_stack, "local.tee operand")?;
            let state = local_states
                .get_mut(*local_index as usize)
                .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
            emit_local_set(script, state, &value)?;
            let value = emit_local_get(script, state)?;
            value_stack.push(value);
            Ok(true)
        }
        Operator::GlobalGet { global_index } => {
            let idx = *global_index as usize;
            let const_value = runtime.global_const_value(idx)?;
            if let Some(value) = const_value {
                let entry = emit_push_int(script, value);
                value_stack.push(entry);
            } else {
                runtime.emit_memory_init_call(script)?;
                let slot = runtime.global_slot(idx)?;
                emit_load_static(script, slot)?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Ok(true)
        }
        Operator::GlobalSet { global_index } => {
            let idx = *global_index as usize;
            if !runtime.global_mutable(idx)? {
                bail!("global {} is immutable", idx);
            }
            let _value = super::pop_value(value_stack, "global.set operand")?;
            runtime.emit_memory_init_call(script)?;
            let slot = runtime.global_slot(idx)?;
            emit_store_static(script, slot)?;
            runtime.clear_global_const(idx)?;
            Ok(true)
        }
        Operator::Drop => {
            let _value = super::pop_value(value_stack, "drop operand")?;
            // Do not backtrack with `truncate` here: it can invalidate pending jump/call
            // fixup positions captured earlier in the same function translation.
            let drop = lookup_opcode("DROP")?;
            script.push(drop.byte);
            Ok(true)
        }
        _ => Ok(false),
    }
}
