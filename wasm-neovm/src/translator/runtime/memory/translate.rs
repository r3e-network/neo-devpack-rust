use super::super::*;

use super::ensure_memory_access;

#[allow(clippy::too_many_arguments)]
pub(crate) fn translate_memory_load(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
    base: StackValue,
    mem_index: u32,
    offset: u64,
    bytes: u32,
    sign_extend: Option<(u32, u32)>,
    result_bits: u32,
    context: &str,
) -> Result<()> {
    ensure_memory_access(runtime, mem_index)?;
    runtime.emit_memory_init_call(script)?;
    let _addr = apply_memory_offset(script, base, offset)
        .with_context(|| format!("failed to apply offset for {}", context))?;
    runtime
        .emit_memory_load_call(script, bytes)
        .with_context(|| format!("failed to emit helper call for {}", context))?;

    let mut raw_value = StackValue {
        const_value: None,
        bytecode_start: None,
    };

    let load_bits = bytes * 8;
    let result = if let Some((from_bits, to_bits)) = sign_extend {
        emit_sign_extend(script, raw_value, from_bits, to_bits)?
    } else {
        if result_bits < load_bits {
            bail!(
                "result bit-width {} smaller than load width {}",
                result_bits,
                load_bits
            );
        }
        if result_bits > load_bits {
            raw_value = emit_zero_extend(script, raw_value, load_bits)?;
        }
        raw_value
    };

    value_stack.push(result);
    Ok(())
}

fn apply_memory_offset(script: &mut Vec<u8>, base: StackValue, offset: u64) -> Result<StackValue> {
    let base = StackValue {
        const_value: base.const_value,
        bytecode_start: None,
    };

    if offset == 0 {
        return emit_zero_extend(script, base, 32);
    }

    let offset_value = emit_push_int(script, offset as i128);
    let added = emit_binary_op(script, "ADD", base, offset_value, |a, b| Some(a + b))?;
    let added = StackValue {
        const_value: added.const_value,
        bytecode_start: None,
    };
    emit_zero_extend(script, added, 32)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn translate_memory_store(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value: StackValue,
    address: StackValue,
    mem_index: u32,
    offset: u64,
    bytes: u32,
    context: &str,
) -> Result<()> {
    let _ = value;
    ensure_memory_access(runtime, mem_index)?;
    runtime.emit_memory_init_call(script)?;
    script.push(lookup_opcode("SWAP")?.byte);
    let _addr = apply_memory_offset(script, address, offset)
        .with_context(|| format!("failed to apply offset for {}", context))?;
    script.push(lookup_opcode("SWAP")?.byte);
    runtime
        .emit_memory_store_call(script, bytes)
        .with_context(|| format!("failed to emit helper call for {}", context))?;
    Ok(())
}

pub(crate) fn translate_memory_fill(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    _dest: StackValue,
    _value: StackValue,
    _len: StackValue,
    mem_index: u32,
) -> Result<()> {
    ensure_memory_access(runtime, mem_index)?;
    runtime.emit_memory_init_call(script)?;
    runtime
        .emit_memory_fill_call(script)
        .context("failed to emit helper call for memory.fill")?;
    Ok(())
}

pub(crate) fn translate_memory_copy(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    _dest: StackValue,
    _src: StackValue,
    _len: StackValue,
    dst_mem: u32,
    src_mem: u32,
) -> Result<()> {
    if dst_mem != 0 || src_mem != 0 {
        bail!("only default memory index 0 is supported for memory.copy");
    }
    ensure_memory_access(runtime, dst_mem)?;
    runtime.emit_memory_init_call(script)?;
    runtime
        .emit_memory_copy_call(script)
        .context("failed to emit helper call for memory.copy")?;
    Ok(())
}

pub(crate) fn translate_memory_init(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    _dest: StackValue,
    _src: StackValue,
    _len: StackValue,
    data_index: u32,
    mem_index: u32,
) -> Result<()> {
    ensure_memory_access(runtime, mem_index)?;
    runtime.emit_memory_init_call(script)?;
    runtime
        .emit_data_init_call(script, data_index)
        .context("failed to emit helper call for memory.init")?;
    Ok(())
}

pub(crate) fn translate_data_drop(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    segment_index: u32,
) -> Result<()> {
    runtime.emit_memory_init_call(script)?;
    runtime
        .emit_data_drop_call(script, segment_index)
        .context("failed to emit helper call for data.drop")?;
    Ok(())
}
