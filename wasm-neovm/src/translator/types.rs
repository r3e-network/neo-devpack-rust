use crate::adapters::SourceChain;
use crate::manifest::{ManifestMethod, RenderedManifest};
use crate::nef::MethodToken;
use serde_json::Value;

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

#[derive(Debug)]
pub struct TranslationConfig<'a> {
    pub contract_name: &'a str,
    pub extra_manifest_overlay: Option<ManifestOverlay>,
    pub source_chain: SourceChain,
}

impl<'a> TranslationConfig<'a> {
    pub fn new(contract_name: &'a str) -> Self {
        Self {
            contract_name,
            extra_manifest_overlay: None,
            source_chain: SourceChain::Neo,
        }
    }

    pub fn with_manifest_overlay(mut self, overlay: ManifestOverlay) -> Self {
        self.extra_manifest_overlay = Some(overlay);
        self
    }

    pub fn with_source_chain(mut self, source_chain: SourceChain) -> Self {
        self.source_chain = source_chain;
        self
    }
}

#[derive(Debug)]
pub struct ManifestOverlay {
    pub value: Value,
    pub label: Option<String>,
}
