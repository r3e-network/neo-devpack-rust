// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! NeoVM helper bodies that bridge the Rust devpack storage facade onto the
//! real `System.Storage.*` SYSCALLs of the executing contract.
//!
//! On wasm32 the devpack exposes pointer/length-encoded storage primitives
//! (`neo_storage_put_bytes`, `neo_storage_delete_bytes`, `neo_storage_get_into`).
//! These helpers are emitted once per contract, called via `CALL_L`, and
//! perform the marshaling between wasm linear memory and the NeoVM evaluation
//! stack: they slice key/value bytes out of memory, dispatch the SYSCALL, and
//! (for `Get`) write the returned bytes back into wasm memory while reporting
//! the actual length to the caller.
//!
//! Two memory layouts are supported:
//! - **compact** — a single `Buffer` held in static slot 0; key/value bytes
//!   come straight off `LDSFLD0` via `SUBSTR` and `MEMCPY` does the writeback.
//! - **chunked** — an `Array` of per-page `Buffer`s for multi-page or growable
//!   memory; bytes are gathered with `emit_chunked_load_byte_at_local` and
//!   scattered with `emit_chunked_store_byte_at_local`, mirroring the loops
//!   used by `memory.copy`/`memory.init` for the same layout.

use anyhow::{anyhow, Result};

use crate::opcodes;
use crate::syscalls;
use crate::translator::helpers::*;

use super::memory::{emit_chunked_load_byte_at_local, emit_chunked_store_byte_at_local};

const SYSCALL_OPERAND_SIZE: u8 = 4;
const STACKITEM_TYPE_BYTESTRING: u8 = 0x28;

fn emit_load_arg(script: &mut Vec<u8>, index: u8) -> Result<()> {
    let opcode_name = match index {
        0 => "LDARG0",
        1 => "LDARG1",
        2 => "LDARG2",
        3 => "LDARG3",
        4 => "LDARG4",
        5 => "LDARG5",
        6 => "LDARG6",
        _ => anyhow::bail!("LDARG slot {} out of range for storage helper", index),
    };
    script.push(lookup_opcode(opcode_name)?.byte);
    Ok(())
}

fn emit_load_local(script: &mut Vec<u8>, index: u8) -> Result<()> {
    let opcode_name = match index {
        0 => "LDLOC0",
        1 => "LDLOC1",
        2 => "LDLOC2",
        3 => "LDLOC3",
        4 => "LDLOC4",
        5 => "LDLOC5",
        6 => "LDLOC6",
        _ => anyhow::bail!("LDLOC slot {} out of range for storage helper", index),
    };
    script.push(lookup_opcode(opcode_name)?.byte);
    Ok(())
}

fn emit_store_local(script: &mut Vec<u8>, index: u8) -> Result<()> {
    let opcode_name = match index {
        0 => "STLOC0",
        1 => "STLOC1",
        2 => "STLOC2",
        3 => "STLOC3",
        4 => "STLOC4",
        5 => "STLOC5",
        6 => "STLOC6",
        _ => anyhow::bail!("STLOC slot {} out of range for storage helper", index),
    };
    script.push(lookup_opcode(opcode_name)?.byte);
    Ok(())
}

fn emit_storage_syscall(script: &mut Vec<u8>, descriptor: &str) -> Result<()> {
    let syscall = syscalls::lookup_extended(descriptor)
        .ok_or_else(|| anyhow!("syscall '{}' not found", descriptor))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
    if opcode.operand_size != SYSCALL_OPERAND_SIZE || opcode.operand_size_prefix != 0 {
        anyhow::bail!("unexpected SYSCALL operand metadata");
    }
    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(())
}

