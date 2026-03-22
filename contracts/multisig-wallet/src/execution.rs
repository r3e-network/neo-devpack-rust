// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

use crate::types::{build_argument_array, CallArgument};

pub fn execute_proposal(
    target: &NeoByteString,
    method: &str,
    arguments: &[CallArgument],
) -> NeoResult<()> {
    let args = build_argument_array(arguments).ok_or_else(|| {
        NeoError::new("Failed to build argument array")
    })?;

    NeoContractRuntime::call(
        target,
        &NeoString::from_str(method),
        &args,
    )
    .map(|_| ())
}
