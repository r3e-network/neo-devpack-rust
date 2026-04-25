// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::lowering::ValueKind;
use super::resources::{ensure_copy_allowed, ensure_has_key, struct_for_index};
use crate::bytecode::{AbilitySet, FunctionDef, MoveModule, MoveOpcode, TypeTag};
use anyhow::{anyhow, bail, Context, Result};
use std::collections::{HashMap, VecDeque};
use wasm_encoder::ValType;

pub(super) fn validate_supported_module(module: &MoveModule) -> Result<()> {
    for func in &module.functions {
        for (idx, tag) in func.parameters.iter().enumerate() {
            validate_supported_type(tag, &format!("function {} parameter {}", func.name, idx))?;
        }
        for (idx, tag) in func.returns.iter().enumerate() {
            validate_supported_type(tag, &format!("function {} return {}", func.name, idx))?;
        }
        for (idx, tag) in effective_locals(func).iter().enumerate() {
            validate_supported_type(tag, &format!("function {} local {}", func.name, idx))?;
        }
        for (pc, opcode) in func.code.iter().enumerate() {
            validate_supported_opcode(func, pc, opcode)?;
        }
    }

    Ok(())
}

pub(super) fn analyze_stack(
    func: &FunctionDef,
    module: &MoveModule,
    struct_lookup: &HashMap<String, AbilitySet>,
) -> Result<Vec<Option<Vec<ValueKind>>>> {
    let locals = effective_locals(func);
    let mut stacks: Vec<Option<Vec<ValueKind>>> = vec![None; func.code.len() + 1];
    stacks[0] = Some(Vec::new());
    let mut work = VecDeque::new();
    work.push_back(0usize);

    while let Some(pc) = work.pop_front() {
        let stack = stacks[pc]
            .clone()
            .ok_or_else(|| anyhow!("missing stack state at pc {}", pc))?;
        let opcode = func
            .code
            .get(pc)
            .ok_or_else(|| anyhow!("pc {} out of bounds", pc))?;
        let (stack_after, jumps) =
            stack_effect(opcode, func, module, &locals, struct_lookup, stack.clone())
                .with_context(|| format!("pc {} opcode {:?}", pc, opcode))?;

        // Fallthrough
        if !is_terminal_opcode(opcode) {
            propagate_stack(pc + 1, &mut stacks, &stack_after, &mut work)?;
        }

        // Branch targets
        for target in jumps {
            propagate_stack(target, &mut stacks, &stack_after, &mut work)?;
        }
    }

    Ok(stacks)
}

fn propagate_stack(
    target: usize,
    stacks: &mut [Option<Vec<ValueKind>>],
    new_stack: &[ValueKind],
    work: &mut VecDeque<usize>,
) -> Result<()> {
    if target >= stacks.len() {
        bail!("branch target {} out of bounds", target);
    }
    match &stacks[target] {
        Some(existing) => {
            if existing != new_stack {
                bail!(
                    "stack mismatch at target {}: {:?} vs {:?}",
                    target,
                    existing,
                    new_stack
                );
            }
        }
        None => {
            stacks[target] = Some(new_stack.to_vec());
            work.push_back(target);
        }
    }
    Ok(())
}

