// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::Result;

use super::{emit_push_int, lookup_opcode};

/// Mask the top of stack to 32-bit unsigned range (AND with 0xFFFFFFFF).
///
/// Computes the mask as (1 << 32) - 1 inline, which is 6 bytes total
/// (PUSH1 + PUSHINT8[32] + SHL + DEC + AND) instead of the 10-byte
/// alternative of pushing the i64 literal directly.
pub(crate) fn emit_mask_u32(script: &mut Vec<u8>) -> Result<()> {
    let _ = emit_push_int(script, 1); // PUSH1 (1 byte)
    let _ = emit_push_int(script, 32); // PUSHINT8 32 (2 bytes)
    script.push(lookup_opcode("SHL")?.byte); // SHL (1 byte)
    script.push(lookup_opcode("DEC")?.byte); // DEC (1 byte)
    script.push(lookup_opcode("AND")?.byte); // AND (1 byte)
    Ok(())
}
