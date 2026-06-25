// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Instruction-level lowering from Move opcodes to WASM
//!
//! This module handles the translation of individual Move opcodes to WASM
//! instructions, including:
//! - Arithmetic and logical operations
//! - Control flow (branches, returns)
//! - Local variable access
//! - Resource operations (MoveTo, MoveFrom, etc.)
//! - Stack management

use crate::bytecode::{AbilitySet, MoveModule, MoveOpcode};
use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use wasm_encoder::{BlockType, Function, Instruction};

use super::super::resources::{
    ensure_has_key, struct_for_index, write_resource_key, write_resource_value,
};
use super::imports::{
    SCRATCH_KEY_OFFSET, SCRATCH_KEY_SIZE, SCRATCH_VALUE_OFFSET, SCRATCH_VALUE_SIZE,
};
use super::{ImportLayout, ValueKind};

/// Emit the body for a single Move opcode
///
/// This function translates one Move opcode into the corresponding WASM
/// instruction sequence, handling stack state transitions and control flow.
#[allow(clippy::too_many_arguments)]
pub fn emit_case_body(
    module: &MoveModule,
    opcode: &MoveOpcode,
    idx: usize,
    dispatch_depth: u32,
    pc_local: u32,
    tmp_a_local: u32,
    tmp_b_local: u32,
    import_offset: u32,
    imports: &ImportLayout,
    needs_storage: bool,
    _struct_lookup: &HashMap<String, AbilitySet>,
    stack_state: &[ValueKind],
    stack_after: &[ValueKind],
    stack_slots: &[u32],
    slot_types: &[ValueKind],
    f: &mut Function,
) -> Result<()> {
    let next_pc = (idx + 1) as i32;

    restore_stack(f, stack_state, stack_slots, slot_types);

    match opcode {
        MoveOpcode::LdU8(v) => {
            f.instruction(&Instruction::I32Const(*v as i32));
        }
        MoveOpcode::LdU64(v) => {
            f.instruction(&Instruction::I64Const(*v as i64));
        }
        MoveOpcode::LdU128(v) => {
            // Move u128 constants wider than i64 cannot be represented in the
            // translator's current 64-bit value model. Bail loudly at translation
            // time rather than silently truncating (X11).
            if *v > i64::MAX as u128 {
                bail!("Move u128 constant {v} exceeds i64::MAX; the translator's 64-bit value model cannot represent it (X11)");
            }
            f.instruction(&Instruction::I64Const(*v as i64));
        }
        MoveOpcode::LdTrue => {
            f.instruction(&Instruction::I32Const(1));
        }
        MoveOpcode::LdFalse => {
            f.instruction(&Instruction::I32Const(0));
        }
        MoveOpcode::LdConst(v) => {
            f.instruction(&Instruction::I64Const(*v as i64));
        }

        MoveOpcode::CopyLoc(idx) | MoveOpcode::MoveLoc(idx) | MoveOpcode::MutBorrowLoc(idx) => {
            f.instruction(&Instruction::LocalGet(*idx as u32));
        }
        MoveOpcode::StLoc(idx) => {
            f.instruction(&Instruction::LocalSet(*idx as u32));
        }
        MoveOpcode::ImmBorrowLoc(idx) => {
            f.instruction(&Instruction::LocalGet(*idx as u32));
        }

        MoveOpcode::Add => {
            // Move integer arithmetic aborts on overflow (X6). Move integers
            // are unsigned here, so `a + b` overflows iff `sum < a`.
            f.instruction(&Instruction::LocalSet(tmp_b_local)); // tmp_b = b
            f.instruction(&Instruction::LocalSet(tmp_a_local)); // tmp_a = a
            f.instruction(&Instruction::LocalGet(tmp_a_local));
            f.instruction(&Instruction::LocalGet(tmp_b_local));
            f.instruction(&Instruction::I64Add); // [sum]
                                                 // Save sum to tmp_b (b no longer needed) and probe sum<a without
                                                 // losing sum: compare a reloaded copy, then re-push sum afterward.
            f.instruction(&Instruction::LocalTee(tmp_b_local)); // [sum]; tmp_b = sum
            f.instruction(&Instruction::LocalGet(tmp_a_local)); // [sum, a]
            f.instruction(&Instruction::I64LtU); // [sum<a] (consumes sum+a)
            f.instruction(&Instruction::If(BlockType::Empty));
            f.instruction(&Instruction::Unreachable);
            f.instruction(&Instruction::End);
            f.instruction(&Instruction::LocalGet(tmp_b_local)); // [sum]
        }
        MoveOpcode::Sub => {
            // Unsigned subtraction overflows iff a < b.
            f.instruction(&Instruction::LocalSet(tmp_b_local)); // tmp_b = b
            f.instruction(&Instruction::LocalSet(tmp_a_local)); // tmp_a = a
            f.instruction(&Instruction::LocalGet(tmp_a_local));
            f.instruction(&Instruction::LocalGet(tmp_b_local));
            f.instruction(&Instruction::I64Sub); // [a-b]
                                                 // Probe a<b: push a and b AFTER the result so the diff survives.
            f.instruction(&Instruction::LocalGet(tmp_a_local)); // [a-b, a]
            f.instruction(&Instruction::LocalGet(tmp_b_local)); // [a-b, a, b]
            f.instruction(&Instruction::I64LtU); // [a-b, a<b]
            f.instruction(&Instruction::If(BlockType::Empty));
            f.instruction(&Instruction::Unreachable);
            f.instruction(&Instruction::End);
            // [a-b]
        }
        MoveOpcode::Mul => {
            // Unsigned mul overflows iff a != 0 && (a*b)/a != b. Guard the
            // division by a!=0 first (else WASM div-by-zero traps).
            f.instruction(&Instruction::LocalSet(tmp_b_local)); // tmp_b = b
            f.instruction(&Instruction::LocalSet(tmp_a_local)); // tmp_a = a
                                                                // Compute product into tmp_a (reuse; we still need a but can recompute).
            f.instruction(&Instruction::LocalGet(tmp_a_local));
            f.instruction(&Instruction::LocalGet(tmp_b_local));
            f.instruction(&Instruction::I64Mul);
            f.instruction(&Instruction::LocalSet(tmp_b_local)); // tmp_b = product (b gone)
                                                                // Probe: a != 0 && product / a != b. Branch on a != 0.
            f.instruction(&Instruction::LocalGet(tmp_a_local));
            f.instruction(&Instruction::I64Const(0));
            f.instruction(&Instruction::I64Ne); // [a!=0]
            f.instruction(&Instruction::If(BlockType::Empty));
            {
                f.instruction(&Instruction::LocalGet(tmp_b_local)); // product
                f.instruction(&Instruction::LocalGet(tmp_a_local)); // a
                f.instruction(&Instruction::I64DivU); // product/a
                f.instruction(&Instruction::LocalGet(tmp_a_local)); // (b is gone; recompute b? no)
                                                                    // We lost b. Re-derive: reload b by recomputing is impossible.
                                                                    // Instead, check (product/a) * a == product. If not, overflow.
                f.instruction(&Instruction::I64Mul); // (product/a)*a
                f.instruction(&Instruction::LocalGet(tmp_b_local)); // product
                f.instruction(&Instruction::I64Ne); // [(product/a)*a != product]
                f.instruction(&Instruction::If(BlockType::Empty));
                f.instruction(&Instruction::Unreachable);
                f.instruction(&Instruction::End);
            }
            f.instruction(&Instruction::End);
            // Return the product.
            f.instruction(&Instruction::LocalGet(tmp_b_local)); // [product]
        }
        MoveOpcode::Div => {
            // Move integers are unsigned; use unsigned division
            f.instruction(&Instruction::I64DivU);
        }
        MoveOpcode::Mod => {
            // Move integers are unsigned; use unsigned remainder
            f.instruction(&Instruction::I64RemU);
        }

        MoveOpcode::Lt => {
            // Move integers are unsigned; use unsigned comparison
            f.instruction(&Instruction::I64LtU);
        }
        MoveOpcode::Gt => {
            f.instruction(&Instruction::I64GtU);
        }
        MoveOpcode::Le => {
            f.instruction(&Instruction::I64LeU);
        }
        MoveOpcode::Ge => {
            f.instruction(&Instruction::I64GeU);
        }
        MoveOpcode::Eq => {
            f.instruction(&Instruction::I64Eq);
        }
        MoveOpcode::Neq => {
            f.instruction(&Instruction::I64Ne);
        }

        MoveOpcode::And => {
            f.instruction(&Instruction::I32And);
        }
        MoveOpcode::Or => {
            f.instruction(&Instruction::I32Or);
        }
        MoveOpcode::Not => {
            f.instruction(&Instruction::I32Eqz);
        }

        MoveOpcode::Branch(_) => {}
        MoveOpcode::BrTrue(_) => {}
        MoveOpcode::BrFalse(_) => {}
        MoveOpcode::Call(idx) => {
            f.instruction(&Instruction::Call(import_offset + *idx as u32));
        }
        MoveOpcode::Ret => {}
        MoveOpcode::Abort => {}

        MoveOpcode::MoveTo(struct_idx) => {
            let struct_def = struct_for_index(module, *struct_idx)?;
            ensure_has_key(struct_def)?;
            if !needs_storage {
                bail!("resource operations require storage imports");
            }

            f.instruction(&Instruction::LocalSet(tmp_a_local)); // value
            f.instruction(&Instruction::LocalSet(tmp_b_local)); // address

            // Move's resource linearity: move_to ABORTS if a resource already
            // exists under the key (X10). Probe existence first and trap.
            write_resource_key(f, struct_def, tmp_b_local);
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_OFFSET));
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_SIZE));
            f.instruction(&Instruction::Call(
                imports
                    .storage_get
                    .ok_or_else(|| anyhow!("storage_get import missing"))?,
            ));
            f.instruction(&Instruction::I64Eqz); // 0 -> absent, 1 -> present
            f.instruction(&Instruction::I32Eqz); // abort when present (present -> 0 -> eqz 1)
            f.instruction(&Instruction::If(BlockType::Empty));
            f.instruction(&Instruction::Unreachable); // resource already exists
            f.instruction(&Instruction::End);

            write_resource_key(f, struct_def, tmp_b_local);
            write_resource_value(f, tmp_a_local);

            f.instruction(&Instruction::I32Const(SCRATCH_KEY_OFFSET));
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_SIZE));
            f.instruction(&Instruction::I32Const(SCRATCH_VALUE_OFFSET));
            f.instruction(&Instruction::I32Const(SCRATCH_VALUE_SIZE));
            f.instruction(&Instruction::Call(
                imports
                    .storage_put
                    .ok_or_else(|| anyhow!("storage_put import missing"))?,
            ));
        }
        MoveOpcode::MoveFrom(struct_idx) => {
            let struct_def = struct_for_index(module, *struct_idx)?;
            ensure_has_key(struct_def)?;
            if !needs_storage {
                bail!("resource operations require storage imports");
            }

            f.instruction(&Instruction::LocalSet(tmp_b_local)); // address
            write_resource_key(f, struct_def, tmp_b_local);
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_OFFSET));
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_SIZE));
            f.instruction(&Instruction::Call(
                imports
                    .storage_get
                    .ok_or_else(|| anyhow!("storage_get import missing"))?,
            ));
            // move_from ABORTS if the resource is absent (X10). Trap on absence.
            f.instruction(&Instruction::I64Eqz); // 1 -> absent
            f.instruction(&Instruction::If(BlockType::Empty));
            f.instruction(&Instruction::Unreachable); // resource absent
            f.instruction(&Instruction::End);
        }
        MoveOpcode::Exists(struct_idx) => {
            let struct_def = struct_for_index(module, *struct_idx)?;
            ensure_has_key(struct_def)?;
            if !needs_storage {
                bail!("resource operations require storage imports");
            }
            f.instruction(&Instruction::LocalSet(tmp_b_local)); // address
            write_resource_key(f, struct_def, tmp_b_local);
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_OFFSET));
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_SIZE));
            f.instruction(&Instruction::Call(
                imports
                    .storage_get
                    .ok_or_else(|| anyhow!("storage_get import missing"))?,
            ));
            f.instruction(&Instruction::I64Eqz);
            f.instruction(&Instruction::I32Eqz); // exists -> !eqz
        }
        MoveOpcode::BorrowGlobal(struct_idx) | MoveOpcode::MutBorrowGlobal(struct_idx) => {
            let struct_def = struct_for_index(module, *struct_idx)?;
            ensure_has_key(struct_def)?;
            if !needs_storage {
                bail!("resource operations require storage imports");
            }
            f.instruction(&Instruction::LocalSet(tmp_b_local)); // address
            write_resource_key(f, struct_def, tmp_b_local);
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_OFFSET));
            f.instruction(&Instruction::I32Const(SCRATCH_KEY_SIZE));
            f.instruction(&Instruction::Call(
                imports
                    .storage_get
                    .ok_or_else(|| anyhow!("storage_get import missing"))?,
            ));
        }

        MoveOpcode::Pack(struct_idx) => {
            // Struct packing is not modelled in the translator's 64-bit value
            // representation; bail loudly at translation time rather than
            // silently dropping field values and pushing a placeholder (X12).
            let _ = struct_for_index(module, *struct_idx)?;
            bail!("unsupported Move feature: Pack (struct construction) is not modelled by the translator's flat value representation (X12)");
        }
        MoveOpcode::Unpack(struct_idx) => {
            let _ = struct_for_index(module, *struct_idx)?;
            bail!("unsupported Move feature: Unpack (struct deconstruction) is not modelled by the translator (X12)");
        }
        MoveOpcode::BorrowField(_) | MoveOpcode::MutBorrowField(_) => {
            bail!("unsupported Move feature: BorrowField/MutBorrowField (struct field access) is not modelled by the translator (X12)");
        }

        MoveOpcode::Pop => {
            f.instruction(&Instruction::Drop);
        }

        MoveOpcode::VecPack(_, _) => {
            bail!("unsupported Move feature: VecPack (vector construction) is not modelled by the translator (X12)");
        }
        MoveOpcode::VecLen(_) => {
            bail!("unsupported Move feature: VecLen is not modelled by the translator (X12)");
        }
        MoveOpcode::VecImmBorrow(_) | MoveOpcode::VecMutBorrow(_) => {
            bail!("unsupported Move feature: VecImmBorrow/VecMutBorrow (vector element access) is not modelled by the translator (X12)");
        }
        MoveOpcode::VecPushBack(_) | MoveOpcode::VecPopBack(_) => {
            bail!("unsupported Move feature: VecPushBack/VecPopBack (vector mutation) is not modelled by the translator (X12)");
        }

        MoveOpcode::CastU8 => {
            // `as u8` must mask to the low 8 bits (X6): I32WrapI64 keeps the
            // low 32 bits, so explicitly AND with 0xFF.
            f.instruction(&Instruction::I32WrapI64);
            f.instruction(&Instruction::I32Const(0xFF));
            f.instruction(&Instruction::I32And);
        }
        MoveOpcode::CastU64 | MoveOpcode::CastU128 => {
            f.instruction(&Instruction::Nop);
        }

        MoveOpcode::Nop => {
            f.instruction(&Instruction::Nop);
        }
    }

    match opcode {
        MoveOpcode::Branch(target) => {
            store_stack(f, stack_after, stack_slots, slot_types);
            f.instruction(&Instruction::I32Const(*target as i32));
            f.instruction(&Instruction::LocalSet(pc_local));
            f.instruction(&Instruction::Br(dispatch_depth));
            return Ok(());
        }
        MoveOpcode::BrTrue(target) => {
            f.instruction(&Instruction::If(BlockType::Empty));
            store_stack(f, stack_after, stack_slots, slot_types);
            f.instruction(&Instruction::I32Const(*target as i32));
            f.instruction(&Instruction::LocalSet(pc_local));
            f.instruction(&Instruction::Br(dispatch_depth + 1));
            f.instruction(&Instruction::Else);
            store_stack(f, stack_after, stack_slots, slot_types);
            f.instruction(&Instruction::I32Const(next_pc));
            f.instruction(&Instruction::LocalSet(pc_local));
            f.instruction(&Instruction::Br(dispatch_depth + 1));
            f.instruction(&Instruction::End);
            return Ok(());
        }
        MoveOpcode::BrFalse(target) => {
            f.instruction(&Instruction::I32Eqz);
            f.instruction(&Instruction::If(BlockType::Empty));
            store_stack(f, stack_after, stack_slots, slot_types);
            f.instruction(&Instruction::I32Const(*target as i32));
            f.instruction(&Instruction::LocalSet(pc_local));
            f.instruction(&Instruction::Br(dispatch_depth + 1));
            f.instruction(&Instruction::Else);
            store_stack(f, stack_after, stack_slots, slot_types);
            f.instruction(&Instruction::I32Const(next_pc));
            f.instruction(&Instruction::LocalSet(pc_local));
            f.instruction(&Instruction::Br(dispatch_depth + 1));
            f.instruction(&Instruction::End);
            return Ok(());
        }
        MoveOpcode::Ret => {
            f.instruction(&Instruction::Return);
            return Ok(());
        }
        MoveOpcode::Abort => {
            f.instruction(&Instruction::Unreachable);
            return Ok(());
        }
        _ => {}
    }

    store_stack(f, stack_after, stack_slots, slot_types);
    f.instruction(&Instruction::I32Const(next_pc));
    f.instruction(&Instruction::LocalSet(pc_local));
    f.instruction(&Instruction::Br(dispatch_depth));

    Ok(())
}

