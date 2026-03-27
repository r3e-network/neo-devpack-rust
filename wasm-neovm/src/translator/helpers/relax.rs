// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Jump/call relaxation pass: converts long-form branch/call instructions
//! to short-form when the offset fits in a signed byte, saving 3 bytes each.
//!
//! NeoVM has paired instructions:
//!   JMP_L (0x23, 5 bytes) ↔ JMP (0x22, 2 bytes)
//!   JMPIF_L (0x25) ↔ JMPIF (0x24)  ...etc
//!   CALL_L (0x35, 5 bytes) ↔ CALL (0x34, 2 bytes)
//!   ENDTRY_L (0x3E, 5 bytes) ↔ ENDTRY (0x3D, 2 bytes)
//!   TRY_L (0x3C, 9 bytes) ↔ TRY (0x3B, 3 bytes)
//!
//! This pass iteratively relaxes instructions until a fixed point is reached,
//! because shrinking one instruction can bring others into short-form range.

use crate::opcodes;

/// Information about a relaxable instruction.
struct Candidate {
    pc: usize,
    orig_size: usize,
    short_size: usize,
    relaxed: bool,
}

/// Long-to-short opcode byte mapping.
fn long_to_short(byte: u8) -> Option<(u8, usize)> {
    match byte {
        0x23 => Some((0x22, 2)), // JMP_L → JMP
        0x25 => Some((0x24, 2)), // JMPIF_L → JMPIF
        0x27 => Some((0x26, 2)), // JMPIFNOT_L → JMPIFNOT
        0x29 => Some((0x28, 2)), // JMPEQ_L → JMPEQ
        0x2B => Some((0x2A, 2)), // JMPNE_L → JMPNE
        0x2D => Some((0x2C, 2)), // JMPGT_L → JMPGT
        0x2F => Some((0x2E, 2)), // JMPGE_L → JMPGE
        0x31 => Some((0x30, 2)), // JMPLT_L → JMPLT
        0x33 => Some((0x32, 2)), // JMPLE_L → JMPLE
        0x35 => Some((0x34, 2)), // CALL_L → CALL
        0x3E => Some((0x3D, 2)), // ENDTRY_L → ENDTRY
        0x3C => Some((0x3B, 3)), // TRY_L → TRY
        _ => None,
    }
}

