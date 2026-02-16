//! Manifest builder for Neo N3 contract manifests
//!
//! This module provides a builder pattern for constructing and modifying
//! Neo N3 contract manifests. It handles:
//! - Initial manifest generation from exported methods
//! - Overlay merging for additional metadata
//! - Safe flag propagation for view methods
//! - Method parity checking between baseline and final manifests

use anyhow::{bail, Result};
use log::warn;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

use super::parity::{collect_method_shapes, MethodShape};
use super::{
    build_manifest, collect_method_names, ensure_manifest_methods_match, merge_manifest,
    propagate_safe_flags, ManifestMethod,
};

/// Builder for Neo N3 contract manifests.
///
/// The builder maintains the baseline manifest (generated from Wasm exports)
/// and applies overlays while ensuring ABI consistency.
#[derive(Debug)]
pub struct ManifestBuilder {
    manifest: Value,
    baseline_methods: HashSet<String>,
    baseline_signatures: HashMap<String, MethodShape>,
    overlay_label: Option<String>,
}

impl ManifestBuilder {
    /// Creates a new manifest builder from the contract name and exported methods.
    ///
    /// # Arguments
    /// * `name` - The contract name
    /// * `methods` - Slice of manifest methods exported from the Wasm module
    pub fn new(name: &str, methods: &[ManifestMethod]) -> Self {
        let manifest = build_manifest(name, methods).value;
        let baseline_signatures = match collect_method_shapes(&manifest) {
            Ok(signatures) => signatures,
            Err(e) => {
                warn!("translator-generated manifest has invalid methods: {}", e);
                std::collections::HashMap::new()
            }
        };
        ManifestBuilder {
            baseline_methods: collect_method_names(&manifest),
            baseline_signatures,
            manifest,
            overlay_label: None,
        }
    }

    pub fn merge_overlay(&mut self, overlay: &Value, label: Option<String>) {
        merge_manifest(&mut self.manifest, overlay);
        if let Some(label) = label {
            self.overlay_label = Some(label);
        }
    }

    pub fn propagate_safe_flags(&mut self) {
        propagate_safe_flags(&mut self.manifest);
    }

    pub fn ensure_method_parity(&self) -> Result<()> {
        ensure_manifest_methods_match(
            &self.manifest,
            &self.baseline_methods,
            self.overlay_label.as_deref(),
        )?;

        let final_shapes = collect_method_shapes(&self.manifest)?;
        let mut mutated: Vec<String> = Vec::new();
        for (name, baseline) in &self.baseline_signatures {
            if let Some(final_shape) = final_shapes.get(name) {
                let baseline_params = baseline.param_types.len();
                let final_params = final_shape.param_types.len();
                let baseline_void = baseline.return_type.eq_ignore_ascii_case("Void");
                let final_void = final_shape.return_type.eq_ignore_ascii_case("Void");

                if baseline_params != final_params
                    || baseline_void != final_void
                    || baseline.offset != final_shape.offset
                {
                    mutated.push(name.clone());
                }
            }
        }
        if !mutated.is_empty() {
            mutated.sort_unstable();
            let hint = self
                .overlay_label
                .as_deref()
                .map(|label| format!(" ({label})"))
                .unwrap_or_default();
            bail!(
                "manifest overlay{} mutated ABI arity or offsets for existing methods: {}",
                hint,
                mutated.join(", ")
            );
        }

        Ok(())
    }

    pub fn enable_feature(&mut self, feature: &str) -> Result<()> {
        let manifest_obj = self
            .manifest
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("manifest root must be an object"))?;
        let entry = manifest_obj
            .entry("features".to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        if let Some(map) = entry.as_object_mut() {
            map.insert(feature.to_string(), Value::Bool(true));
        }
        Ok(())
    }

    pub fn manifest_value(&self) -> &Value {
        &self.manifest
    }

    pub fn manifest_value_mut(&mut self) -> &mut Value {
        &mut self.manifest
    }

    pub fn into_rendered(self) -> super::RenderedManifest {
        let mut manifest = self.manifest;
        if let Some(features) = manifest
            .get_mut("features")
            .and_then(serde_json::Value::as_object_mut)
        {
            features.clear();
        }

        super::RenderedManifest { value: manifest }
    }
}