/// Emit a sequence that consumes `(ptr, len)` from arg slots `ptr_arg`/`len_arg`
/// and pushes a `ByteString` containing `memory[ptr..ptr+len]`.
///
/// `compact` mode collapses to a single `LDSFLD0 + SUBSTR`. `chunked` mode
/// allocates a fresh `Buffer`, walks pages byte-by-byte using the same
/// chunked helpers that back `memory.copy`, then converts the buffer to a
/// `ByteString` so it is acceptable as a `System.Storage.*` argument.
///
/// Local slot conventions for chunked mode:
/// - `acc_local` — accumulating buffer
/// - `idx_local` — running byte index
/// - `tmp_local` — scratch slot reused for both the absolute address and the
///   byte read from memory (`emit_chunked_load_byte_at_local` reads from this
///   slot and pushes the byte; we then fold the byte back through
///   `STLOC tmp_local` before re-using it for the next page address).
fn emit_extract_memory_bytes(
    script: &mut Vec<u8>,
    ptr_arg: u8,
    len_arg: u8,
    chunked: bool,
    acc_local: u8,
    idx_local: u8,
    tmp_local: u8,
) -> Result<()> {
    if !chunked {
        script.push(lookup_opcode("LDSFLD0")?.byte);
        emit_load_arg(script, ptr_arg)?;
        emit_load_arg(script, len_arg)?;
        script.push(lookup_opcode("SUBSTR")?.byte);
        return Ok(());
    }

    emit_load_arg(script, len_arg)?;
    script.push(lookup_opcode("NEWBUFFER")?.byte);
    emit_store_local(script, acc_local)?;

    script.push(lookup_opcode("PUSH0")?.byte);
    emit_store_local(script, idx_local)?;

    let loop_start = script.len();
    emit_load_local(script, idx_local)?;
    emit_load_arg(script, len_arg)?;
    script.push(lookup_opcode("EQUAL")?.byte);
    let exit_jump = emit_jump_placeholder(script, "JMPIF_L")?;

    // tmp_local <- ptr + idx
    emit_load_arg(script, ptr_arg)?;
    emit_load_local(script, idx_local)?;
    script.push(lookup_opcode("ADD")?.byte);
    emit_store_local(script, tmp_local)?;

    emit_chunked_load_byte_at_local(script, tmp_local)?;
    emit_store_local(script, tmp_local)?;

    emit_load_local(script, acc_local)?;
    emit_load_local(script, idx_local)?;
    emit_load_local(script, tmp_local)?;
    script.push(lookup_opcode("SETITEM")?.byte);

    emit_load_local(script, idx_local)?;
    script.push(lookup_opcode("INC")?.byte);
    emit_store_local(script, idx_local)?;
    let back_jump = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, back_jump, loop_start)?;

    emit_load_local(script, acc_local)?;
    let convert =
        opcodes::lookup("CONVERT").ok_or_else(|| anyhow!("CONVERT opcode metadata missing"))?;
    if convert.operand_size != 1 || convert.operand_size_prefix != 0 {
        anyhow::bail!("unexpected CONVERT operand metadata");
    }
    script.push(convert.byte);
    script.push(STACKITEM_TYPE_BYTESTRING);
    Ok(())
}