pub(super) fn stack_effect(
    opcode: &MoveOpcode,
    func: &FunctionDef,
    module: &MoveModule,
    locals: &[TypeTag],
    struct_lookup: &HashMap<String, AbilitySet>,
    mut stack: Vec<ValueKind>,
) -> Result<(Vec<ValueKind>, Vec<usize>)> {
    let mut jumps = Vec::new();
    match opcode {
        MoveOpcode::LdU8(_) => stack.push(ValueKind::I32),
        MoveOpcode::LdU64(_) | MoveOpcode::LdU128(_) => stack.push(ValueKind::I64),
        MoveOpcode::LdTrue | MoveOpcode::LdFalse => stack.push(ValueKind::I32),
        MoveOpcode::LdConst(_) => stack.push(ValueKind::I64),

        MoveOpcode::CopyLoc(idx) => {
            let ty = locals
                .get(*idx as usize)
                .ok_or_else(|| anyhow!("local {} out of range", idx))?;
            ensure_copy_allowed(ty, struct_lookup)?;
            stack.push(kind_from_tag(ty));
        }
        MoveOpcode::MoveLoc(idx) => {
            let ty = locals
                .get(*idx as usize)
                .ok_or_else(|| anyhow!("local {} out of range", idx))?;
            stack.push(kind_from_tag(ty));
        }
        MoveOpcode::StLoc(idx) => {
            let ty = locals
                .get(*idx as usize)
                .ok_or_else(|| anyhow!("local {} out of range", idx))?;
            pop_expected(&mut stack, kind_from_tag(ty), "StLoc")?;
        }
        MoveOpcode::MutBorrowLoc(idx) | MoveOpcode::ImmBorrowLoc(idx) => {
            let _ty = locals
                .get(*idx as usize)
                .ok_or_else(|| anyhow!("local {} out of range", idx))?;
            stack.push(ValueKind::I32);
        }

        MoveOpcode::Add | MoveOpcode::Sub | MoveOpcode::Mul | MoveOpcode::Div | MoveOpcode::Mod => {
            pop_expected(&mut stack, ValueKind::I64, "arith lhs")?;
            pop_expected(&mut stack, ValueKind::I64, "arith rhs")?;
            stack.push(ValueKind::I64);
        }

        MoveOpcode::Lt
        | MoveOpcode::Gt
        | MoveOpcode::Le
        | MoveOpcode::Ge
        | MoveOpcode::Eq
        | MoveOpcode::Neq => {
            pop_expected(&mut stack, ValueKind::I64, "cmp lhs")?;
            pop_expected(&mut stack, ValueKind::I64, "cmp rhs")?;
            stack.push(ValueKind::I32);
        }

        MoveOpcode::And | MoveOpcode::Or | MoveOpcode::Not => {
            pop_expected(&mut stack, ValueKind::I32, "boolean op")?;
            if !matches!(opcode, MoveOpcode::Not) {
                pop_expected(&mut stack, ValueKind::I32, "boolean rhs")?;
            }
            stack.push(ValueKind::I32);
        }

        MoveOpcode::Branch(target) => {
            jumps.push(*target as usize);
        }
        MoveOpcode::BrTrue(target) | MoveOpcode::BrFalse(target) => {
            pop_expected(&mut stack, ValueKind::I32, "branch condition")?;
            jumps.push(*target as usize);
        }
        MoveOpcode::Call(idx) => {
            let target = module
                .functions
                .get(*idx as usize)
                .ok_or_else(|| anyhow!("call index {} out of range", idx))?;
            for param in target.parameters.iter().rev() {
                pop_expected(&mut stack, kind_from_tag(param), "call arg")?;
            }
            for ret in &target.returns {
                stack.push(kind_from_tag(ret));
            }
        }
        MoveOpcode::Ret => {
            for ret in func.returns.iter().rev() {
                pop_expected(&mut stack, kind_from_tag(ret), "return value")?;
            }
        }
        MoveOpcode::Abort => {}

        MoveOpcode::MoveTo(struct_idx)
        | MoveOpcode::MoveFrom(struct_idx)
        | MoveOpcode::Exists(struct_idx)
        | MoveOpcode::BorrowGlobal(struct_idx)
        | MoveOpcode::MutBorrowGlobal(struct_idx) => {
            let struct_def = struct_for_index(module, *struct_idx)?;
            ensure_has_key(struct_def)?;
            if matches!(opcode, MoveOpcode::MoveTo(_)) {
                pop_expected(&mut stack, ValueKind::I64, "resource value")?;
                pop_expected(&mut stack, ValueKind::I64, "address")?;
            } else {
                pop_expected(&mut stack, ValueKind::I64, "address")?;
            }
            if matches!(
                opcode,
                MoveOpcode::MoveFrom(_)
                    | MoveOpcode::BorrowGlobal(_)
                    | MoveOpcode::MutBorrowGlobal(_)
            ) {
                stack.push(ValueKind::I64);
            } else if matches!(opcode, MoveOpcode::Exists(_)) {
                stack.push(ValueKind::I32);
            }
        }

        MoveOpcode::Pack(struct_idx) => {
            let struct_def = struct_for_index(module, *struct_idx)?;
            for _ in &struct_def.fields {
                pop_any(&mut stack, "Pack field")?;
            }
            stack.push(ValueKind::I64);
        }
        MoveOpcode::Unpack(struct_idx) => {
            let struct_def = struct_for_index(module, *struct_idx)?;
            pop_any(&mut stack, "Unpack struct")?;
            for _ in &struct_def.fields {
                stack.push(ValueKind::I64);
            }
        }

        MoveOpcode::BorrowField(_) | MoveOpcode::MutBorrowField(_) => {
            pop_any(&mut stack, "borrow_field struct")?;
            stack.push(ValueKind::I32);
        }

        MoveOpcode::Pop => {
            pop_any(&mut stack, "Pop")?;
        }

        MoveOpcode::VecPack(_, _) => stack.push(ValueKind::I32),
        MoveOpcode::VecLen(_) => {
            pop_any(&mut stack, "VecLen")?;
            stack.push(ValueKind::I64);
        }
        MoveOpcode::VecImmBorrow(_) | MoveOpcode::VecMutBorrow(_) => {
            pop_any(&mut stack, "VecBorrow")?;
            stack.push(ValueKind::I32);
        }
        MoveOpcode::VecPushBack(_) | MoveOpcode::VecPopBack(_) => {
            pop_any(&mut stack, "Vec")?;
        }

        MoveOpcode::CastU8 | MoveOpcode::CastU64 | MoveOpcode::CastU128 => {}
        MoveOpcode::Nop => {}
    }

    Ok((stack, jumps))
}

