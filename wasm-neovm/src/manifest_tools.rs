// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use log::{debug, info};
use serde_json::{Map, Value};

pub(crate) fn compare_manifest(reference_path: &Path, generated: &Value) -> Result<()> {
    let bytes = fs::read_to_string(reference_path)
        .with_context(|| format!("failed to read manifest {}", reference_path.display()))?;
    let reference: Value = serde_json::from_str(&bytes).with_context(|| {
        format!(
            "failed to parse manifest JSON from {}",
            reference_path.display()
        )
    })?;
    if &reference == generated {
        info!("Manifest matches {}", reference_path.display());
        return Ok(());
    }

    let expected = serde_json::to_string_pretty(&reference)?;
    let actual = serde_json::to_string_pretty(generated)?;
    info!("Manifest differs from {}:", reference_path.display());
    for diff in diff::lines(&expected, &actual) {
        use diff::Result::{Both, Left, Right};
        match diff {
            Left(line) => debug!("-{}", line),
            Right(line) => debug!("+{}", line),
            Both(_, _) => {}
        }
    }
    bail!(
        "generated manifest does not match {}",
        reference_path.display()
    );
}

pub(crate) fn apply_source_url(manifest: &mut Value, source: &str) {
    if let Some(obj) = manifest.as_object_mut() {
        obj.insert("source".to_string(), Value::String(source.to_string()));
        let extra = obj
            .entry("extra")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Some(extra_obj) = extra.as_object_mut() {
            extra_obj.insert("nefSource".to_string(), Value::String(source.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compare_manifest;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn compare_manifest_matches_reference_file() {
        let file = NamedTempFile::new().unwrap();
        let value = json!({"name": "Contract"});
        std::fs::write(file.path(), serde_json::to_string(&value).unwrap()).unwrap();
        compare_manifest(file.path(), &value).unwrap();
    }

    #[test]
    fn compare_manifest_detects_difference() {
        let file = NamedTempFile::new().unwrap();
        let reference = json!({"name": "Reference"});
        let generated = json!({"name": "Generated"});
        std::fs::write(file.path(), serde_json::to_string(&reference).unwrap()).unwrap();
        let err = compare_manifest(file.path(), &generated).unwrap_err();
        assert!(err
            .to_string()
            .contains("generated manifest does not match"));
    }
}
