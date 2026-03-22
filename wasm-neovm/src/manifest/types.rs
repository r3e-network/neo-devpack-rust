// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use serde::Serialize;

/// A parameter in a Neo N3 contract manifest method.
#[derive(Debug, Clone, Serialize)]
pub struct ManifestParameter {
    /// Parameter name.
    pub name: String,
    #[serde(rename = "type")]
    /// Parameter type (e.g. `"Hash160"`, `"Integer"`).
    pub kind: String,
}

/// A method entry in a Neo N3 contract manifest ABI.
#[derive(Debug, Clone, Serialize)]
pub struct ManifestMethod {
    /// Method name.
    pub name: String,
    /// Method parameters.
    pub parameters: Vec<ManifestParameter>,
    #[serde(rename = "returntype")]
    /// Return type (e.g. `"Void"`, `"Boolean"`).
    pub return_type: String,
    /// Bytecode offset of the method entry point.
    pub offset: u32,
    /// Whether this method is safe (read-only).
    pub safe: bool,
}

/// A fully rendered Neo N3 contract manifest.
#[derive(Debug, Clone)]
pub struct RenderedManifest {
    /// The underlying JSON value.
    pub value: serde_json::Value,
}

impl RenderedManifest {
    /// Serialize the manifest to a pretty-printed JSON string.
    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.value)
    }
}
