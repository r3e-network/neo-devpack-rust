use super::super::*;

impl<'a> DriverState<'a> {
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
                    let table_idx = table_index.unwrap_or(0) as usize;
                    let offset_raw = evaluate_offset_expr(offset_expr)
                        .context("failed to evaluate element offset")?;
                    let offset = (offset_raw as u32) as usize;
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
                .map(|opt| opt.map(|v| v as i32).unwrap_or(FUNCREF_NULL as i32))
                .collect();

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
