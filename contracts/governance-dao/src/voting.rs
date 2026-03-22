// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

use crate::storage::*;

pub fn has_voted(ctx: &NeoStorageContext, id: i64, address: &NeoByteString) -> bool {
    load_from_storage(ctx, &vote_key(id, address)).unwrap_or(false)
}

pub fn record_vote(ctx: &NeoStorageContext, id: i64, address: &NeoByteString) -> NeoResult<()> {
    store_to_storage(ctx, &vote_key(id, address), &true)
}

pub fn call_transfer(
    token: &NeoByteString,
    from: &NeoByteString,
    to: &NeoByteString,
    amount: i64,
) -> bool {
    let mut args = NeoArray::new();
    args.push(NeoValue::from(from.clone()));
    args.push(NeoValue::from(to.clone()));
    args.push(NeoValue::from(NeoInteger::new(amount)));

    match NeoContractRuntime::call(token, &NeoString::from_str("transfer"), &args) {
        Ok(value) => value
            .as_boolean()
            .map(|flag| flag.as_bool())
            .unwrap_or(false),
        Err(_) => false,
    }
}
