// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT
//
//! Heap-free multisig sample.

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "SampleMultisig"
}"#
);

const CFG_THRESHOLD: &[u8] = b"cfg:threshold";
const CFG_OWNER_COUNT: &[u8] = b"cfg:owners";

#[neo_contract]
pub struct SampleMultisigContract;

#[neo_contract]
impl SampleMultisigContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(safe, name = "threshold")]
    pub fn threshold() -> i64 {
        RawStorage::get_i64(CFG_THRESHOLD).unwrap_or(-1)
    }

    #[neo_method(safe, name = "ownerCount")]
    pub fn owner_count() -> i64 {
        RawStorage::get_i64(CFG_OWNER_COUNT).unwrap_or(-1)
    }
}

impl Default for SampleMultisigContract {
    fn default() -> Self {
        Self::new()
    }
}
