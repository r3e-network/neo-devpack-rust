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

use super::super::resources::{ensure_has_key, struct_for_index, struct_hash, write_resource_key, write_resource_value};
use super::imports::{SCRATCH_KEY_OFFSET, SCRATCH_KEY_SIZE, SCRATCH_VALUE_OFFSET, SCRATCH_VALUE_SIZE};
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
            f.instruction(&Instruction::I64Add);
        }
        MoveOpcode::Sub => {
            f.instruction(&Instruction::I64Sub);
        }
        MoveOpcode::Mul => {
            f.instruction(&Instruction::I64Mul);
        }
        MoveOpcode::Div => {
            f.instruction(&Instruction::I64DivS);
        }
        MoveOpcode::Mod => {
            f.instruction(&Instruction::I64RemS);
        }

        MoveOpcode::Lt => {
            f.instruction(&Instruction::I64LtS);
        }
        MoveOpcode::Gt => {
            f.instruction(&Instruction::I64GtS);
        }
        MoveOpcode::Le => {
            f.instruction(&Instruction::I64LeS);
        }
        MoveOpcode::Ge => {
            f.instruction(&Instruction::I64GeS);
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
            let struct_def = struct_for_index(module, *struct_idx)?;
            // Discard field values but keep a placeholder handle
            for _ in &struct_def.fields {
                f.instruction(&Instruction::Drop);
            }
            f.instruction(&Instruction::I64Const(struct_hash(&struct_def.name) as i64));
        }
        MoveOpcode::Unpack(struct_idx) => {
            let struct_def = struct_for_index(module, *struct_idx)?;
            // Drop the struct value, push zeroes for each field
            f.instruction(&Instruction::Drop);
            for _ in &struct_def.fields {
                f.instruction(&Instruction::I64Const(0));
            }
        }
        MoveOpcode::BorrowField(_) | MoveOpcode::MutBorrowField(_) => {
            f.instruction(&Instruction::Unreachable);
        }

        MoveOpcode::Pop => {
            f.instruction(&Instruction::Drop);
        }

        MoveOpcode::VecPack(_, _) => {
            f.instruction(&Instruction::Unreachable);
        }
        MoveOpcode::VecLen(_) => {
            f.instruction(&Instruction::I64Const(0));
        }
        MoveOpcode::VecImmBorrow(_) | MoveOpcode::VecMutBorrow(_) => {
            f.instruction(&Instruction::Unreachable);
        }
        MoveOpcode::VecPushBack(_) | MoveOpcode::VecPopBack(_) => {
            f.instruction(&Instruction::Unreachable);
        }

        MoveOpcode::CastU8 => {
            f.instruction(&Instruction::I32WrapI64);
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
pub fn store_stack(func: &mut Function, state: &[ValueKind], slots: &[u32], slot_types: &[ValueKind]) {
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
