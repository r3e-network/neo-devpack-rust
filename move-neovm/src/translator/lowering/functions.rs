//! Function lowering orchestration
//!
//! This module handles the translation of Move functions to WASM, including:
//! - Function section generation
//! - Code section generation with dispatch loop
//! - Local variable management
//! - Stack slot allocation

use crate::bytecode::{AbilitySet, FunctionDef, MoveModule};
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use wasm_encoder::{BlockType, CodeSection, Function, FunctionSection, Instruction, ValType};

use super::super::analysis::{analyze_stack, derive_slot_types, effective_locals, stack_effect, val_type_from_tag};
use super::super::resources::build_struct_lookup;
use super::instructions::emit_case_body;
use super::ImportLayout;

/// Build function and code sections
///
/// Returns:
/// - FunctionSection: Function type references
/// - CodeSection: Function bodies with locals and instructions
pub fn build_functions(
    module: &MoveModule,
    func_type_indices: &[u32],
    imported_functions: u32,
    import_layout: &ImportLayout,
    needs_storage: bool,
) -> Result<(FunctionSection, CodeSection)> {
    let mut functions = FunctionSection::new();
    let mut code = CodeSection::new();
    let struct_lookup = build_struct_lookup(&module.structs);

    for (idx, func) in module.functions.iter().enumerate() {
        functions.function(func_type_indices[idx]);
        let wasm_func = lower_function(
            module,
            func,
            imported_functions,
            import_layout,
            needs_storage,
            &struct_lookup,
        )
        .with_context(|| format!("lowering function {}", func.name))?;
        code.function(&wasm_func);
    }

    Ok((functions, code))
}

/// Lower a single Move function into a WASM function body
///
/// This function implements a dispatch loop pattern to handle arbitrary
/// Move bytecode control flow without exceeding WASM block depth limits.
fn lower_function(
    module: &MoveModule,
    func: &FunctionDef,
    import_offset: u32,
    imports: &ImportLayout,
    needs_storage: bool,
    struct_lookup: &HashMap<String, AbilitySet>,
) -> Result<Function> {
    let locals = effective_locals(func);
    if locals.len() < func.parameters.len() {
        bail!(
            "function {} has {} parameters but only {} locals",
            func.name,
            func.parameters.len(),
            locals.len()
        );
    }

    let stack_states = analyze_stack(func, module, struct_lookup)?;
    let slot_types = derive_slot_types(&stack_states);

    // Additional locals: user-declared (beyond params) + pc + scratch temporaries
    let mut local_types: Vec<ValType> = locals[func.parameters.len()..]
        .iter()
        .map(val_type_from_tag)
        .collect();
    let base_local_index = func.parameters.len() as u32;
    let pc_local = base_local_index + local_types.len() as u32;
    local_types.push(ValType::I32);
    let tmp_a_local = base_local_index + local_types.len() as u32;
    local_types.push(ValType::I64);
    let tmp_b_local = base_local_index + local_types.len() as u32;
    local_types.push(ValType::I64);
    let mut stack_slots = Vec::new();
    for kind in &slot_types {
        let idx = base_local_index + local_types.len() as u32;
        stack_slots.push(idx);
        local_types.push(kind.val_type());
    }

    let mut wasm_locals = Vec::new();
    for ty in local_types {
        wasm_locals.push((1, ty));
    }

    let case_count = func.code.len();
    let mut f = Function::new(wasm_locals);

    // pc = 0
    f.instruction(&Instruction::I32Const(0));
    f.instruction(&Instruction::LocalSet(pc_local));

    // Dispatch loop
    f.instruction(&Instruction::Loop(BlockType::Empty)); // depth 0
    f.instruction(&Instruction::Block(BlockType::Empty)); // default block depth 1

    // Nest blocks for each instruction (reverse order)
    for _ in 0..case_count {
        f.instruction(&Instruction::Block(BlockType::Empty));
    }

    // br_table dispatch
    f.instruction(&Instruction::LocalGet(pc_local));
    let targets: Vec<_> = (0..case_count as u32).collect();
    f.instruction(&Instruction::BrTable(targets.into(), case_count as u32));

    // Emit case bodies
    for (idx, opcode) in func.code.iter().enumerate() {
        f.instruction(&Instruction::End); // close current block
        let dispatch_depth = (case_count - idx) as u32;
        let entry_state = stack_states.get(idx).cloned().flatten();
        if entry_state.is_none() {
            // Unreachable case: trap and continue
            f.instruction(&Instruction::Unreachable);
            f.instruction(&Instruction::Br(dispatch_depth));
            continue;
        }
        let entry_stack = entry_state.unwrap();
        let (stack_after, _) = stack_effect(
            opcode,
            func,
            module,
            &locals,
            struct_lookup,
            entry_stack.clone(),
        )?;
        emit_case_body(
            module,
            opcode,
            idx,
            dispatch_depth,
            pc_local,
            tmp_a_local,
            tmp_b_local,
            import_offset,
            imports,
            needs_storage,
            struct_lookup,
            &entry_stack,
            &stack_after,
            &stack_slots,
            &slot_types,
            &mut f,
        )?;
    }

    // Default path: trap
    f.instruction(&Instruction::End); // default block
    f.instruction(&Instruction::Unreachable);

    f.instruction(&Instruction::End); // loop
    f.instruction(&Instruction::Unreachable);
    f.instruction(&Instruction::End); // function end

    Ok(f)
}
