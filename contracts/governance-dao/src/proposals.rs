use neo_devpack::prelude::*;

use crate::storage::*;
use crate::types::Proposal;

pub fn next_proposal_id(ctx: &NeoStorageContext) -> Option<i64> {
    let current: i64 = load_from_storage(ctx, PROPOSAL_COUNTER_KEY).unwrap_or(0);
    let next = current.checked_add(1)?;
    store_to_storage(ctx, PROPOSAL_COUNTER_KEY, &next).ok()?;
    Some(next)
}

pub fn load_proposal(ctx: &NeoStorageContext, id: i64) -> Option<Proposal> {
    load_from_storage(ctx, &proposal_key(id))
}

pub fn store_proposal(ctx: &NeoStorageContext, id: i64, proposal: &Proposal) -> NeoResult<()> {
    store_to_storage(ctx, &proposal_key(id), proposal)
}

pub fn execute_proposal(
    target: &NeoByteString,
    method: &str,
) -> NeoResult<()> {
    let args = NeoArray::<NeoValue>::new();
    NeoContractRuntime::call(
        target,
        &NeoString::from_str(method),
        &args,
    )
    .map(|_| ())
}