/// Copy `length` bytes from a `ByteString` (top of stack at entry) into wasm
/// memory at `out_ptr`. Consumes the `ByteString` from the stack.
///
/// For chunked memory the value is first stored in `value_local`, then a byte
/// loop scatters its content into individual page buffers via
/// `emit_chunked_store_byte_at_local`. For compact memory we emit the same
/// `MEMCPY` sequence used by `memory.init` for active data segments.
fn emit_write_value_to_memory(
    script: &mut Vec<u8>,
    out_ptr_arg: u8,
    length_local: u8,
    chunked: bool,
    value_local: u8,
    idx_local: u8,
    tmp_local: u8,
) -> Result<()> {
    if !chunked {
        // Stack at entry: [value]
        script.push(lookup_opcode("LDSFLD0")?.byte); // [value, mem]
        script.push(lookup_opcode("SWAP")?.byte); // [mem, value]
        emit_load_arg(script, out_ptr_arg)?; // [mem, value, out_ptr]
        script.push(lookup_opcode("SWAP")?.byte); // [mem, out_ptr, value]
        script.push(lookup_opcode("PUSH0")?.byte); // [mem, out_ptr, value, 0]
        emit_load_local(script, length_local)?; // [mem, out_ptr, value, 0, length]
        script.push(lookup_opcode("MEMCPY")?.byte);
        return Ok(());
    }

    emit_store_local(script, value_local)?; // value out of the stack into a local
    script.push(lookup_opcode("PUSH0")?.byte);
    emit_store_local(script, idx_local)?; // idx = 0

    let byte_local = tmp_local
        .checked_add(1)
        .ok_or_else(|| anyhow!("storage helper slot wrap"))?;

    let loop_start = script.len();
    emit_load_local(script, idx_local)?;
    emit_load_local(script, length_local)?;
    script.push(lookup_opcode("EQUAL")?.byte);
    let exit_jump = emit_jump_placeholder(script, "JMPIF_L")?;

    // tmp_local = out_ptr + idx (absolute wasm address for this byte).
    emit_load_arg(script, out_ptr_arg)?;
    emit_load_local(script, idx_local)?;
    script.push(lookup_opcode("ADD")?.byte);
    emit_store_local(script, tmp_local)?;

    // byte_local = value[idx].
    emit_load_local(script, value_local)?;
    emit_load_local(script, idx_local)?;
    script.push(lookup_opcode("PICKITEM")?.byte);
    emit_store_local(script, byte_local)?;

    emit_chunked_store_byte_at_local(script, tmp_local, byte_local)?;

    emit_load_local(script, idx_local)?;
    script.push(lookup_opcode("INC")?.byte);
    emit_store_local(script, idx_local)?;
    let back_jump = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, back_jump, loop_start)?;

    Ok(())
}

/// Emit `neo_storage_put_bytes(key_ptr, key_len, value_ptr, value_len)`.
///
/// Compact-memory layout (3 locals are unused but reserved so the slot map is
/// stable across modes):
///   ```text
///   INITSLOT 0, 4
///   SYSCALL System.Storage.GetContext
///   SUBSTR <key>
///   SUBSTR <value>
///   SYSCALL System.Storage.Put
///   RET
///   ```
///
/// Chunked-memory layout uses 5 locals (acc, idx, tmp for key build; same set
/// reused for value build) to scatter pages into `Buffer` accumulators before
/// passing them to `Storage.Put`.
/// Emit `neo_storage_put_bytes(key_ptr, key_len, value_ptr, value_len)`.
///
/// INITSLOT pops arguments TOP-FIRST into `Arguments[0..N]`, so for a wasm
/// `call $neo_storage_put_bytes` the slot mapping ends up REVERSED relative
/// to the C signature:
///   ARG0 = value_len   ARG1 = value_ptr
///   ARG2 = key_len     ARG3 = key_ptr
///
/// We then push `[value, key, ctx]` (ctx on top) before SYSCALL Put: Neo's
/// `ApplicationEngine.OnSysCall` pops `descriptor.Parameters[0..N]` from the
/// top down, so the FIRST C# parameter (`StorageContext`) must sit on top of
/// the evaluation stack at SYSCALL time.
pub(in crate::translator::runtime) fn emit_storage_put_helper(
    script: &mut Vec<u8>,
    chunked: bool,
) -> Result<()> {
    let local_count = if chunked { 3 } else { 0 };
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(local_count);
    script.push(4); // 4 args

    emit_extract_memory_bytes(script, 1, 0, chunked, 0, 1, 2)?; // value (ARG1=ptr, ARG0=len)
    emit_extract_memory_bytes(script, 3, 2, chunked, 0, 1, 2)?; // key   (ARG3=ptr, ARG2=len)
    emit_storage_syscall(script, "System.Storage.GetContext")?; // ctx (top)
    emit_storage_syscall(script, "System.Storage.Put")?;
    script.push(lookup_opcode("RET")?.byte);
    Ok(())
}

