// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, bail, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Collect all method names from a manifest's ABI section.
pub fn collect_method_names(manifest: &Value) -> HashSet<String> {
    manifest
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(Value::as_array)
        .map(|methods| {
            methods
                .iter()
                .filter_map(|method| method.get("name").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn collect_method_shapes(manifest: &Value) -> Result<HashMap<String, MethodShape>> {
    let mut shapes = HashMap::new();
    let Some(methods) = manifest
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(Value::as_array)
    else {
        return Ok(shapes);
    };

    for method in methods {
        let name = method
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("manifest method missing name"))?
            .to_string();

        let params = method
            .get("parameters")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("manifest method '{}' missing parameters array", name))?;
        let mut param_types = Vec::with_capacity(params.len());
        for param in params {
            let ty = param
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("manifest method '{}' parameter missing type", name))?;
            param_types.push(ty.to_string());
        }

        let return_type = method
            .get("returntype")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("manifest method '{}' missing returntype", name))?;
        let offset = method
            .get("offset")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow!("manifest method '{}' missing offset", name))?;
        let offset_u32 = u32::try_from(offset)
            .map_err(|_| anyhow!("manifest method '{}' offset exceeds u32 range", name))?;

        shapes.insert(
            name,
            MethodShape {
                param_types,
                return_type: return_type.to_string(),
                offset: offset_u32,
            },
        );
    }

    Ok(shapes)
}

/// Ensure the final manifest methods match the baseline set, detecting additions or removals.
pub fn ensure_manifest_methods_match(
    manifest: &Value,
    baseline: &HashSet<String>,
    overlay_label: Option<&str>,
) -> Result<()> {
    let final_names = collect_method_names(manifest);
    let mut introduced: Vec<String> = final_names.difference(baseline).cloned().collect();
    let mut missing: Vec<String> = baseline.difference(&final_names).cloned().collect();
    introduced.sort_unstable();
    missing.sort_unstable();

    if !introduced.is_empty() || !missing.is_empty() {
        let hint = overlay_label
            .map(|label| format!(" ({label})"))
            .unwrap_or_default();
        if !introduced.is_empty() {
            bail!(
                "manifest overlay{} introduced ABI methods that do not match the translated exports: {}",
                hint,
                introduced.join(", ")
            );
        }
        bail!(
            "manifest overlay{} removed exported ABI methods: {}",
            hint,
            missing.join(", ")
        );
    }

    Ok(())
}

/// Collect method names flagged `safe: true` anywhere inside an overlay
/// fragment. Overlays are manifest-shaped, but the ABI may be nested inside
/// wrapper objects/arrays, so this recurses the same way the custom-section
/// parser does.
pub(super) fn collect_safe_method_names(value: &Value, accumulator: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(methods) = map
                .get("abi")
                .and_then(Value::as_object)
                .and_then(|abi| abi.get("methods"))
                .and_then(Value::as_array)
            {
                for method in methods {
                    if method.get("safe").and_then(Value::as_bool).unwrap_or(false) {
                        if let Some(name) = method.get("name").and_then(Value::as_str) {
                            accumulator.insert(name.to_string());
                        }
                    }
                }
            }
            for child in map.values() {
                collect_safe_method_names(child, accumulator);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_safe_method_names(item, accumulator);
            }
        }
        _ => {}
    }
}

/// Ensure every method name an overlay marks `safe` refers to an exported
/// method, suggesting near-misses so a typo in a safe-list fails translation
/// with an actionable message instead of a generic parity error.
pub(super) fn ensure_safe_methods_exist(
    safe_names: &HashSet<String>,
    exported: &HashSet<String>,
    overlay_label: Option<&str>,
) -> Result<()> {
    let mut unknown: Vec<&str> = safe_names
        .iter()
        .filter(|name| !exported.contains(*name))
        .map(String::as_str)
        .collect();
    if unknown.is_empty() {
        return Ok(());
    }
    unknown.sort_unstable();

    let hint = overlay_label
        .map(|label| format!(" ({label})"))
        .unwrap_or_default();
    let details: Vec<String> = unknown
        .iter()
        .map(|name| match closest_method(name, exported) {
            Some(candidate) => format!("{name} (did you mean '{candidate}'?)"),
            None => (*name).to_string(),
        })
        .collect();
    bail!(
        "manifest overlay{} marked methods safe that are not exported: {}",
        hint,
        details.join(", ")
    );
}

/// Find the exported method name closest to `name` (edit distance <= 2).
fn closest_method<'a>(name: &str, exported: &'a HashSet<String>) -> Option<&'a str> {
    exported
        .iter()
        .map(|candidate| (levenshtein(name, candidate), candidate.as_str()))
        .filter(|(distance, _)| *distance <= 2)
        // Tie-break on the name itself so the suggestion is deterministic.
        .min_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)))
        .map(|(_, candidate)| candidate)
}

/// Classic two-row Levenshtein edit distance.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            current[j + 1] = (prev[j + 1] + 1).min(current[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut current);
    }
    prev[b.len()]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MethodShape {
    pub(super) param_types: Vec<String>,
    pub(super) return_type: String,
    pub(super) offset: u32,
}
