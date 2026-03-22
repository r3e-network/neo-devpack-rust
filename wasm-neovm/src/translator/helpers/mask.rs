// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::Result;

use super::{emit_push_int, lookup_opcode};

pub(crate) fn emit_mask_u32(script: &mut Vec<u8>) -> Result<()> {
    let _ = emit_push_int(script, 1);
    let _ = emit_push_int(script, 32);
    script.push(lookup_opcode("SHL")?.byte);
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("SUB")?.byte);
    script.push(lookup_opcode("AND")?.byte);
    Ok(())
}
