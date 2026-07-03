// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;

pub(in crate::translator::runtime) const WASM_MEMORY_PAGE_BYTES: i128 = 65_536;

fn emit_ldloc(script: &mut Vec<u8>, slot: u8) -> Result<()> {
    let name = match slot {
        0 => "LDLOC0",
        1 => "LDLOC1",
        2 => "LDLOC2",
        3 => "LDLOC3",
        4 => "LDLOC4",
        5 => "LDLOC5",
        6 => "LDLOC6",
        _ => bail!("chunked memory helper local {} is out of range", slot),
    };
    script.push(lookup_opcode(name)?.byte);
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_load_byte_at_local(
    script: &mut Vec<u8>,
    address_local: u8,
) -> Result<()> {
    script.push(op::LDSFLD0);
    emit_ldloc(script, address_local)?;
    let _ = emit_push_int(script, WASM_MEMORY_PAGE_BYTES);
    script.push(op::DIV);
    script.push(op::PICKITEM);
    emit_ldloc(script, address_local)?;
    let _ = emit_push_int(script, WASM_MEMORY_PAGE_BYTES);
    script.push(op::MOD);
    script.push(op::PICKITEM);
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_store_byte_at_local(
    script: &mut Vec<u8>,
    address_local: u8,
    value_local: u8,
) -> Result<()> {
    script.push(op::LDSFLD0);
    emit_ldloc(script, address_local)?;
    let _ = emit_push_int(script, WASM_MEMORY_PAGE_BYTES);
    script.push(op::DIV);
    script.push(op::PICKITEM);
    emit_ldloc(script, address_local)?;
    let _ = emit_push_int(script, WASM_MEMORY_PAGE_BYTES);
    script.push(op::MOD);
    emit_ldloc(script, value_local)?;
    script.push(op::SETITEM);
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_new_page(script: &mut Vec<u8>) -> Result<()> {
    let _ = emit_push_int(script, WASM_MEMORY_PAGE_BYTES);
    script.push(op::NEWBUFFER);
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_copy_literal_to_memory(
    script: &mut Vec<u8>,
    dest_offset: u64,
    bytes: &[u8],
) -> Result<()> {
    let mut copied = 0usize;
    while copied < bytes.len() {
        let absolute = dest_offset as usize + copied;
        let page_offset = absolute % WASM_MEMORY_PAGE_BYTES as usize;
        let page_remaining = WASM_MEMORY_PAGE_BYTES as usize - page_offset;
        let chunk_len = page_remaining.min(bytes.len() - copied);
        let page_index = absolute / WASM_MEMORY_PAGE_BYTES as usize;

        script.push(op::LDSFLD0);
        let _ = emit_push_int(script, page_index as i128);
        script.push(op::PICKITEM);
        let _ = emit_push_int(script, page_offset as i128);
        emit_push_data(script, &bytes[copied..copied + chunk_len])?;
        script.push(op::PUSH0);
        let _ = emit_push_int(script, chunk_len as i128);
        script.push(op::MEMCPY);

        copied += chunk_len;
    }
    Ok(())
}
