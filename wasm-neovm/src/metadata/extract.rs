// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};
use serde_json::Value;

use super::{NefMetadata, SOURCE_EXTRA_KEY, SOURCE_TOP_LEVEL_KEY, TOKEN_COLLECTION_KEY};
use crate::nef::MethodToken;

/// Extract NEF metadata (source URL and method tokens) from a manifest JSON value.
pub fn extract_nef_metadata(manifest: &Value) -> Result<NefMetadata> {
    let source = extract_source(manifest);
    let method_tokens = extract_method_tokens(manifest)?;
    Ok(NefMetadata {
        source,
        method_tokens,
    })
}

fn extract_source(manifest: &Value) -> Option<String> {
    manifest
        .get(SOURCE_TOP_LEVEL_KEY)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            manifest
                .get("extra")
                .and_then(Value::as_object)
                .and_then(|extra| extra.get(SOURCE_EXTRA_KEY))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn extract_method_tokens(manifest: &Value) -> Result<Vec<MethodToken>> {
    let extra = match manifest.get("extra") {
        Some(Value::Object(obj)) => obj,
        Some(_) => bail!("manifest 'extra' field must be an object when present"),
        None => return Ok(Vec::new()),
    };

    let raw_tokens = match extra.get(TOKEN_COLLECTION_KEY) {
        Some(value) => value,
        None => return Ok(Vec::new()),
    };

    super::parse::parse_method_tokens_value(raw_tokens, "manifest extra")
}
