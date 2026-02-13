use super::super::*;

impl DriverState {
    pub(super) fn handle_element_section(
        &mut self,
        reader: wasmparser::ElementSectionReader<'_>,
    ) -> Result<()> {
        for element in reader {
            let element = element?;

            let (table_index_opt, offset_opt) = match element.kind {
                wasmparser::ElementKind::Active {
                    table_index,
                    offset_expr,
                } => {
                    let table_index = table_index.unwrap_or(0);
                    let table_idx = usize::try_from(table_index).map_err(|_| {
                        anyhow!("element table index {} exceeds usize range", table_index)
                    })?;
                    let offset_raw = evaluate_offset_expr(offset_expr)
                        .context("failed to evaluate element offset")?;
                    let offset = usize::try_from(offset_raw)
                        .map_err(|_| anyhow!("element segment offset must be non-negative"))?;
                    (Some(table_idx), Some(offset))
                }
                wasmparser::ElementKind::Passive => (None, None),
                wasmparser::ElementKind::Declared => {
                    bail!("declared element segments are not supported")
                }
            };

            let mut func_refs: Vec<Option<u32>> = Vec::new();
            match element.items {
                wasmparser::ElementItems::Functions(funcs) => {
                    for func in funcs {
                        func_refs.push(Some(func?));
                    }
                }
                wasmparser::ElementItems::Expressions(ref_ty, exprs) => {
                    if ref_ty != RefType::FUNCREF {
                        bail!(
                            "element expressions for reference type {:?} are not supported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                            ref_ty
                        );
                    }
                    for expr in exprs {
                        let expr = expr?;
                        let mut reader = expr.get_operators_reader();
                        let mut value: Option<Option<u32>> = None;
                        while !reader.eof() {
                            let op = reader.read()?;
                            match op {
                                Operator::RefNull { hty } => {
                                    if hty != HeapType::FUNC {
                                        bail!(
                                            "element expression uses unsupported heap type {:?}",
                                            hty
                                        );
                                    }
                                    value = Some(None);
                                }
                                Operator::RefFunc { function_index } => {
                                    value = Some(Some(function_index));
                                }
                                Operator::End => break,
                                other => {
                                    bail!(
                                        "unsupported instruction {:?} in element segment expression",
                                        other
                                    );
                                }
                            }
                        }
                        let parsed = value
                            .ok_or_else(|| anyhow!("element expression did not yield a value"))?;
                        func_refs.push(parsed);
                    }
                }
            }

            let values_i32: Vec<i32> = func_refs
                .iter()
                .map(|opt| match opt {
                    Some(value) => i32::try_from(*value).map_err(|_| {
                        anyhow!(
                            "function index {} exceeds i32 range for table representation",
                            value
                        )
                    }),
                    None => Ok(FUNCREF_NULL as i32),
                })
                .collect::<Result<Vec<_>>>()?;

            if let Some(table_index) = table_index_opt {
                let offset = offset_opt
                    .ok_or_else(|| anyhow!("active element missing offset expression"))?;
                if table_index >= self.tables.len() {
                    bail!(
                        "element segment references unknown table index {} ({} tables declared)",
                        table_index,
                        self.tables.len()
                    );
                }
                self.runtime
                    .register_active_element(table_index, offset, values_i32)
                    .context("failed to register active element segment")?;
            } else {
                self.runtime.register_passive_element(values_i32);
            }
        }

        Ok(())
    }
}
