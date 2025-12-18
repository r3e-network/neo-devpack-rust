use super::super::super::*;

impl RuntimeHelpers {
    pub(super) fn emit_passive_data_helpers(&mut self, script: &mut Vec<u8>) -> Result<()> {
        for (index, segment) in self.data_segments.iter_mut().enumerate() {
            if let DataSegmentKind::Passive {
                init_record,
                drop_record,
                byte_slot,
                drop_slot,
            } = &mut segment.kind
            {
                let byte_slot = byte_slot
                    .ok_or_else(|| anyhow!("passive segment {} missing byte slot", index))?;
                let drop_slot = drop_slot
                    .ok_or_else(|| anyhow!("passive segment {} missing drop slot", index))?;

                if !init_record.calls.is_empty() {
                    let helper_offset = match init_record.offset {
                        Some(existing) => existing,
                        None => {
                            let helper_offset = script.len();
                            emit_data_init_helper(
                                script,
                                byte_slot,
                                drop_slot,
                                segment.bytes.len(),
                            )?;
                            init_record.offset = Some(helper_offset);
                            helper_offset
                        }
                    };

                    for &call_pos in &init_record.calls {
                        patch_call(script, call_pos, helper_offset)?;
                    }
                }

                if !drop_record.calls.is_empty() {
                    let helper_offset = match drop_record.offset {
                        Some(existing) => existing,
                        None => {
                            let helper_offset = script.len();
                            emit_data_drop_helper(script, drop_slot)?;
                            drop_record.offset = Some(helper_offset);
                            helper_offset
                        }
                    };

                    for &call_pos in &drop_record.calls {
                        patch_call(script, call_pos, helper_offset)?;
                    }
                }
            }
        }

        Ok(())
    }
}
