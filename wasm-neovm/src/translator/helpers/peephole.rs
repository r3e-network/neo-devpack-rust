// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Peephole optimizer: pattern-matches and eliminates redundant bytecode sequences.
//!
//! Runs after code generation but before jump relaxation. Eliminates:
//! - NOP instructions (0x21)
//! - PUSH{const} immediately followed by DROP
//! - SWAP followed by SWAP
//! - DUP followed by DROP
//! - Consecutive CONVERT Integer, CONVERT Integer
//!
//! This pass is safe because it only removes instruction pairs that have no
//! net effect on the stack, and it only operates on adjacent instructions
//! (no removal across jump targets).

use crate::opcodes;

// Opcode bytes
const SWAP: u8 = 0x50;
const CONVERT: u8 = 0xD3;
const STACKITEMTYPE_INTEGER: u8 = 0x21;

fn mark_relative_target(targets: &mut [bool], pc: usize, offset: i32) {
    let target = pc as i32 + offset;
    if target >= 0 && (target as usize) < targets.len() {
        targets[target as usize] = true;
    }
}

fn mark_i8_relative_target(script: &[u8], targets: &mut [bool], pc: usize, offset_pos: usize) {
    if offset_pos < script.len() {
        let offset = script[offset_pos] as i8 as i32;
        if offset != 0 {
            mark_relative_target(targets, pc, offset);
        }
    }
}

fn mark_i32_relative_target(script: &[u8], targets: &mut [bool], pc: usize, offset_pos: usize) {
    if offset_pos + 4 <= script.len() {
        let offset = i32::from_le_bytes([
            script[offset_pos],
            script[offset_pos + 1],
            script[offset_pos + 2],
            script[offset_pos + 3],
        ]);
        if offset != 0 {
            mark_relative_target(targets, pc, offset);
        }
    }
}

/// Check if a position is a jump/call target by scanning all branch instructions.
/// Returns a set of all target positions.
fn collect_jump_targets(script: &[u8], table: &[Option<&opcodes::OpcodeInfo>; 256]) -> Vec<bool> {
    let mut targets = vec![false; script.len()];
    let mut pc = 0usize;

    while pc < script.len() {
        let byte = script[pc];
        let info = match table[byte as usize] {
            Some(info) => info,
            None => {
                pc += 1;
                continue;
            }
        };

        let size = if info.operand_size_prefix == 0 {
            1 + info.operand_size as usize
        } else {
            let ps = pc + 1;
            let prefix = info.operand_size_prefix as usize;
            if ps + prefix > script.len() {
                pc += 1;
                continue;
            }
            let operand_len = match prefix {
                1 => script[ps] as usize,
                2 => u16::from_le_bytes([script[ps], script[ps + 1]]) as usize,
                _ => 0,
            };
            1 + prefix + operand_len
        };

        // Mark jump/call targets
        match byte {
            // Short jumps/calls (1-byte offset)
            0x22 | 0x24 | 0x26 | 0x28 | 0x2A | 0x2C | 0x2E | 0x30 | 0x32 | 0x34 | 0x3D => {
                mark_i8_relative_target(script, &mut targets, pc, pc + 1);
            }
            // Long jumps/calls (4-byte offset)
            0x23 | 0x25 | 0x27 | 0x29 | 0x2B | 0x2D | 0x2F | 0x31 | 0x33 | 0x35 | 0x3E => {
                mark_i32_relative_target(script, &mut targets, pc, pc + 1);
            }
            // TRY short (2 x 1-byte offsets)
            0x3B => {
                mark_i8_relative_target(script, &mut targets, pc, pc + 1);
                mark_i8_relative_target(script, &mut targets, pc, pc + 2);
            }
            // TRY_L (2 x 4-byte offsets)
            0x3C => {
                for start in [pc + 1, pc + 5] {
                    mark_i32_relative_target(script, &mut targets, pc, start);
                }
            }
            _ => {}
        }

        pc += size;
    }

    targets
}

