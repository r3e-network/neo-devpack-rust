// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use neo_types::*;

/// Minimal representation of contract management utilities used in tests.
pub struct NeoContractRuntime;

impl NeoContractRuntime {
    pub fn create(
        script: &NeoByteString,
        _manifest: &NeoContractManifest,
    ) -> NeoResult<NeoByteString> {
        let mut data = script.as_slice().to_vec();
        data.extend_from_slice(&[0x01, 0x02, 0x03]);
        Ok(NeoByteString::new(data))
    }

    pub fn update(
        _script_hash: &NeoByteString,
        _script: &NeoByteString,
        _manifest: &NeoContractManifest,
    ) -> NeoResult<()> {
        Ok(())
    }

    pub fn destroy(_script_hash: &NeoByteString) -> NeoResult<()> {
        Ok(())
    }

    pub fn call(
        _script_hash: &NeoByteString,
        _method: &NeoString,
        _args: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoValue> {
        Ok(NeoValue::Null)
    }
}
