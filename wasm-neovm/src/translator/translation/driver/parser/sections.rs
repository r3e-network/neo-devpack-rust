use super::super::exports::ExportAlias;
use super::super::*;

impl DriverState {
    pub(super) fn handle_type_section(
        &mut self,
        reader: wasmparser::TypeSectionReader<'_>,
    ) -> Result<()> {
        for group in reader {
            let group = group?;
            for (_, subtype) in group.into_types_and_offsets() {
                if let CompositeInnerType::Func(func) = subtype.composite_type.inner {
                    self.frontend.register_signature(func);
                }
            }
        }

        Ok(())
    }

    pub(super) fn handle_import_section(
        &mut self,
        reader: wasmparser::ImportSectionReader<'_>,
    ) -> Result<()> {
        for import in reader {
            let import = import?;
            match import.ty {
                TypeRef::Func(type_index) => {
                    self.frontend
                        .register_import(import.module, import.name, type_index);
                }
                TypeRef::Global(_) => {
                    bail!(
                        "global imports are not supported ({}::{})",
                        import.module,
                        import.name
                    );
                }
                _ => {
                    bail!(
                        "only function imports are supported (found non-function import {})",
                        import.name
                    );
                }
            }
        }

        Ok(())
    }

    pub(super) fn handle_function_section(
        &mut self,
        reader: wasmparser::FunctionSectionReader<'_>,
    ) -> Result<()> {
        for idx in reader {
            self.frontend.register_defined_function(idx?);
        }
        Ok(())
    }

    pub(super) fn handle_table_section(
        &mut self,
        reader: wasmparser::TableSectionReader<'_>,
    ) -> Result<()> {
        for table in reader {
            let table = table?;
            if table.ty.table64 {
                bail!("table64 is not supported");
            }
            if table.ty.shared {
                bail!("shared tables are not supported");
            }
            if table.ty.element_type != RefType::FUNCREF {
                bail!(
                    "reference type {:?} tables are not supported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                    table.ty.element_type
                );
            }
            let initial_len = usize::try_from(table.ty.initial)
                .context("table initial size exceeds host limits")?;
            if initial_len as u32 > self.behavior.max_table_size {
                bail!(
                    "table initial size {} exceeds configured maximum {}",
                    initial_len,
                    self.behavior.max_table_size
                );
            }
            let maximum = match table.ty.maximum {
                Some(max) => {
                    let max_u32 =
                        u32::try_from(max).context("table maximum exceeds 32-bit range")?;
                    if max_u32 > self.behavior.max_table_size {
                        bail!(
                            "table maximum size {} exceeds configured maximum {}",
                            max_u32,
                            self.behavior.max_table_size
                        );
                    }
                    Some(max_u32)
                }
                None => None,
            };
            self.runtime.register_table(initial_len, maximum);
            self.tables.push(TableInfo);
        }

        Ok(())
    }

    pub(super) fn handle_global_section(
        &mut self,
        reader: wasmparser::GlobalSectionReader<'_>,
    ) -> Result<()> {
        for entry in reader {
            let entry = entry?;
            let value_type = entry.ty.content_type;
            match value_type {
                ValType::I32 | ValType::I64 => {}
                other => bail!("only i32/i64 globals are supported (found {:?})", other),
            }
            let initial = evaluate_global_init(entry.init_expr, value_type)
                .context("failed to evaluate global initialiser")?;
            self.runtime.register_global(entry.ty.mutable, initial);
        }

        Ok(())
    }

    pub(super) fn handle_export_section(
        &mut self,
        reader: wasmparser::ExportSectionReader<'_>,
    ) -> Result<()> {
        for export in reader {
            let export = export?;
            if export.kind == ExternalKind::Func {
                let entry = self.exported_funcs.entry(export.index).or_default();
                entry.names.push(ExportAlias {
                    name: export.name.to_string(),
                    processed: false,
                });
                if (export.index as usize) < self.frontend.import_len() {
                    self.import_export_indices.insert(export.index as usize);
                }
            }
        }

        Ok(())
    }

    pub(super) fn handle_memory_section(
        &mut self,
        reader: wasmparser::MemorySectionReader<'_>,
    ) -> Result<()> {
        for mem in reader {
            let mem = mem?;
            if mem.memory64 {
                bail!("memory64 is not supported");
            }
            if mem.shared {
                bail!("shared memories are not supported");
            }
            let initial =
                u32::try_from(mem.initial).context("memory initial size exceeds 32-bit range")?;
            if initial > self.behavior.max_memory_pages {
                bail!(
                    "memory initial pages {} exceeds configured maximum {}",
                    initial,
                    self.behavior.max_memory_pages
                );
            }
            let maximum = match mem.maximum {
                Some(max) => {
                    let max_u32 =
                        u32::try_from(max).context("memory maximum exceeds 32-bit range")?;
                    if max_u32 > self.behavior.max_memory_pages {
                        bail!(
                            "memory maximum pages {} exceeds configured maximum {}",
                            max_u32,
                            self.behavior.max_memory_pages
                        );
                    }
                    Some(max_u32)
                }
                None => None,
            };
            self.runtime
                .set_memory_config(initial, maximum)
                .context("failed to register memory section")?;
        }

        Ok(())
    }
}
