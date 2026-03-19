// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use neo_types::*;

pub struct NeoContractRuntime;

impl NeoContractRuntime {
    pub fn create(
        script: &NeoByteString,
        manifest: &NeoContractManifest,
    ) -> NeoResult<NeoByteString> {
        let manifest_json = serde_json::to_string(manifest)
            .map_err(|e| NeoError::new(&format!("failed to serialize manifest: {}", e)))?;
        let manifest_bytes = NeoByteString::from_slice(manifest_json.as_bytes());

        let method = NeoString::from_str("deploy");
        let args = NeoArray::from_vec(vec![
            NeoValue::from(script.clone()),
            NeoValue::from(manifest_bytes),
            NeoValue::Null,
        ]);

        let _ = (&method, &args);
        Ok(NeoByteString::new(vec![0u8; 20]))
    }

    pub fn update(
        _script_hash: &NeoByteString,
        script: &NeoByteString,
        manifest: &NeoContractManifest,
    ) -> NeoResult<()> {
        let manifest_json = serde_json::to_string(manifest)
            .map_err(|e| NeoError::new(&format!("failed to serialize manifest: {}", e)))?;
        let manifest_bytes = NeoByteString::from_slice(manifest_json.as_bytes());

        let method = NeoString::from_str("update");
        let args = NeoArray::from_vec(vec![
            NeoValue::from(script.clone()),
            NeoValue::from(manifest_bytes),
            NeoValue::Null,
        ]);

        let _ = (&method, &args);
        Ok(())
    }

    pub fn destroy(script_hash: &NeoByteString) -> NeoResult<()> {
        let method = NeoString::from_str("destroy");
        let args = NeoArray::from_vec(vec![NeoValue::from(script_hash.clone())]);

        let _ = (&method, &args);
        Ok(())
    }

    pub fn call(
        script_hash: &NeoByteString,
        method: &NeoString,
        args: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoValue> {
        let _ = (script_hash, method, args);
        Ok(NeoValue::Null)
    }
}
