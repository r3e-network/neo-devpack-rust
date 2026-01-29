use super::{FunctionImport, ModuleTypes};
use wasmparser::FuncType;

/// Aggregates Wasm module metadata (types + imports) before translation.
#[derive(Default)]
pub struct ModuleFrontend {
    module_types: ModuleTypes,
    imports: Vec<FunctionImport>,
}

impl ModuleFrontend {
    pub fn new() -> Self {
        // Pre-allocate with typical capacities (Round 62 optimization)
        Self {
            module_types: ModuleTypes::default(),
            imports: Vec::with_capacity(32),
        }
    }

    pub fn register_signature(&mut self, func: FuncType) {
        self.module_types.register_signature(func);
    }

    pub fn register_defined_function(&mut self, type_index: u32) {
        self.module_types.push_defined_type_index(type_index);
    }

    pub fn register_import(&mut self, module: &str, name: &str, type_index: u32) {
        self.imports.push(FunctionImport {
            module: module.to_string(),
            name: name.to_string(),
            type_index,
        });
    }

    pub fn module_types(&self) -> &ModuleTypes {
        &self.module_types
    }

    pub fn imports(&self) -> &[FunctionImport] {
        &self.imports
    }

    pub fn import_len(&self) -> usize {
        self.imports.len()
    }
}
