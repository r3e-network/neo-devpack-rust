use super::super::*;

impl DriverState {
    pub(super) fn handle_data_section(
        &mut self,
        reader: wasmparser::DataSectionReader<'_>,
    ) -> Result<()> {
        for entry in reader {
            let entry = entry?;
            let data_bytes = entry.data.to_vec();
            match entry.kind {
                DataKind::Passive => {
                    self.runtime
                        .register_passive_segment(data_bytes)
                        .context("failed to register passive data segment")?;
                }
                DataKind::Active {
                    memory_index,
                    offset_expr,
                } => {
                    if memory_index != 0 {
                        bail!("only default memory index 0 is supported for active data segments");
                    }
                    let offset_raw = evaluate_offset_expr(offset_expr)
                        .context("failed to evaluate active data segment offset")?;
                    let offset = u64::try_from(offset_raw)
                        .map_err(|_| anyhow!("active data segment offset must be non-negative"))?;
                    self.runtime
                        .register_active_segment(memory_index, offset, data_bytes)
                        .context("failed to register active data segment")?;
                }
            }
        }

        Ok(())
    }
}
