use wasmparser::FuncType;

/// Records function import metadata so the translator can bridge Wasm imports
/// to NeoVM stubs or syscalls.
#[derive(Debug, Clone)]
pub struct FunctionImport {
    pub module: String,
    pub name: String,
    pub type_index: u32,
}

/// Tracks Wasm function signatures and their type indices to decouple parsing
/// from translation logic.
#[derive(Debug)]
pub struct ModuleTypes {
    signatures: Vec<FuncType>,
    defined_function_types: Vec<u32>,
}

impl Default for ModuleTypes {
    fn default() -> Self {
        // Pre-allocate with typical capacities (Round 62 optimization)
        Self {
            signatures: Vec::with_capacity(64),
            defined_function_types: Vec::with_capacity(64),
        }
    }
}

impl ModuleTypes {
    pub fn register_signature(&mut self, func: FuncType) {
        self.signatures.push(func);
    }

    pub fn push_defined_type_index(&mut self, type_index: u32) {
        self.defined_function_types.push(type_index);
    }

    pub fn signature(&self, index: usize) -> Option<&FuncType> {
        self.signatures.get(index)
    }

    pub fn defined_type_index(&self, index: usize) -> Option<u32> {
        self.defined_function_types.get(index).copied()
    }

    pub fn signatures(&self) -> &[FuncType] {
        &self.signatures
    }

    pub fn defined_type_indices(&self) -> &[u32] {
        &self.defined_function_types
    }

    pub fn defined_functions_len(&self) -> usize {
        self.defined_function_types.len()
    }
}
