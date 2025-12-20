use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    types: &[FuncType],
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut Vec<ControlFrame>,
    is_unreachable: &mut bool,
) -> Result<bool> {
    match op {
        Operator::Block { blockty: ty, .. } => {
            let result_count = block_result_count(*ty, types)?;
            control_stack.push(ControlFrame {
                kind: ControlKind::Block,
                stack_height: value_stack.len(),
                result_count,
                start_offset: script.len(),
                end_fixups: Vec::new(),
                if_false_fixup: None,
                has_else: false,
                entry_reachable: !*is_unreachable,
                end_reachable_from_branch: false,
                if_then_end_reachable: None,
            });
            Ok(true)
        }
        Operator::Loop { blockty: ty, .. } => {
            let result_count = block_result_count(*ty, types)?;
            control_stack.push(ControlFrame {
                kind: ControlKind::Loop,
                stack_height: value_stack.len(),
                result_count,
                start_offset: script.len(),
                end_fixups: Vec::new(),
                if_false_fixup: None,
                has_else: false,
                entry_reachable: !*is_unreachable,
                end_reachable_from_branch: false,
                if_then_end_reachable: None,
            });
            Ok(true)
        }
        Operator::If { blockty: ty, .. } => {
            let result_count = block_result_count(*ty, types)?;
            let _cond =
                super::pop_value_maybe_unreachable(value_stack, "if condition", *is_unreachable)?;
            // Condition already materialised on stack
            let jump_pos = emit_jump_placeholder(script, "JMPIFNOT_L")?;
            control_stack.push(ControlFrame {
                kind: ControlKind::If,
                stack_height: value_stack.len(),
                result_count,
                start_offset: script.len(),
                end_fixups: Vec::new(),
                if_false_fixup: Some(jump_pos),
                has_else: false,
                entry_reachable: !*is_unreachable,
                end_reachable_from_branch: false,
                if_then_end_reachable: None,
            });
            Ok(true)
        }
        Operator::Else => {
            let frame = control_stack
                .last_mut()
                .ok_or_else(|| anyhow!("ELSE without matching IF"))?;
            if !matches!(frame.kind, ControlKind::If) {
                bail!("ELSE can only appear within an IF block");
            }
            if let Some(pos) = frame.if_false_fixup.take() {
                let else_start = script.len();
                patch_jump(script, pos, else_start)?;
            }
            // Jump over else body when the THEN branch executes
            let jump_end = emit_jump_placeholder(script, "JMP_L")?;
            frame.end_fixups.push(jump_end);
            frame.has_else = true;
            frame.start_offset = script.len();
            if !*is_unreachable {
                let expected = frame.stack_height + frame.result_count;
                if value_stack.len() < expected {
                    bail!(
                        "if branch must leave {} value(s) on the stack (found {})",
                        frame.result_count,
                        value_stack.len().saturating_sub(frame.stack_height)
                    );
                }
            }
            let then_fallthrough_reachable = !*is_unreachable;
            frame.if_then_end_reachable =
                Some(then_fallthrough_reachable || frame.end_reachable_from_branch);

            super::set_stack_height_polymorphic(value_stack, frame.stack_height);

            // ELSE is reachable iff the IF itself was reachable.
            *is_unreachable = !frame.entry_reachable;
            Ok(true)
        }
        Operator::End => {
            let frame = control_stack
                .pop()
                .ok_or_else(|| anyhow!("END without matching block"))?;
            let end_label = script.len();

            match frame.kind {
                ControlKind::If => {
                    if let Some(pos) = frame.if_false_fixup {
                        patch_jump(script, pos, end_label)?;
                    }
                    if frame.result_count > 0 && !frame.has_else && frame.entry_reachable {
                        bail!("if with a result type requires an else branch");
                    }
                }
                ControlKind::Loop | ControlKind::Block => {}
                ControlKind::Function => {
                    // This is the final END of the function
                    if let Some(pos) = frame.if_false_fixup {
                        patch_jump(script, pos, end_label)?;
                    }
                }
            }
            for fixup in frame.end_fixups {
                patch_jump(script, fixup, end_label)?;
            }
            let target_height = frame.stack_height + frame.result_count;
            let fallthrough_reachable = !*is_unreachable;
            let end_reachable = match frame.kind {
                ControlKind::If => {
                    if frame.has_else {
                        let then_end = frame.if_then_end_reachable.unwrap_or(false);
                        then_end || fallthrough_reachable || frame.end_reachable_from_branch
                    } else {
                        // If without else: false-condition path reaches END if the IF was reachable.
                        fallthrough_reachable
                            || frame.entry_reachable
                            || frame.end_reachable_from_branch
                    }
                }
                ControlKind::Loop => fallthrough_reachable,
                ControlKind::Block | ControlKind::Function => {
                    fallthrough_reachable || frame.end_reachable_from_branch
                }
            };

            if !*is_unreachable && value_stack.len() < target_height {
                bail!(
                    "{:?} block expected at least {} value(s) on the stack at end but found {}",
                    frame.kind,
                    frame.result_count,
                    value_stack.len().saturating_sub(frame.stack_height)
                );
            }
            if *is_unreachable && end_reachable && value_stack.len() < target_height {
                super::set_stack_height_polymorphic(value_stack, target_height);
            } else {
                value_stack.truncate(target_height);
            }

            *is_unreachable = !end_reachable;
            Ok(true)
        }
        Operator::Br { relative_depth } => {
            handle_branch(
                script,
                value_stack,
                control_stack,
                *relative_depth as usize,
                false,
                is_unreachable,
            )?;
            Ok(true)
        }
        Operator::BrIf { relative_depth } => {
            let _cond = super::pop_value_maybe_unreachable(
                value_stack,
                "br_if condition",
                *is_unreachable,
            )?;
            handle_branch(
                script,
                value_stack,
                control_stack,
                *relative_depth as usize,
                true,
                is_unreachable,
            )?;
            Ok(true)
        }
        Operator::BrTable { targets } => {
            let index =
                super::pop_value_maybe_unreachable(value_stack, "br_table index", *is_unreachable)?;
            let mut target_depths: Vec<usize> = Vec::with_capacity(targets.len() as usize);
            for target in targets.targets() {
                target_depths.push(target? as usize);
            }
            let default_depth = targets.default() as usize;
            handle_br_table(
                script,
                value_stack,
                control_stack,
                index,
                &target_depths,
                default_depth,
                is_unreachable,
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
