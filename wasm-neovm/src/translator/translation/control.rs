// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

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

    // WASM semantics: branching to a label of arity N pops the top N values as
    // the branch operands, DISCARDS every value between the target frame's entry
    // height and those N operands, then transfers control with the N operands
    // becoming the target's results.
    //
    // For Function frames: branching means providing return values, keep the top
    //   `result_count` values, discard everything beneath them.
    // For Loop frames: branching (continue) must match the loop entry stack exactly.
    // For Block/If frames: branching (break) targets the end label; the top
    //   `result_count` values are the operands that must be PRESERVED and become
    //   the block result, while any values buried beneath them (e.g. left there
    //   by wasm-opt code motion, or simply earlier operands on the stack) must be
    //   removed. The number of values to keep on top is N = result_count; the
    //   number of buried values to remove is K = stack_height - frame.stack_height
    //   (i.e. current_len - N - frame.stack_height).
    let keep_top = match frame.kind {
        ControlKind::Function => frame.result_count,
        ControlKind::Loop => 0, // loop continues keep nothing special; strict match below
        _ => frame.result_count,
    };

    if matches!(frame.kind, ControlKind::Loop) {
        // Loop continue: strict-arity match against the loop entry height.
        if !*is_unreachable && value_stack.len() != frame.stack_height {
            bail!(
                "branch requires {} values but current stack has {} (loop continue)",
                frame.stack_height,
                value_stack.len()
            );
        }
        let opcode = if conditional { "JMPIF_L" } else { "JMP_L" };
        emit_jump_to(script, opcode, frame.start_offset)?;
        if !conditional {
            // Unconditional branch makes subsequent code unreachable.
            *is_unreachable = true;
        }
        return Ok(());
    }

    // Floor of the region that survives the branch (base values that belong to
    // the target frame and outlive the branch). For Function returns nothing is
    // kept beneath the results; for Block/If breaks the target frame's entry
    // height is preserved.
    let base = match frame.kind {
        ControlKind::Function => 0,
        _ => frame.stack_height,
    };

    if !conditional {
        // Unconditional `br` / `br_table` target.
        if !*is_unreachable {
            // Ensure the top `keep_top` operands exist. wasm-opt can rearrange
            // code so a branch is reached with fewer values than the target
            // expects; synthesize PUSH0 placeholders for the missing operands
            // (the optimizer proved they are not observably needed). For Function
            // returns a missing result is a genuine bug and must be caught.
            let needed = base + keep_top;
            if value_stack.len() < needed {
                if matches!(frame.kind, ControlKind::Function) {
                    bail!(
                        "branch requires {} values but current stack has {} (function return)",
                        keep_top,
                        value_stack.len()
                    );
                }
                while value_stack.len() < needed {
                    script.push(op::PUSH0);
                    value_stack.push(StackValue::unknown());
                }
            }
            // Remove the K buried values located just beneath the top
            // `keep_top` operands, keeping the operands in place. On NeoVM,
            // removing the item at depth N (0-based from the top) is
            // `emit_push_int(N); XDROP`. Each removal shifts the next buried
            // value down to depth N, so repeat K times.
            let discard = value_stack.len() - needed;
            for _ in 0..discard {
                let _ = emit_push_int(script, keep_top as i128);
                script.push(op::XDROP);
                // Remove the topmost buried entry from the abstract stack
                // (the item just beneath the preserved top `keep_top`).
                value_stack.remove(value_stack.len() - 1 - keep_top);
            }
        }

        let opcode = "JMP_L";
        let pos = emit_jump_placeholder(script, opcode)?;
        frame.end_fixups.push(pos);

        // Truncate the abstract stack to base + keep_top so block/function
        // results are preserved. (value_stack is already at this height when the
        // discard loop ran; this also covers the unreachable path.)
        value_stack.truncate(base + keep_top);
        // Unconditional branch makes subsequent code unreachable.
        *is_unreachable = true;
        return Ok(());
    }

    // Conditional `br_if`. The condition has already been popped from the
    // abstract `value_stack` by the caller, but is still on top of the concrete
    // NeoVM stack and is consumed by the conditional jump.
    //
    // WASM only discards the buried values on the path that ACTUALLY branches;
    // the fall-through must retain the full operand stack (minus the condition).
    // Structure the emission so the operand-preserving discard runs only when
    // the branch is taken:
    //
    //     JMPIFNOT_L skip      ; consume condition; not-taken -> fall through
    //     <discard buried, keep top N>
    //     JMP_L target         ; taken -> jump to the frame's end label
    //   skip:                  ; fall-through continues here, full stack intact
    if *is_unreachable {
        // In unreachable code we still must consume the condition with a jump to
        // keep the concrete stack model consistent; emit a plain conditional jump
        // to the end label and leave the abstract stack untouched.
        let pos = emit_jump_placeholder(script, "JMPIF_L")?;
        frame.end_fixups.push(pos);
        return Ok(());
    }

    let needed = base + keep_top;

    // Function returns remain strict: a conditional return missing its result
    // value is a genuine correctness bug and must be caught (matching the
    // unconditional path and the function epilogue). Block/If breaks may be
    // padded for wasm-opt code motion, but Function frames may not.
    if matches!(frame.kind, ControlKind::Function) && value_stack.len() < needed {
        bail!(
            "branch requires {} values but current stack has {} (function return)",
            keep_top,
            value_stack.len()
        );
    }

    // Number of buried values to discard on the taken path.
    let discard = value_stack.len().saturating_sub(needed);
    let pad = needed.saturating_sub(value_stack.len());

    if discard == 0 && pad == 0 {
        // No buried values to remove and no padding required: a plain conditional
        // jump to the end label suffices, and the fall-through keeps the operands.
        let pos = emit_jump_placeholder(script, "JMPIF_L")?;
        frame.end_fixups.push(pos);
        // Do NOT mutate value_stack: the fall-through retains all operands.
        return Ok(());
    }

    // Taken path needs extra work (discard buried values and/or pad operands).
    // Skip it on the not-taken (fall-through) path.
    let skip_fixup = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    // --- taken path ---
    // Pad missing top operands (wasm-opt edge case), then discard buried values.
    for _ in 0..pad {
        script.push(op::PUSH0);
    }
    for _ in 0..discard {
        let _ = emit_push_int(script, keep_top as i128);
        script.push(op::XDROP);
    }
    let target_fixup = emit_jump_placeholder(script, "JMP_L")?;
    frame.end_fixups.push(target_fixup);

    // --- skip label (fall-through) ---
    let skip_label = script.len();
    patch_jump(script, skip_fixup, skip_label)?;

    // The fall-through retains the full operand stack (minus the popped
    // condition, which the caller already removed). Do NOT truncate value_stack.
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
        script.push(op::DROP);
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
    let dup = op::DUP;
    // NUMEQUAL, not type-strict EQUAL: the br_table selector is a wasm i32 that
    // may be a NeoVM Boolean (e.g. `match (a < b) as u32 { .. }`). EQUAL would
    // compare Boolean(0/1) against the Integer case labels by type and never
    // match; NUMEQUAL coerces both to BigInteger. Selector is always numeric,
    // so this is safe. (Same class as the eqz/eq/ne type-strict-EQUAL fix.)
    let equal = op::NUMEQUAL;
    let drop = op::DROP;

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