fn build_opcode_table() -> [Option<&'static opcodes::OpcodeInfo>; 256] {
    let mut table: [Option<&'static opcodes::OpcodeInfo>; 256] = [None; 256];
    for info in opcodes::all() {
        table[info.byte as usize] = Some(info);
    }
    table
}

fn instr_size(script: &[u8], pc: usize, table: &[Option<&opcodes::OpcodeInfo>; 256]) -> usize {
    let info = match table[script[pc] as usize] {
        Some(info) => info,
        None => return 1,
    };
    let prefix = info.operand_size_prefix as usize;
    if prefix == 0 {
        return 1 + info.operand_size as usize;
    }
    let ps = pc + 1;
    if ps + prefix > script.len() {
        return 1;
    }
    let operand_len = match prefix {
        1 => script[ps] as usize,
        2 => u16::from_le_bytes([script[ps], script[ps + 1]]) as usize,
        4 => {
            let raw = i32::from_le_bytes([script[ps], script[ps + 1], script[ps + 2], script[ps + 3]]);
            if raw < 0 { 0 } else { raw as usize }
        }
        _ => 0,
    };
    1 + prefix + operand_len
}

fn read_i32_le(script: &[u8], pos: usize) -> i32 {
    i32::from_le_bytes([script[pos], script[pos + 1], script[pos + 2], script[pos + 3]])
}

/// Build a per-byte shift table: shift[i] = total bytes removed by relaxed
/// instructions that START strictly before position i.
/// An instruction at position `pc` maps to new position `pc - shift[pc]`.
fn build_shift_table(script_len: usize, candidates: &[Candidate]) -> Vec<usize> {
    let mut shift = vec![0usize; script_len + 1];
    let mut cumulative = 0usize;
    let mut ci = 0;
    for (i, slot) in shift.iter_mut().enumerate().take(script_len + 1) {
        // Skip candidates at position i (they don't affect their own position)
        while ci < candidates.len() && candidates[ci].pc == i {
            ci += 1;
        }
        *slot = cumulative;
        // After writing shift[i], add savings from any relaxed candidate at i
        for c in candidates.iter() {
            if c.pc == i && c.relaxed {
                cumulative += c.orig_size - c.short_size;
            }
        }
    }
    shift
}

/// Relax the bytecode script, converting long-form jumps/calls to short-form
/// where possible. Returns the relaxed script.
///
/// `method_offsets` are updated in-place to reflect new positions.
pub fn relax_script(script: &[u8], method_offsets: &mut [u32]) -> Vec<u8> {
    if script.is_empty() {
        return Vec::new();
    }

    let table = build_opcode_table();

    // Phase 1: Find all relaxable long-form instructions.
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut pc = 0usize;
    while pc < script.len() {
        let size = instr_size(script, pc, &table);
        if let Some((_short_byte, short_size)) = long_to_short(script[pc]) {
            candidates.push(Candidate { pc, orig_size: size, short_size, relaxed: false });
        }
        pc += size;
    }

    if candidates.is_empty() {
        return script.to_vec();
    }

    // Phase 2: Iterative relaxation until fixed point.
    loop {
        let mut changed = false;
        let shift = build_shift_table(script.len(), &candidates);

        #[allow(clippy::needless_range_loop)]
        for i in 0..candidates.len() {
            if candidates[i].relaxed {
                continue;
            }
            let c = &candidates[i];
            let opcode = script[c.pc];
            let new_pc = (c.pc - shift[c.pc]) as i32;

            if opcode == 0x3C {
                // TRY_L: check both offsets
                let catch_off = read_i32_le(script, c.pc + 1);
                let finally_off = read_i32_le(script, c.pc + 5);

                let catch_fits = catch_off == 0 || {
                    let t = (c.pc as i32 + catch_off) as usize;
                    let new_t = (t - shift[t]) as i32;
                    (i8::MIN as i32..=i8::MAX as i32).contains(&(new_t - new_pc))
                };
                let finally_fits = finally_off == 0 || {
                    let t = (c.pc as i32 + finally_off) as usize;
                    let new_t = (t - shift[t]) as i32;
                    (i8::MIN as i32..=i8::MAX as i32).contains(&(new_t - new_pc))
                };

                if catch_fits && finally_fits {
                    candidates[i].relaxed = true;
                    changed = true;
                }
            } else {
                let orig_offset = read_i32_le(script, c.pc + 1);
                let target = (c.pc as i32 + orig_offset) as usize;
                let new_target = (target - shift[target]) as i32;
                let new_offset = new_target - new_pc;

                if (i8::MIN as i32..=i8::MAX as i32).contains(&new_offset) {
                    candidates[i].relaxed = true;
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    // Check if anything was relaxed
    if !candidates.iter().any(|c| c.relaxed) {
        return script.to_vec();
    }

    // Phase 3: Rebuild with final shift table.
    let shift = build_shift_table(script.len(), &candidates);

    let remap = |orig: usize| -> usize { orig - shift[orig] };

    let new_len = script.len() - shift[script.len()];
    let mut out = Vec::with_capacity(new_len);
    let mut ci = 0usize;

    pc = 0;
    while pc < script.len() {
        let size = instr_size(script, pc, &table);
        let opcode = script[pc];

        // Is this a relaxed candidate?
        let is_relaxed_candidate = ci < candidates.len()
            && candidates[ci].pc == pc
            && candidates[ci].relaxed;
        let is_candidate = ci < candidates.len() && candidates[ci].pc == pc;
        if is_candidate {
            ci += 1;
        }

        if is_relaxed_candidate {
            let (short_byte, _) = long_to_short(opcode).unwrap();

            if opcode == 0x3C {
                let catch_off = read_i32_le(script, pc + 1);
                let finally_off = read_i32_le(script, pc + 5);
                let new_pc_i32 = remap(pc) as i32;

                out.push(short_byte);
                out.push(if catch_off == 0 { 0 } else {
                    let t = (pc as i32 + catch_off) as usize;
                    (remap(t) as i32 - new_pc_i32) as i8 as u8
                });
                out.push(if finally_off == 0 { 0 } else {
                    let t = (pc as i32 + finally_off) as usize;
                    (remap(t) as i32 - new_pc_i32) as i8 as u8
                });
            } else {
                let orig_offset = read_i32_le(script, pc + 1);
                let target = (pc as i32 + orig_offset) as usize;
                let new_offset = remap(target) as i32 - remap(pc) as i32;

                out.push(short_byte);
                out.push(new_offset as i8 as u8);
            }
        } else {
            // Emit instruction, rewriting any offsets it contains.
            let is_long = matches!(opcode, 0x23|0x25|0x27|0x29|0x2B|0x2D|0x2F|0x31|0x33|0x35|0x3E);
            let is_short = matches!(opcode, 0x22|0x24|0x26|0x28|0x2A|0x2C|0x2E|0x30|0x32|0x34|0x3D);

            if is_long {
                let off = read_i32_le(script, pc + 1);
                let target = (pc as i32 + off) as usize;
                let new_off = remap(target) as i32 - remap(pc) as i32;
                out.push(opcode);
                out.extend_from_slice(&new_off.to_le_bytes());
            } else if is_short {
                let off = script[pc + 1] as i8 as i32;
                let target = (pc as i32 + off) as usize;
                let new_off = remap(target) as i32 - remap(pc) as i32;
                out.push(opcode);
                out.push(new_off as i8 as u8);
            } else if opcode == 0x3C {
                // TRY_L not relaxed
                let catch_off = read_i32_le(script, pc + 1);
                let finally_off = read_i32_le(script, pc + 5);
                let np = remap(pc) as i32;
                out.push(opcode);
                if catch_off == 0 {
                    out.extend_from_slice(&0i32.to_le_bytes());
                } else {
                    let t = (pc as i32 + catch_off) as usize;
                    out.extend_from_slice(&(remap(t) as i32 - np).to_le_bytes());
                }
                if finally_off == 0 {
                    out.extend_from_slice(&0i32.to_le_bytes());
                } else {
                    let t = (pc as i32 + finally_off) as usize;
                    out.extend_from_slice(&(remap(t) as i32 - np).to_le_bytes());
                }
            } else if opcode == 0x3B {
                // TRY short
                let catch_off = script[pc + 1] as i8 as i32;
                let finally_off = script[pc + 2] as i8 as i32;
                let np = remap(pc) as i32;
                out.push(opcode);
                if catch_off == 0 { out.push(0); } else {
                    let t = (pc as i32 + catch_off) as usize;
                    out.push((remap(t) as i32 - np) as i8 as u8);
                }
                if finally_off == 0 { out.push(0); } else {
                    let t = (pc as i32 + finally_off) as usize;
                    out.push((remap(t) as i32 - np) as i8 as u8);
                }
            } else if opcode == 0x0A {
                // PUSHA
                let off = read_i32_le(script, pc + 1);
                let target = (pc as i32 + off) as usize;
                let new_off = remap(target) as i32 - remap(pc) as i32;
                out.push(opcode);
                out.extend_from_slice(&new_off.to_le_bytes());
            } else {
                out.extend_from_slice(&script[pc..pc + size]);
            }
        }

        pc += size;
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
    fn relax_short_jmp_l() {
        // JMP_L +5 at offset 0, target = offset 5, NOP at offset 5
        let script = vec![
            0x23, 0x05, 0x00, 0x00, 0x00, // JMP_L +5 (target = 5)
            0x21, // NOP at offset 5
        ];
        let mut offsets = vec![];
        let relaxed = relax_script(&script, &mut offsets);

        assert_eq!(relaxed.len(), 3); // JMP (2) + NOP (1)
        assert_eq!(relaxed[0], 0x22); // JMP short
        assert_eq!(relaxed[1] as i8, 2); // +2 (past 2-byte JMP to NOP)
        assert_eq!(relaxed[2], 0x21); // NOP
    }

    #[test]
    fn no_relax_large_offset() {
        let mut script = vec![0x23]; // JMP_L
        script.extend_from_slice(&200i32.to_le_bytes());
        script.resize(205, 0x21);

        let original_len = script.len();
        let mut offsets = vec![];
        let relaxed = relax_script(&script, &mut offsets);
        assert_eq!(relaxed.len(), original_len);
    }

    #[test]
    fn relax_updates_method_offsets() {
        let script = vec![
            0x23, 0x05, 0x00, 0x00, 0x00, // JMP_L +5 at offset 0
            0x10, // PUSH0 at offset 5
            0x40, // RET at offset 6
        ];
        let mut offsets = vec![5u32];
        let relaxed = relax_script(&script, &mut offsets);

        assert_eq!(relaxed.len(), 4); // JMP(2) + PUSH0(1) + RET(1)
        assert_eq!(offsets[0], 2); // was 5, now 2
    }

    #[test]
    fn relax_call_l_backward() {
        // CALL_L at offset 3 with offset -3 → target = 0
        let script = vec![
            0x21, // NOP at 0
            0x21, // NOP at 1
            0x21, // NOP at 2
            0x35, 0xFD, 0xFF, 0xFF, 0xFF, // CALL_L -3 (target = 0)
        ];
        let mut offsets = vec![];
        let relaxed = relax_script(&script, &mut offsets);

        assert_eq!(relaxed.len(), 5); // 3 NOPs + CALL(2)
        assert_eq!(relaxed[3], 0x34); // CALL short
        assert_eq!(relaxed[4] as i8, -3); // offset -3
    }

    #[test]
    fn empty_script() {
        let mut offsets = vec![];
        let relaxed = relax_script(&[], &mut offsets);
        assert!(relaxed.is_empty());
    }

    #[test]
    fn multiple_relaxations() {
        // Two JMP_L instructions jumping forward
        let script = vec![
            0x23, 0x0A, 0x00, 0x00, 0x00, // JMP_L +10 at 0, target = 10
            0x23, 0x05, 0x00, 0x00, 0x00, // JMP_L +5 at 5, target = 10
            0x21, // NOP at 10
        ];
        let mut offsets = vec![];
        let relaxed = relax_script(&script, &mut offsets);

        // Both should be relaxed: JMP(2) + JMP(2) + NOP(1) = 5 bytes
        assert_eq!(relaxed.len(), 5);
        assert_eq!(relaxed[0], 0x22); // JMP short
        assert_eq!(relaxed[2], 0x22); // JMP short
    }
}