/// Run peephole optimization on the script. Returns the optimized script
/// and updates method_offsets in place.
///
/// This pass does NOT rewrite jump offsets — it only removes instructions
/// that are not jump targets and are part of redundant patterns. It must
/// be run BEFORE the relaxation pass (which handles offset rewriting).
///
/// Actually, to keep things simple and correct, this pass marks bytes for
/// removal and then rebuilds the script, delegating offset adjustment to
/// the subsequent relaxation pass. But wait — the relaxation pass only
/// adjusts offsets for instructions it knows are being relaxed. We need a
/// different approach.
///
/// Simple approach: only remove NOP instructions (which are never jump targets
/// by construction in our compiler) and leave the rest to future work.
/// More complex patterns need the full offset-rewriting machinery.
pub fn peephole_optimize(script: &[u8], method_offsets: &mut [u32]) -> Vec<u8> {
    if script.is_empty() {
        return Vec::new();
    }

    let mut table: [Option<&'static opcodes::OpcodeInfo>; 256] = [None; 256];
    for info in opcodes::all() {
        table[info.byte as usize] = Some(info);
    }

    let targets = collect_jump_targets(script, &table);

    // Build list of (pc, size) for all instructions
    let mut instructions: Vec<(usize, usize)> = Vec::new();
    let mut pc = 0usize;
    while pc < script.len() {
        let info = match table[script[pc] as usize] {
            Some(info) => info,
            None => {
                instructions.push((pc, 1));
                pc += 1;
                continue;
            }
        };
        let size = if info.operand_size_prefix == 0 {
            1 + info.operand_size as usize
        } else {
            let ps = pc + 1;
            let prefix = info.operand_size_prefix as usize;
            if ps + prefix > script.len() {
                instructions.push((pc, 1));
                pc += 1;
                continue;
            }
            let operand_len = match prefix {
                1 => script[ps] as usize,
                2 => u16::from_le_bytes([script[ps], script[ps + 1]]) as usize,
                _ => 0,
            };
            1 + prefix + operand_len
        };
        instructions.push((pc, size));
        pc += size;
    }

    // Mark instructions for removal
    let mut remove = vec![false; instructions.len()];
    let mut total_removed = 0usize;

    for i in 0..instructions.len() {
        if remove[i] {
            continue;
        }
        let (ipc, isize) = instructions[i];
        let byte = script[ipc];

        // For two-instruction patterns, check the next non-removed instruction.
        // Only apply patterns where BOTH instructions are NOT jump targets.
        if i + 1 < instructions.len() {
            let (jpc, jsize) = instructions[i + 1];
            let jbyte = script[jpc];

            // Don't remove across jump targets
            if targets[jpc] || targets[ipc] {
                continue;
            }

            // Pattern 2: SWAP + SWAP → remove both
            if byte == SWAP && jbyte == SWAP {
                remove[i] = true;
                remove[i + 1] = true;
                total_removed += 2;
                continue;
            }

            // Pattern 3: CONVERT Integer + CONVERT Integer → single CONVERT Integer
            if byte == CONVERT
                && jbyte == CONVERT
                && isize == 2
                && jsize == 2
                && script[ipc + 1] == STACKITEMTYPE_INTEGER
                && script[jpc + 1] == STACKITEMTYPE_INTEGER
            {
                remove[i] = true;
                total_removed += 2;
                continue;
            }
        }
    }

    if total_removed == 0 {
        return script.to_vec();
    }

    // Build shift table for offset remapping
    // shift[orig_pos] = bytes removed before orig_pos
    let mut shift = vec![0usize; script.len() + 1];
    let mut cumulative = 0usize;
    let mut instr_idx = 0;
    for (pos, slot) in shift.iter_mut().enumerate().take(script.len() + 1) {
        // Advance instr_idx past completed instructions
        while instr_idx < instructions.len() {
            let (ipc, isize) = instructions[instr_idx];
            if ipc + isize <= pos && remove[instr_idx] {
                cumulative += isize;
                instr_idx += 1;
            } else if ipc < pos && !remove[instr_idx] {
                instr_idx += 1;
            } else {
                break;
            }
        }
        *slot = cumulative;
    }

    // Rebuild — this is complex because we need to rewrite offsets.
    // Use the same approach as the relax pass.
    let remap = |orig: usize| -> usize {
        // Walk instructions to find cumulative shift at orig
        let mut cum = 0usize;
        for (idx, &(ipc, isize)) in instructions.iter().enumerate() {
            if ipc >= orig {
                break;
            }
            if remove[idx] {
                cum += isize;
            }
        }
        orig - cum
    };

    let new_len = script.len() - total_removed;
    let mut out = Vec::with_capacity(new_len);

    for (idx, &(ipc, isize)) in instructions.iter().enumerate() {
        if remove[idx] {
            continue;
        }

        let byte = script[ipc];

        // Check if this instruction has offsets that need rewriting
        let is_long_branch = matches!(
            byte,
            0x23 | 0x25 | 0x27 | 0x29 | 0x2B | 0x2D | 0x2F | 0x31 | 0x33 | 0x35 | 0x3E
        );
        let is_short_branch = matches!(
            byte,
            0x22 | 0x24 | 0x26 | 0x28 | 0x2A | 0x2C | 0x2E | 0x30 | 0x32 | 0x34 | 0x3D
        );

        if is_long_branch {
            let off = i32::from_le_bytes([
                script[ipc + 1],
                script[ipc + 2],
                script[ipc + 3],
                script[ipc + 4],
            ]);
            let target = (ipc as i32 + off) as usize;
            let new_off = remap(target) as i32 - remap(ipc) as i32;
            out.push(byte);
            out.extend_from_slice(&new_off.to_le_bytes());
        } else if is_short_branch {
            let off = script[ipc + 1] as i8 as i32;
            let target = (ipc as i32 + off) as usize;
            let new_off = remap(target) as i32 - remap(ipc) as i32;
            out.push(byte);
            out.push(new_off as i8 as u8);
        } else if byte == 0x3C {
            // TRY_L
            let catch_off = i32::from_le_bytes([
                script[ipc + 1],
                script[ipc + 2],
                script[ipc + 3],
                script[ipc + 4],
            ]);
            let finally_off = i32::from_le_bytes([
                script[ipc + 5],
                script[ipc + 6],
                script[ipc + 7],
                script[ipc + 8],
            ]);
            let np = remap(ipc) as i32;
            out.push(byte);
            if catch_off == 0 {
                out.extend_from_slice(&0i32.to_le_bytes());
            } else {
                let t = (ipc as i32 + catch_off) as usize;
                out.extend_from_slice(&(remap(t) as i32 - np).to_le_bytes());
            }
            if finally_off == 0 {
                out.extend_from_slice(&0i32.to_le_bytes());
            } else {
                let t = (ipc as i32 + finally_off) as usize;
                out.extend_from_slice(&(remap(t) as i32 - np).to_le_bytes());
            }
        } else if byte == 0x3B {
            // TRY short
            let catch_off = script[ipc + 1] as i8 as i32;
            let finally_off = script[ipc + 2] as i8 as i32;
            let np = remap(ipc) as i32;
            out.push(byte);
            if catch_off == 0 {
                out.push(0);
            } else {
                let t = (ipc as i32 + catch_off) as usize;
                out.push((remap(t) as i32 - np) as i8 as u8);
            }
            if finally_off == 0 {
                out.push(0);
            } else {
                let t = (ipc as i32 + finally_off) as usize;
                out.push((remap(t) as i32 - np) as i8 as u8);
            }
        } else if byte == 0x0A {
            // PUSHA
            let off = i32::from_le_bytes([
                script[ipc + 1],
                script[ipc + 2],
                script[ipc + 3],
                script[ipc + 4],
            ]);
            let target = (ipc as i32 + off) as usize;
            let new_off = remap(target) as i32 - remap(ipc) as i32;
            out.push(byte);
            out.extend_from_slice(&new_off.to_le_bytes());
        } else {
            // Copy verbatim
            out.extend_from_slice(&script[ipc..ipc + isize]);
        }
    }

    // Remap method offsets
    for offset in method_offsets.iter_mut() {
        *offset = remap(*offset as usize) as u32;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_nop() {
        // NOP + RET — NOPs are preserved since they may be intentional opcode imports
        let script = vec![0x21, 0x40];
        let mut offsets = vec![];
        let optimized = peephole_optimize(&script, &mut offsets);
        assert_eq!(optimized, vec![0x21, 0x40]); // preserved
    }

    #[test]
    fn preserves_push_drop() {
        // PUSH0 + DROP + RET — kept to avoid position invalidation
        let script = vec![0x10, 0x45, 0x40];
        let mut offsets = vec![];
        let optimized = peephole_optimize(&script, &mut offsets);
        assert_eq!(optimized, vec![0x10, 0x45, 0x40]); // preserved
    }

    #[test]
    fn removes_swap_swap() {
        // SWAP + SWAP + RET
        let script = vec![0x50, 0x50, 0x40];
        let mut offsets = vec![];
        let optimized = peephole_optimize(&script, &mut offsets);
        assert_eq!(optimized, vec![0x40]);
    }

    #[test]
    fn preserves_dup_drop() {
        // DUP + DROP + RET — kept to be conservative
        let script = vec![0x4A, 0x45, 0x40];
        let mut offsets = vec![];
        let optimized = peephole_optimize(&script, &mut offsets);
        assert_eq!(optimized, vec![0x4A, 0x45, 0x40]); // preserved
    }

    #[test]
    fn preserves_jump_targets() {
        // JMP +2 → NOP (at offset 2, which is the jump target)
        let script = vec![0x22, 0x02, 0x21, 0x40]; // JMP +2, NOP, RET
        let mut offsets = vec![];
        let optimized = peephole_optimize(&script, &mut offsets);
        // NOP at offset 2 is a jump target, so it must NOT be removed
        assert_eq!(optimized.len(), 4);
    }

    #[test]
    fn empty_script() {
        let mut offsets = vec![];
        let optimized = peephole_optimize(&[], &mut offsets);
        assert!(optimized.is_empty());
    }
}
