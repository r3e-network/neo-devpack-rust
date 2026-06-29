// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

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
        pending_sign_extend: None,
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
        pending_sign_extend: None,
    };

    // Normalise the base to its unsigned 32-bit value FIRST. WASM memory
    // addresses are unsigned i32, but the operand can arrive sign-extended
    // (a "negative" BigInteger standing for a high address). Masking before
    // adding the offset is essential: masking the *sum* instead would let a
    // high base cancel against the offset (e.g. base 0xFFFFFFFC + offset 8
    // would fold to 4) and silently access the wrong location.
    let base = emit_zero_extend(script, base, 32)?;

    if offset == 0 {
        return Ok(base);
    }

    // Validate offset doesn't exceed i64::MAX to prevent overflow when casting to i128
    // and to ensure the result fits within reasonable memory addressing bounds
    if offset > i64::MAX as u64 {
        bail!(
            "memory offset {} exceeds maximum supported value (i64::MAX)",
            offset
        );
    }

    let offset_value = emit_push_int(script, offset as i128);

    // Effective address = unsigned base + static offset, computed exactly
    // (NeoVM integers are arbitrary precision). It is intentionally NOT
    // re-masked to 32 bits: the downstream bounds check rejects any address
    // that reaches or exceeds the memory size, which is the WASM
    // out-of-bounds trap. Re-masking would wrap a >= 2^32 address back into
    // range and turn a trap into a silent wrong-address access.
    let added = emit_binary_op(script, "ADD", base, offset_value, |a, b| {
        // Both operands are now non-negative (base in [0, 2^32), offset in
        // [0, i64::MAX]); defer to the runtime ADD if the constant sum would
        // not fit in i64 (it is still bounds-checked downstream).
        let result = a.checked_add(b)?;
        if result > i64::MAX as i128 {
            return None;
        }
        Some(result)
    })?;
    Ok(StackValue {
        const_value: added.const_value,
        bytecode_start: None,
        pending_sign_extend: None,
    })
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
    if dst_mem != 0 {
        bail!("destination memory {} is not supported; only memory index 0 is supported for memory.copy", dst_mem);
    }
    if src_mem != 0 {
        bail!(
            "source memory {} is not supported; only memory index 0 is supported for memory.copy",
            src_mem
        );
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
