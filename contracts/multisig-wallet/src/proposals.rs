// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

use crate::storage::*;
use crate::types::{decode_arguments_from_bytes, encode_arguments, CallArgument, Proposal};
use crate::utils::read_bytes;

pub fn next_proposal_id(ctx: &NeoStorageContext) -> Option<i64> {
    let current = read_i64(ctx, PROPOSAL_COUNTER_KEY).unwrap_or(0);
    let next = current.checked_add(1)?;
    write_i64(ctx, PROPOSAL_COUNTER_KEY, next).ok()?;
    Some(next)
}

pub fn load_proposal(ctx: &NeoStorageContext, id: i64) -> Option<Proposal> {
    let proposer = read_proposal_address(ctx, id, PROPOSER_SUFFIX)?;
    let target = read_proposal_address(ctx, id, TARGET_SUFFIX)?;
    let method = read_proposal_string(ctx, id, METHOD_SUFFIX)?;
    let arguments = read_proposal_arguments(ctx, id)?;
    let approvals = read_proposal_approvals(ctx, id)?;
    let executed = read_proposal_bool(ctx, id, EXECUTED_SUFFIX)?;
    Some(Proposal {
        proposer,
        target,
        method,
        arguments,
        approvals,
        executed,
    })
}

pub fn store_proposal(ctx: &NeoStorageContext, id: i64, proposal: &Proposal) -> NeoResult<()> {
    write_bytes(
        ctx,
        &proposal_field_key(id, PROPOSER_SUFFIX),
        proposal.proposer.as_slice(),
    )?;
    write_bytes(
        ctx,
        &proposal_field_key(id, TARGET_SUFFIX),
        proposal.target.as_slice(),
    )?;
    write_string(
        ctx,
        &proposal_field_key(id, METHOD_SUFFIX),
        &proposal.method,
    )?;
    write_bytes(
        ctx,
        &proposal_field_key(id, ARG_SUFFIX),
        &encode_arguments(&proposal.arguments),
    )?;
    write_u16(
        ctx,
        &proposal_field_key(id, APPROVAL_COUNT_SUFFIX),
        proposal.approvals.len() as u16,
    )?;
    for (idx, approval) in proposal.approvals.iter().enumerate() {
        write_bytes(
            ctx,
            &proposal_approval_key(id, idx as u16),
            approval.as_slice(),
        )?;
    }
    write_bool(
        ctx,
        &proposal_field_key(id, EXECUTED_SUFFIX),
        proposal.executed,
    )?;
    Ok(())
}

pub fn remove_proposal_entries(ctx: &NeoStorageContext, id: i64) -> NeoResult<()> {
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, PROPOSER_SUFFIX)),
    );
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, TARGET_SUFFIX)),
    );
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, METHOD_SUFFIX)),
    );
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, ARG_SUFFIX)),
    );
    let count = read_u16(ctx, &proposal_field_key(id, APPROVAL_COUNT_SUFFIX)).unwrap_or(0);
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, APPROVAL_COUNT_SUFFIX)),
    );
    for idx in 0..count {
        let _ = NeoStorage::delete(
            ctx,
            &NeoByteString::from_slice(&proposal_approval_key(id, idx)),
        );
    }
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, EXECUTED_SUFFIX)),
    );
    Ok(())
}

fn read_proposal_address(ctx: &NeoStorageContext, id: i64, suffix: &[u8]) -> Option<NeoByteString> {
    let bytes = read_storage_bytes(ctx, &proposal_field_key(id, suffix))?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

fn read_proposal_string(ctx: &NeoStorageContext, id: i64, suffix: &[u8]) -> Option<String> {
    read_storage_string(ctx, &proposal_field_key(id, suffix))
}

fn read_proposal_bool(ctx: &NeoStorageContext, id: i64, suffix: &[u8]) -> Option<bool> {
    read_bool(ctx, &proposal_field_key(id, suffix))
}

fn read_proposal_arguments(ctx: &NeoStorageContext, id: i64) -> Option<Vec<CallArgument>> {
    let bytes = read_storage_bytes(ctx, &proposal_field_key(id, ARG_SUFFIX)).unwrap_or_default();
    decode_arguments_from_bytes(&bytes)
}

fn read_proposal_approvals(ctx: &NeoStorageContext, id: i64) -> Option<Vec<NeoByteString>> {
    let count = read_u16(ctx, &proposal_field_key(id, APPROVAL_COUNT_SUFFIX)).unwrap_or(0);
    let mut approvals = Vec::with_capacity(count as usize);
    for idx in 0..count {
        let bytes = read_storage_bytes(ctx, &proposal_approval_key(id, idx))?;
        if bytes.len() != 20 {
            return None;
        }
        approvals.push(NeoByteString::from_slice(&bytes));
    }
    Some(approvals)
}

pub fn decode_arguments(ptr: i64, len: i64) -> Option<Vec<CallArgument>> {
    if len <= 0 {
        return Some(Vec::new());
    }
    let bytes = read_bytes(ptr, len)?;
    decode_arguments_from_bytes(&bytes)
}