/// Emit `neo_storage_delete_bytes(key_ptr, key_len)`.
///
/// INITSLOT slot mapping (top-first pop): ARG0 = key_len, ARG1 = key_ptr.
pub(in crate::translator::runtime) fn emit_storage_delete_helper(
    script: &mut Vec<u8>,
    chunked: bool,
) -> Result<()> {
    let local_count = if chunked { 3 } else { 0 };
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(local_count);
    script.push(2); // 2 args

    // `Delete(ctx, key)`: stack at SYSCALL must be `[key, ctx]` (ctx on top).
    emit_extract_memory_bytes(script, 1, 0, chunked, 0, 1, 2)?; // key (ARG1=ptr, ARG0=len)
    emit_storage_syscall(script, "System.Storage.GetContext")?; // ctx (top)
    emit_storage_syscall(script, "System.Storage.Delete")?;
    script.push(lookup_opcode("RET")?.byte);
    Ok(())
}

/// Emit `neo_storage_get_into(key_ptr, key_len, out_ptr, out_cap) -> i32`.
///
/// Returns:
/// - the actual byte length written into wasm memory on success (`>= 0`),
/// - `-1` when the key is not present in storage,
/// - `-(needed_length)` when the caller-provided buffer is too small to hold
///   the value; the caller can grow the buffer and retry.
///
/// Local layout:
/// - slot 0: cached value length
/// - slots 1..=4 (chunked only): accumulating buffer / running index / scratch
///   for byte address / scratch for byte value, mirroring the same convention
///   used by `memory.copy`/`memory.init` chunked helpers.
pub(in crate::translator::runtime) fn emit_storage_get_helper(
    script: &mut Vec<u8>,
    chunked: bool,
) -> Result<()> {
    // INITSLOT slot mapping (top-first pop) for
    // `neo_storage_get_into(key_ptr, key_len, out_ptr, out_cap) -> i32`:
    //   ARG0 = out_cap   ARG1 = out_ptr
    //   ARG2 = key_len   ARG3 = key_ptr
    //
    // Slot 0: cached value length. Chunked mode adds slots 1..=4 for
    // buffer/index/addr/byte (see emit_extract_memory_bytes /
    // emit_write_value_to_memory comments).
    let local_count = if chunked { 5 } else { 1 };
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(local_count);
    script.push(4); // 4 args: key_ptr, key_len, out_ptr, out_cap

    // `Get(ctx, key)`: stack at SYSCALL must be `[key, ctx]` (ctx on top).
    emit_extract_memory_bytes(script, 3, 2, chunked, 1, 2, 3)?; // key (ARG3=ptr, ARG2=len)
    emit_storage_syscall(script, "System.Storage.GetContext")?; // ctx (top)
    emit_storage_syscall(script, "System.Storage.Get")?;
    // Stack: [value-or-null]

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("ISNULL")?.byte);
    let jump_not_found = emit_jump_placeholder(script, "JMPIF_L")?;
    // Fallthrough stack (value not null): [value]

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_arg(script, 0)?; // out_cap (ARG0)
    script.push(lookup_opcode("GT")?.byte);
    let jump_too_small = emit_jump_placeholder(script, "JMPIF_L")?;
    // Fallthrough stack: [value]

    emit_write_value_to_memory(script, 1, 0, chunked, 1, 2, 3)?; // out_ptr=ARG1, length cached in LOC0

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("RET")?.byte);

    let not_found_label = script.len();
    // Stack at entry: [null]
    script.push(lookup_opcode("DROP")?.byte);
    let _ = emit_push_int(script, -1);
    script.push(lookup_opcode("RET")?.byte);

    let too_small_label = script.len();
    // Stack at entry: [value]
    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("NEGATE")?.byte);
    script.push(lookup_opcode("RET")?.byte);

    patch_jump(script, jump_not_found, not_found_label)?;
    patch_jump(script, jump_too_small, too_small_label)?;
    Ok(())
}
