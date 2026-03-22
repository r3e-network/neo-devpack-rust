// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, bail, Result};

use crate::opcodes;

pub(crate) fn validate_script(script: &[u8]) -> Result<()> {
    use std::convert::TryInto;

    if script.is_empty() {
        return Ok(());
    }

    let mut opcode_table: [Option<&'static opcodes::OpcodeInfo>; 256] = [None; 256];
    for info in opcodes::all() {
        opcode_table[info.byte as usize] = Some(info);
    }

    let instruction_size = |pc: usize, info: &opcodes::OpcodeInfo| -> Result<usize> {
        let prefix = info.operand_size_prefix as usize;
        if prefix == 0 {
            let size = 1usize + info.operand_size as usize;
            if pc + size > script.len() {
                bail!(
                    "opcode {} at offset {} extends past end of script (size {}, len {})",
                    info.name,
                    pc,
                    size,
                    script.len()
                );
            }
            return Ok(size);
        }

        let prefix_start = pc + 1;
        let prefix_end = prefix_start + prefix;
        if prefix_end > script.len() {
            bail!(
                "opcode {} at offset {} missing operand length prefix (need {}, len {})",
                info.name,
                pc,
                prefix,
                script.len()
            );
        }

        let operand_len = match prefix {
            1 => script[prefix_start] as usize,
            2 => {
                let bytes: [u8; 2] = script[prefix_start..prefix_end].try_into().map_err(|_| {
                    anyhow!("internal error: cannot read 2-byte prefix at offset {}", pc)
                })?;
                u16::from_le_bytes(bytes) as usize
            }
            4 => {
                let bytes: [u8; 4] = script[prefix_start..prefix_end].try_into().map_err(|_| {
                    anyhow!("internal error: cannot read 4-byte prefix at offset {}", pc)
                })?;
                let raw = i32::from_le_bytes(bytes);
                if raw < 0 {
                    bail!(
                        "opcode {} at offset {} has negative operand length {}",
                        info.name,
                        pc,
                        raw
                    );
                }
                raw as usize
            }
            other => bail!(
                "opcode {} at offset {} has unsupported operand size prefix {}",
                info.name,
                pc,
                other
            ),
        };

        let size = 1usize + prefix + operand_len;
        if pc + size > script.len() {
            bail!(
                "opcode {} at offset {} extends past end of script (size {}, len {})",
                info.name,
                pc,
                size,
                script.len()
            );
        }
        Ok(size)
    };

    let mut boundaries = vec![false; script.len()];
    let mut pc = 0usize;
    while pc < script.len() {
        boundaries[pc] = true;
        let op = script[pc];
        let info = opcode_table[op as usize]
            .ok_or_else(|| anyhow!("unknown opcode 0x{:02X} at offset {}", op, pc))?;
        let size = instruction_size(pc, info)?;
        pc += size;
    }
    if pc != script.len() {
        bail!(
            "script terminates in the middle of an instruction (final pc {}, len {})",
            pc,
            script.len()
        );
    }

    let validate_target = |name: &str, at: usize, offset: i32| -> Result<()> {
        let target = (at as i32).checked_add(offset).ok_or_else(|| {
            anyhow!(
                "{} at offset {} overflows instruction pointer with offset {}",
                name,
                at,
                offset
            )
        })?;
        if target < 0 {
            bail!(
                "{} at offset {} targets negative instruction pointer {}",
                name,
                at,
                target
            );
        }
        let target = target as usize;
        if target >= script.len() {
            bail!(
                "{} at offset {} targets out-of-range instruction pointer {} (len {})",
                name,
                at,
                target,
                script.len()
            );
        }
        if !boundaries[target] {
            bail!(
                "{} at offset {} targets non-instruction boundary {}",
                name,
                at,
                target
            );
        }
        Ok(())
    };

    pc = 0usize;
    while pc < script.len() {
        let op = script[pc];
        let info = opcode_table[op as usize].ok_or_else(|| {
            anyhow!(
                "internal error: opcode 0x{:02X} at offset {} disappeared from table",
                op,
                pc
            )
        })?;
        let size = instruction_size(pc, info)?;

        match info.name {
            "JMP" | "JMPIF" | "JMPIFNOT" | "JMPEQ" | "JMPNE" | "JMPGT" | "JMPGE" | "JMPLT"
            | "JMPLE" | "CALL" | "ENDTRY" => {
                let offset = script[pc + 1] as i8 as i32;
                validate_target(info.name, pc, offset)?;
            }
            "PUSHA" | "JMP_L" | "JMPIF_L" | "JMPIFNOT_L" | "JMPEQ_L" | "JMPNE_L" | "JMPGT_L"
            | "JMPGE_L" | "JMPLT_L" | "JMPLE_L" | "CALL_L" | "ENDTRY_L" => {
                let offset_bytes: [u8; 4] = script[pc + 1..pc + 5].try_into().map_err(|_| {
                    anyhow!("internal error: cannot read 4-byte offset at offset {}", pc)
                })?;
                let offset = i32::from_le_bytes(offset_bytes);
                validate_target(info.name, pc, offset)?;
            }
            "TRY" => {
                let catch_offset = script[pc + 1] as i8 as i32;
                let finally_offset = script[pc + 2] as i8 as i32;
                if catch_offset == 0 && finally_offset == 0 {
                    bail!(
                        "TRY at offset {} must specify a catch and/or finally block",
                        pc
                    );
                }
                validate_target("TRY (catch)", pc, catch_offset)?;
                validate_target("TRY (finally)", pc, finally_offset)?;
            }
            "TRY_L" => {
                let catch_bytes: [u8; 4] = script[pc + 1..pc + 5].try_into().map_err(|_| {
                    anyhow!(
                        "internal error: cannot read TRY_L catch offset at offset {}",
                        pc
                    )
                })?;
                let finally_bytes: [u8; 4] = script[pc + 5..pc + 9].try_into().map_err(|_| {
                    anyhow!(
                        "internal error: cannot read TRY_L finally offset at offset {}",
                        pc
                    )
                })?;
                let catch_offset = i32::from_le_bytes(catch_bytes);
                let finally_offset = i32::from_le_bytes(finally_bytes);
                if catch_offset == 0 && finally_offset == 0 {
                    bail!(
                        "TRY_L at offset {} must specify a catch and/or finally block",
                        pc
                    );
                }
                validate_target("TRY_L (catch)", pc, catch_offset)?;
                validate_target("TRY_L (finally)", pc, finally_offset)?;
            }
            _ => {}
        }

        pc += size;
    }

    Ok(())
}
