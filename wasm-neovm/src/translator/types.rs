use crate::manifest::{ManifestMethod, RenderedManifest};
use crate::nef::MethodToken;

#[derive(Debug, Clone)]
pub(crate) struct StackValue {
    pub(crate) const_value: Option<i128>,
    pub(crate) bytecode_start: Option<usize>,
}

#[derive(Debug)]
pub struct Translation {
    pub script: Vec<u8>,
    pub method_tokens: Vec<MethodToken>,
    pub manifest: RenderedManifest,
    pub source_url: Option<String>,
}

#[derive(Debug)]
pub struct ManifestData {
    pub methods: Vec<ManifestMethod>,
}

#[derive(Debug, Clone)]
pub struct FunctionImport {
    pub module: String,
    pub name: String,
    pub type_index: u32,
}
