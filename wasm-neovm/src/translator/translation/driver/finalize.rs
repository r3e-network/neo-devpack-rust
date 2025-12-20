use super::*;

use import_stub::emit_import_export_stub;
use start::{append_start_stub, resolve_start_descriptor};

use super::state::DriverState;

impl<'a> DriverState<'a> {
    pub(super) fn finalize(mut self) -> Result<Translation> {
        if !self.saw_code_section && self.import_export_indices.is_empty() {
            bail!("input module does not contain a code section");
        }

        let adapter = self.adapter.as_ref();

        for &import_idx in &self.import_export_indices {
            let Some(entry) = self.exported_funcs.get_mut(&(import_idx as u32)) else {
                continue;
            };
            if !entry.names.iter().any(|alias| !alias.processed) {
                continue;
            }

            let import = self.frontend.imports().get(import_idx).ok_or_else(|| {
                anyhow!(
                    "export references missing import function index {}",
                    import_idx
                )
            })?;
            let type_index = get_import_type_index(import)?;
            let func_type = self
                .frontend
                .module_types()
                .signature(type_index as usize)
                .ok_or_else(|| {
                    anyhow!(
                        "invalid type index {} for import {}::{}",
                        type_index,
                        import.module,
                        import.name
                    )
                })?;

            if func_type.results().len() > 1 {
                bail!(
                    "multi-value returns are not supported for exported import {}::{}",
                    import.module,
                    import.name
                );
            }

            let parameter_defs: Vec<ManifestParameter> = func_type
                .params()
                .iter()
                .enumerate()
                .map(|(idx, param)| ManifestParameter {
                    name: format!("arg{}", idx),
                    kind: wasm_val_type_to_manifest(param).unwrap_or_else(|_| "Any".to_string()),
                })
                .collect();

            let offset = self.script.len();
            let return_kind = emit_import_export_stub(
                &mut self.script,
                &mut self.runtime,
                self.frontend.imports(),
                self.frontend.module_types().signatures(),
                import_idx,
                &mut self.feature_tracker,
                adapter,
            )
            .with_context(|| {
                format!(
                    "failed to synthesise export stub for import {}::{}",
                    import.module, import.name
                )
            })?;

            for alias in entry.names.iter_mut() {
                if alias.processed {
                    continue;
                }
                self.methods.push(ManifestMethod {
                    name: alias.name.clone(),
                    parameters: parameter_defs.clone(),
                    return_type: return_kind.clone(),
                    offset: offset as u32,
                    safe: false,
                });
                self.feature_tracker.register_export(&alias.name);
                alias.processed = true;
            }
        }

        let mut missing: Vec<String> = self
            .exported_funcs
            .values()
            .flat_map(|entry| {
                entry
                    .names
                    .iter()
                    .filter(|alias| !alias.processed)
                    .map(|alias| alias.name.clone())
            })
            .collect();
        missing.sort_unstable();

        if !missing.is_empty() {
            bail!(
                "did not translate exported functions: {}",
                missing.join(", ")
            );
        }

        if self.methods.is_empty() {
            bail!(
                "no exportable functions were translated – ensure functions are exported and meet translation constraints"
            );
        }

        for method in &mut self.methods {
            if self.overlay_safe_methods.remove(&method.name) {
                method.safe = true;
            }
        }
        if !self.overlay_safe_methods.is_empty() {
            let mut missing: Vec<String> = self.overlay_safe_methods.into_iter().collect();
            missing.sort_unstable();
            bail!(
                "manifest overlays marked the following methods safe but they were not exported: {}",
                missing.join(", ")
            );
        }

        let start_descriptor = resolve_start_descriptor(
            self.start_function,
            self.start_defined_offset,
            &self.frontend,
            &mut self.feature_tracker,
            adapter,
        )?;

        self.runtime.finalize(
            &mut self.script,
            start_descriptor.as_ref(),
            self.frontend.imports(),
            self.frontend.module_types().signatures(),
            adapter,
        )?;

        if let Some(descriptor) = &start_descriptor {
            if let (Some(init_offset), Some(start_slot)) =
                (self.runtime.memory_init_offset(), self.runtime.start_slot())
            {
                let start_names: Vec<String> = self
                    .exported_funcs
                    .get(&descriptor.function_index)
                    .map(|entry| entry.names.iter().map(|alias| alias.name.clone()).collect())
                    .unwrap_or_default();

                if !start_names.is_empty() {
                    let start_body_offset = match descriptor.kind {
                        StartKind::Defined { offset } => Some(offset),
                        StartKind::Import => None,
                    };
                    let stub_offset = append_start_stub(
                        &mut self.script,
                        init_offset,
                        start_body_offset,
                        start_slot,
                    )?;
                    self.runtime
                        .patch_start_calls(&mut self.script, stub_offset)?;
                    for method in &mut self.methods {
                        if start_names.contains(&method.name) {
                            method.offset = stub_offset as u32;
                        }
                    }
                }
            }
        }

        let mut manifest_builder = ManifestBuilder::new(self.contract_name, &self.methods);
        if let Some(overlay) = self.manifest_overlay {
            manifest_builder
                .merge_overlay(&overlay, Some("embedded neo.manifest sections".to_string()));
        }
        if let Some(extra) = self.extra_manifest_overlay {
            manifest_builder.merge_overlay(&extra.value, extra.label);
        }
        self.feature_tracker.apply(&mut manifest_builder);
        manifest_builder.propagate_safe_flags();
        manifest_builder.ensure_method_parity()?;

        let mut metadata = extract_nef_metadata(manifest_builder.manifest_value())?;
        metadata.method_tokens.extend(self.section_method_tokens);
        metadata
            .method_tokens
            .extend(infer_contract_tokens(&self.script));
        dedup_method_tokens(&mut metadata.method_tokens);
        if metadata.source.is_none() {
            metadata.source = self.section_source;
        }

        update_manifest_metadata(
            manifest_builder.manifest_value_mut(),
            metadata.source.as_deref(),
            &metadata.method_tokens,
        )?;

        validate_script(&self.script).context("generated NeoVM script failed validation")?;

        Ok(Translation {
            script: self.script,
            manifest: manifest_builder.into_rendered(),
            method_tokens: metadata.method_tokens.clone(),
            source_url: metadata.source.clone(),
        })
    }
}
