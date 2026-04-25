// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::Result;

use super::lookup_opcode;
use super::offsets::emit_placeholder;
use super::push::emit_push_int;

/// Emit a CALL_L placeholder for runtime helper calls
///
/// This is a convenience wrapper that defaults to CALL_L opcode
pub fn emit_call_placeholder(script: &mut Vec<u8>) -> Result<usize> {
    emit_placeholder(script, "CALL_L")
}

/// Emit a direct call to a known target.
///
/// Uses compact `CALL` when the relative displacement fits in an `i8`,
/// otherwise falls back to `CALL_L`.
pub fn emit_call_to(script: &mut Vec<u8>, target: usize) -> Result<()> {
    let opcode_pos = script.len();
    let opcode_pos_i64 = i64::try_from(opcode_pos)
        .map_err(|_| anyhow::anyhow!("script offset {} exceeds i64 range", opcode_pos))?;
    let target_i64 = i64::try_from(target)
        .map_err(|_| anyhow::anyhow!("target offset {} exceeds i64 range", target))?;
    let delta_i64 = target_i64 - opcode_pos_i64;

    if (i8::MIN as i64..=i8::MAX as i64).contains(&delta_i64) {
        script.push(lookup_opcode("CALL")?.byte);
        script.push(delta_i64 as i8 as u8);
        return Ok(());
    }

    let delta_i32 = i32::try_from(delta_i64)
        .map_err(|_| anyhow::anyhow!("call delta {} exceeds i32 range", delta_i64))?;
    script.push(lookup_opcode("CALL_L")?.byte);
    script.extend_from_slice(&delta_i32.to_le_bytes());
    Ok(())
}

/// Patch a previously emitted call with the actual target offset
///
/// Re-exported from offsets module for backward compatibility
pub use super::offsets::patch_offset as patch_call;

/// Reverse the top `n` items on the NeoVM evaluation stack.
///
/// This is the standard adapter sequence for calling a wasm-defined function
/// from the translator: WebAssembly pushes args left-to-right (last arg on
/// top), but NeoVM `INITSLOT` pops top-first into `Arguments[0..N]`. Without
/// reversing, `local.get 0` would resolve to the LAST wasm arg in the callee.
/// Reversing the top `n` items before `CALL_L` makes the slot ordering match
/// wasm's `local.get N == argN` invariant.
///
/// `n == 0` and `n == 1` are no-ops. `n == 2` uses `SWAP`; `n == 3`/`4` use
/// the dedicated `REVERSE3`/`REVERSE4` opcodes; larger counts fall back to
/// `PUSH n + REVERSEN`.
pub fn emit_reverse_top_n(script: &mut Vec<u8>, n: usize) -> Result<()> {
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