fn pop_expected(stack: &mut Vec<ValueKind>, expected: ValueKind, op: &str) -> Result<ValueKind> {
    let value = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow in {}", op))?;
    if value != expected {
        bail!(
            "type mismatch in {}: expected {:?} got {:?}",
            op,
            expected,
            value
        );
    }
    Ok(value)
}

fn pop_any(stack: &mut Vec<ValueKind>, op: &str) -> Result<ValueKind> {
    stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow in {}", op))
}

fn validate_supported_type(tag: &TypeTag, context: &str) -> Result<()> {
    match tag {
        TypeTag::U128 => {
            bail!(
                "unsupported Move type in {}: u128 requires multi-word lowering and cannot be translated losslessly",
                context
            );
        }
        TypeTag::U256 => {
            bail!(
                "unsupported Move type in {}: u256 requires multi-word lowering and cannot be translated losslessly",
                context
            );
        }
        TypeTag::Vector(inner) | TypeTag::Reference(inner) | TypeTag::MutableReference(inner) => {
            validate_supported_type(inner, context)?
        }
        TypeTag::Bool
        | TypeTag::U8
        | TypeTag::U64
        | TypeTag::Address
        | TypeTag::Signer
        | TypeTag::Struct(_) => {}
    }

    Ok(())
}

fn validate_supported_opcode(func: &FunctionDef, pc: usize, opcode: &MoveOpcode) -> Result<()> {
    let reason = match opcode {
        MoveOpcode::LdU128(_) | MoveOpcode::CastU128 => {
            Some("u128 values would be truncated to i64")
        }
        MoveOpcode::Pack(_)
        | MoveOpcode::Unpack(_)
        | MoveOpcode::BorrowField(_)
        | MoveOpcode::MutBorrowField(_) => {
            Some("struct materialization and field access are not implemented")
        }
        MoveOpcode::MoveTo(_)
        | MoveOpcode::MoveFrom(_)
        | MoveOpcode::Exists(_)
        | MoveOpcode::BorrowGlobal(_)
        | MoveOpcode::MutBorrowGlobal(_) => {
            Some("global resource operations are not implemented losslessly")
        }
        MoveOpcode::VecPack(_, _)
        | MoveOpcode::VecLen(_)
        | MoveOpcode::VecImmBorrow(_)
        | MoveOpcode::VecMutBorrow(_)
        | MoveOpcode::VecPushBack(_)
        | MoveOpcode::VecPopBack(_) => Some("vector operations are not implemented"),
        _ => None,
    };

    if let Some(reason) = reason {
        bail!(
            "unsupported Move opcode {:?} at pc {} in function {}: {}",
            opcode,
            pc,
            func.name,
            reason
        );
    }

    Ok(())
}

fn is_terminal_opcode(opcode: &MoveOpcode) -> bool {
    matches!(
        opcode,
        MoveOpcode::Ret | MoveOpcode::Abort | MoveOpcode::Branch(_)
    )
}

pub(super) fn effective_locals(func: &FunctionDef) -> Vec<TypeTag> {
    if func.locals.is_empty() {
        func.parameters.clone()
    } else {
        func.locals.clone()
    }
}

pub(super) fn val_type_from_tag(tag: &TypeTag) -> ValType {
    kind_from_tag(tag).val_type()
}

pub(super) fn kind_from_tag(tag: &TypeTag) -> ValueKind {
    match tag {
        TypeTag::Bool | TypeTag::U8 => ValueKind::I32,
        TypeTag::U64 | TypeTag::U128 | TypeTag::U256 => ValueKind::I64,
        TypeTag::Address | TypeTag::Signer => ValueKind::I64,
        TypeTag::Vector(_) | TypeTag::Struct(_) => ValueKind::I64,
        TypeTag::Reference(_) | TypeTag::MutableReference(_) => ValueKind::I32,
    }
}

pub(super) fn derive_slot_types(states: &[Option<Vec<ValueKind>>]) -> Vec<ValueKind> {
    let mut slots: Vec<ValueKind> = Vec::new();
    for state in states.iter().filter_map(|s| s.as_ref()) {
        for (idx, kind) in state.iter().enumerate() {
            match slots.get_mut(idx) {
                Some(existing) => {
                    if matches!(kind, ValueKind::I64) {
                        *existing = ValueKind::I64;
                    }
                }
                None => slots.push(*kind),
            }
        }
    }
    slots
}
