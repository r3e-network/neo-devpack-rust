// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, Result};
use serde_json::{json, Map, Value};

use super::{method_tokens_to_json, SOURCE_EXTRA_KEY, TOKEN_COLLECTION_KEY};
use crate::nef::MethodToken;

/// Update a manifest JSON with source and method token metadata.
pub fn update_manifest_metadata(
    manifest: &mut Value,
    source: Option<&str>,
    tokens: &[MethodToken],
) -> Result<()> {
    let manifest_obj = manifest
        .as_object_mut()
        .ok_or_else(|| anyhow!("manifest JSON must be an object"))?;

    if let Some(src) = source {
        let extra = ensure_extra_object(manifest_obj)?;
        extra.insert(SOURCE_EXTRA_KEY.to_string(), Value::String(src.to_string()));
    }

    if !tokens.is_empty() {
        let extra = ensure_extra_object(manifest_obj)?;
        extra.insert(
            TOKEN_COLLECTION_KEY.to_string(),
            method_tokens_to_json(tokens),
        );
    }

    Ok(())
}

fn ensure_extra_object(manifest_obj: &mut Map<String, Value>) -> Result<&mut Map<String, Value>> {
    if !manifest_obj.contains_key("extra") {
        manifest_obj.insert("extra".to_string(), json!({}));
    }
    manifest_obj
        .get_mut("extra")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow!("manifest 'extra' field must be a JSON object"))
}
