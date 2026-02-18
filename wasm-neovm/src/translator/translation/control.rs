use anyhow::{anyhow, bail, Result};
use wasmparser::{BlockType, FuncType, ValType};

use crate::numeric;

use crate::translator::helpers::*;
use crate::translator::types::StackValue;

/// Represents different kinds of control flow constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlKind {
    Block,
    Loop,
    If,
    Function,
}

/// Represents a control flow frame on the control stack.
#[derive(Debug, Clone)]
pub(super) struct ControlFrame {
    pub(super) kind: ControlKind,
    pub(super) stack_height: usize,
    pub(super) result_count: usize, // Expected number of results (for Function and Block types)
    pub(super) start_offset: usize,
    pub(super) end_fixups: Vec<usize>,
    pub(super) if_false_fixup: Option<usize>,
    pub(super) has_else: bool,
    /// Whether the instruction that introduced this frame was reachable.
    /// Used to correctly model WASM reachability merges (eg `if` without `else`).
    pub(super) entry_reachable: bool,
    /// Whether any reachable branch can transfer control to this frame's `end` label.
    /// (Loops branch to their start label and do not contribute to end reachability.)
    pub(super) end_reachable_from_branch: bool,
    /// For `if` frames, records whether the `then` branch can reach the `end` label.
    pub(super) if_then_end_reachable: Option<bool>,
}

/// Returns the result types for a block type using slice-based returns (Round 65 optimization)
fn block_result_types<'a>(
    ty: BlockType,
    types: &'a [FuncType],
    single_type_buffer: &'a mut [ValType; 1],
) -> Result<&'a [ValType]> {
    match ty {
        BlockType::Empty => Ok(&[]),
        BlockType::Type(ty) => {
            single_type_buffer[0] = ty;
            Ok(&single_type_buffer[..])
        }
        BlockType::FuncType(idx) => {
            let func = types
                .get(idx as usize)
                .ok_or_else(|| anyhow!("block type index {} out of bounds", idx))?;
            if !func.params().is_empty() {
                bail!(
                    "block type index {} carries parameters; block parameters are unsupported",
                    idx
                );
            }
            Ok(func.results())
        }
    }
}

pub(super) fn block_result_count(ty: BlockType, types: &[FuncType]) -> Result<usize> {
    // Stack-allocated buffer for single-type results (Round 61, 65 optimizations)
    let mut single_type_buffer = [ValType::I32; 1];
    let results = block_result_types(ty, types, &mut single_type_buffer)?;

    if results.len() > 1 {
        bail!("blocks with multi-value results are not supported");
    }
    for ty in results {
        match ty {
            ValType::I32 | ValType::I64 => {}
            ValType::F32 | ValType::F64 => return numeric::unsupported_float("block result type"),
            ValType::V128 => return numeric::unsupported_simd("block result type"),
            ValType::Ref(_) => return numeric::unsupported_reference_type("block result type"),
        }
    }

    Ok(results.len())
}

pub(super) fn handle_branch(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut [ControlFrame],
    relative_depth: usize,
    conditional: bool,
    is_unreachable: &mut bool,
) -> Result<()> {
    if relative_depth >= control_stack.len() {
        bail!(
            "branch depth {} exceeds current control stack",
            relative_depth
        );
    }
    let target_index = control_stack.len() - 1 - relative_depth;
    let (prefix, _) = control_stack.split_at_mut(target_index + 1);
    let frame = &mut prefix[target_index];

    // Any reachable branch to a non-loop label makes the corresponding `end` label reachable.
    if !*is_unreachable && !matches!(frame.kind, ControlKind::Loop) {
        frame.end_reachable_from_branch = true;
    }

    // Only validate stack height if we're not already in unreachable code.
    // For Function frames: branching means providing return values, validate against result_count.
    // For other frames: branching means jumping to end of block, validate against entry height + results.
    if !*is_unreachable {
        let expected = match frame.kind {
            ControlKind::Function => frame.result_count,
            ControlKind::Loop => frame.stack_height,
            _ => frame.stack_height + frame.result_count,
        };
        if value_stack.len() != expected {
            bail!(
                "branch requires {} values but current stack has {}",
                expected,
                value_stack.len()
            );
        }
    }

    match frame.kind {
        ControlKind::Loop => {
            let opcode = if conditional { "JMPIF_L" } else { "JMP_L" };
            emit_jump_to(script, opcode, frame.start_offset)?;
        }
        _ => {
            let opcode = if conditional { "JMPIF_L" } else { "JMP_L" };
            let pos = emit_jump_placeholder(script, opcode)?;
            frame.end_fixups.push(pos);
        }
    }

    if !conditional {
        // For Function frames, keep result_count values on stack for return.
        // For other frames, truncate to stack_height + result_count so block results are preserved.
        let target_size = match frame.kind {
            ControlKind::Function => frame.result_count,
            ControlKind::Loop => frame.stack_height,
            _ => frame.stack_height + frame.result_count,
        };
        value_stack.truncate(target_size);
        // Unconditional branch makes subsequent code unreachable.
        *is_unreachable = true;
    }

    Ok(())
}

pub(super) fn handle_br_table(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut [ControlFrame],
    index: StackValue,
    targets: &[usize],
    default_depth: usize,
    is_unreachable: &mut bool,
) -> Result<()> {
    if let Some(const_idx) = index.const_value {
        if index.bytecode_start.is_some() {
            // Do not rewind script bytes here; it can invalidate pending
            // jump/call fixups tracked in surrounding control frames.
        }
        script.push(lookup_opcode("DROP")?.byte);
        let idx = if const_idx < 0 || const_idx > usize::MAX as i128 {
            usize::MAX
        } else {
            const_idx as usize
        };
        let depth = targets.get(idx).copied().unwrap_or(default_depth);
        handle_branch(
            script,
            value_stack,
            control_stack,
            depth,
            false,
            is_unreachable,
        )?;
        return Ok(());
    }

    emit_br_table_dynamic(
        script,
        value_stack,
        control_stack,
        targets,
        default_depth,
        is_unreachable,
    )
}

fn emit_br_table_dynamic(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut [ControlFrame],
    targets: &[usize],
    default_depth: usize,
    is_unreachable: &mut bool,
) -> Result<()> {
    let dup = lookup_opcode("DUP")?.byte;
    let equal = lookup_opcode("EQUAL")?.byte;
    let drop = lookup_opcode("DROP")?.byte;

    let mut case_fixups: Vec<(usize, usize)> = Vec::with_capacity(targets.len());

    for (case_index, &depth) in targets.iter().enumerate() {
        script.push(dup);
        let _ = emit_push_int(script, case_index as i128);
        script.push(equal);
        let fixup = emit_jump_placeholder(script, "JMPIF_L")?;
        case_fixups.push((fixup, depth));
    }

    let origin_unreachable = *is_unreachable;

    script.push(drop);
    let mut branch_unreachable = origin_unreachable;
    handle_branch(
        script,
        value_stack,
        control_stack,
        default_depth,
        false,
        &mut branch_unreachable,
    )?;

    for (fixup, depth) in case_fixups {
        let label_pos = script.len();
        patch_jump(script, fixup, label_pos)?;
        script.push(drop);
        let mut branch_unreachable = origin_unreachable;
        handle_branch(
            script,
            value_stack,
            control_stack,
            depth,
            false,
            &mut branch_unreachable,
        )?;
    }

    // `br_table` is an unconditional branch (even though its targets are dynamic).
    *is_unreachable = true;

    Ok(())
}
