// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

mod access;
mod bulk;
mod management;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    if management::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if access::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if bulk::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    Ok(false)
}
