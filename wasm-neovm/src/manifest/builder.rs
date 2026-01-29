use anyhow::{bail, Result};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

use super::parity::{collect_method_shapes, MethodShape};
use super::{
    build_manifest, collect_method_names, ensure_manifest_methods_match, merge_manifest,
    propagate_safe_flags, ManifestMethod,
};

#[derive(Debug)]
pub struct ManifestBuilder {
    manifest: Value,
    baseline_methods: HashSet<String>,
    baseline_signatures: HashMap<String, MethodShape>,
    overlay_label: Option<String>,
}

impl ManifestBuilder {
    pub fn new(name: &str, methods: &[ManifestMethod]) -> Self {
        let manifest = build_manifest(name, methods).value;
        let baseline_signatures = collect_method_shapes(&manifest)
            .unwrap_or_else(|e| {
                // In debug builds, panic with details; in release, use empty map
                #[cfg(debug_assertions)]
                panic!("translator-generated manifest must contain well-formed methods: {}", e);
                #[cfg(not(debug_assertions))]
                {
                    eprintln!("Warning: translator-generated manifest has invalid methods: {}", e);
                    std::collections::HashMap::new()
                }
            });
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
        super::RenderedManifest {
            value: self.manifest,
        }
    }
}
