// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

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

    /// Merge an overlay JSON value into the manifest.
    pub fn merge_overlay(&mut self, overlay: &Value, label: Option<String>) {
        merge_manifest(&mut self.manifest, overlay);
        if let Some(label) = label {
            self.overlay_label = Some(label);
        }
    }

    /// Propagate safe flags from overlay into matching ABI methods.
    pub fn propagate_safe_flags(&mut self) {
        propagate_safe_flags(&mut self.manifest);
    }

    /// Verify that the final manifest has the same methods as the baseline.
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
                if baseline != final_shape {
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
                "manifest overlay{} mutated ABI signatures or offsets for existing methods: {}",
                hint,
                mutated.join(", ")
            );
        }

        Ok(())
    }

    /// Enable a feature flag in the manifest.
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

    /// Return a shared reference to the underlying manifest JSON.
    pub fn manifest_value(&self) -> &Value {
        &self.manifest
    }

    /// Return a mutable reference to the underlying manifest JSON.
    pub fn manifest_value_mut(&mut self) -> &mut Value {
        &mut self.manifest
    }

    /// Consume the builder and return a `RenderedManifest`.
    ///
    /// The translator tracks declared `features.storage`/`features.payable`
    /// internally, but the on-disk manifest is rendered with `features: {}`
    /// because Neo Express's deployer rejects manifests that carry non-empty
    /// feature maps; the actual storage capability still flows through the
    /// emitted `System.Storage.*` SYSCALLs in the script body.
    pub fn into_rendered(self) -> super::RenderedManifest {
        super::RenderedManifest {
            value: self.manifest,
        }
    }
}
