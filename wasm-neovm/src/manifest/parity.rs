use anyhow::{anyhow, bail, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

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

        shapes.insert(
            name,
            MethodShape {
                param_types,
                return_type: return_type.to_string(),
                offset: offset as u32,
            },
        );
    }

    Ok(shapes)
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MethodShape {
    pub(super) param_types: Vec<String>,
    pub(super) return_type: String,
    pub(super) offset: u32,
}
