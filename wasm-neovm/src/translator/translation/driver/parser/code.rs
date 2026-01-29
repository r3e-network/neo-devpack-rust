use super::super::super::function::{translate_function, TranslationContext};
use super::super::*;

impl DriverState {
    pub(super) fn handle_code_section_start(&mut self) -> Result<()> {
        self.saw_code_section = true;
        self.next_defined_index = 0;
        let total_functions =
            self.frontend.import_len() + self.frontend.module_types().defined_functions_len();
        self.function_registry = Some(FunctionRegistry::new(total_functions));
        Ok(())
    }

    pub(super) fn handle_code_section_entry(
        &mut self,
        body: wasmparser::FunctionBody<'_>,
    ) -> Result<()> {
        let functions = self
            .function_registry
            .as_mut()
            .ok_or_else(|| anyhow!("code section encountered without initialisation"))?;
        let defined_index = self.next_defined_index;
        self.next_defined_index += 1;

        let func_index = self.frontend.import_len() + defined_index;
        let func_index_u32 = func_index as u32;
        let maybe_export = self.exported_funcs.get_mut(&func_index_u32);

        let function_name_owned = maybe_export
            .as_ref()
            .and_then(|entry| entry.names.first().map(|alias| alias.name.clone()))
            .unwrap_or_else(|| format!("<internal:{}>", func_index));
        let function_name = function_name_owned.as_str();

        let type_index = self
            .frontend
            .module_types()
            .defined_type_index(defined_index)
            .ok_or_else(|| {
                anyhow!(
                    "no type index recorded for function '{}' (defined index {})",
                    function_name,
                    defined_index
                )
            })?;

        let func_type = self
            .frontend
            .module_types()
            .signature(type_index as usize)
            .ok_or_else(|| {
                anyhow!(
                    "type index {} referenced by function '{}' out of bounds",
                    type_index,
                    function_name
                )
            })?;

        let offset = self.script.len();
        functions
            .register_offset(&mut self.script, func_index, offset)
            .context("failed to register function offset")?;

        if self.start_function == Some(func_index as u32) {
            self.start_defined_offset = Some(offset);
        }

        let suppress_init = self.start_function == Some(func_index as u32);
        let was_suppressed = self.runtime.set_memory_init_suppressed(suppress_init);

        let mut ctx = TranslationContext {
            func_type,
            body: &body,
            script: &mut self.script,
            imports: self.frontend.imports(),
            types: self.frontend.module_types().signatures(),
            func_type_indices: self.frontend.module_types().defined_type_indices(),
            runtime: &mut self.runtime,
            tables: &self.tables,
            functions,
            function_index: func_index,
            start_function: self.start_function,
            function_name,
            features: &mut self.feature_tracker,
            adapter: self.adapter.as_ref(),
        };
        let translation_result = translate_function(&mut ctx);
        self.runtime.set_memory_init_suppressed(was_suppressed);
        let return_kind = translation_result
            .with_context(|| format!("failed to translate function '{}'", function_name))?;

        if let Some(entry) = maybe_export {
            let parameter_defs: Vec<ManifestParameter> = func_type
                .params()
                .iter()
                .enumerate()
                .map(|(idx, param)| ManifestParameter {
                    name: format!("arg{}", idx),
                    kind: wasm_val_type_to_manifest(param).unwrap_or_else(|_| "Any".to_string()),
                })
                .collect();

            for alias in entry.names.iter_mut() {
                let method = ManifestMethod {
                    name: alias.name.clone(),
                    parameters: parameter_defs.clone(),
                    return_type: return_kind.clone(),
                    offset: offset as u32,
                    safe: false,
                };
                self.methods.push(method);
                self.feature_tracker.register_export(&alias.name);
                alias.processed = true;
            }
        }

        Ok(())
    }
}