/// Restore stack state from local slots
///
/// This function loads values from local variables back onto the WASM stack,
/// performing type conversions as needed.
pub fn restore_stack(
    func: &mut Function,
    state: &[ValueKind],
    slots: &[u32],
    slot_types: &[ValueKind],
) {
    for (idx, kind) in state.iter().enumerate() {
        let slot = slots[idx];
        match slot_types.get(idx).copied().unwrap_or(ValueKind::I64) {
            ValueKind::I32 => {
                func.instruction(&Instruction::LocalGet(slot));
            }
            ValueKind::I64 => {
                func.instruction(&Instruction::LocalGet(slot));
                if matches!(kind, ValueKind::I32) {
                    func.instruction(&Instruction::I32WrapI64);
                }
            }
        }
    }
}

/// Store stack state to local slots
///
/// This function saves values from the WASM stack into local variables,
/// performing type conversions as needed.
pub fn store_stack(
    func: &mut Function,
    state: &[ValueKind],
    slots: &[u32],
    slot_types: &[ValueKind],
) {
    for (idx, kind) in state.iter().enumerate().rev() {
        let slot = slots[idx];
        match slot_types.get(idx).copied().unwrap_or(ValueKind::I64) {
            ValueKind::I32 => {
                if matches!(kind, ValueKind::I64) {
                    func.instruction(&Instruction::I32WrapI64);
                }
                func.instruction(&Instruction::LocalSet(slot));
            }
            ValueKind::I64 => {
                if matches!(kind, ValueKind::I32) {
                    func.instruction(&Instruction::I64ExtendI32U);
                }
                func.instruction(&Instruction::LocalSet(slot));
            }
        }
    }
}
