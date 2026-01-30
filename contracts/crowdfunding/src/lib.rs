use core::slice;
use neo_devpack::{codec, prelude::*};
use neo_devpack::serde::{Deserialize, Serialize};

const CAMPAIGN_KEY: &[u8] = b"crowd:campaign";
const CONTRIBUTION_PREFIX: &[u8] = b"crowd:contrib:";

#[derive(Clone, Serialize, Deserialize)]
struct Campaign {
    owner: NeoByteString,
    token: NeoByteString,
    target: i64,
    deadline: i64,
    total_raised: i64,
    finalized: bool,
    successful: bool,
}

neo_manifest_overlay!(
    r#"{
    "name": "NeoCrowdfund",
    "supportedstandards": ["NEP-17"],
    "features": { "storage": true }
}"#
);

#[neo_event]
pub struct CampaignConfigured {
    pub owner: NeoByteString,
    pub target: NeoInteger,
    pub deadline: NeoInteger,
}

#[neo_event]
pub struct ContributionReceived {
    pub contributor: NeoByteString,
    pub amount: NeoInteger,
    pub total_raised: NeoInteger,
}

#[neo_event]
pub struct CampaignFinalized {
    pub successful: NeoBoolean,
    pub total_raised: NeoInteger,
}

#[neo_event]
pub struct RefundIssued {
    pub contributor: NeoByteString,
    pub amount: NeoInteger,
}

#[no_mangle]
#[neo_safe]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn getCampaign() -> NeoByteString {
    storage_context()
        .and_then(|ctx| load_campaign(&ctx))
        .map(|campaign| serialize_value(&campaign))
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[no_mangle]
#[neo_safe]
pub extern "C" fn contributionOf(address_ptr: i64, address_len: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(address) = read_address(address_ptr, address_len) else {
        return 0;
    };
    load_contribution(&ctx, &address)
}

#[no_mangle]
pub extern "C" fn configure(
    owner_ptr: i64,
    owner_len: i64,
    token_ptr: i64,
    token_len: i64,
    target: i64,
    deadline: i64,
) -> i64 {
    if target <= 0 || deadline <= 0 {
        return 0;
    }

    let Some(ctx) = storage_context() else {
        return 0;
    };
    if load_campaign(&ctx).is_some() {
        return 0;
    }

    let Some(owner) = read_address(owner_ptr, owner_len) else {
        return 0;
    };
    let Some(token) = read_address(token_ptr, token_len) else {
        return 0;
    };

    let campaign = Campaign {
        owner: owner.clone(),
        token: token.clone(),
        target,
        deadline,
        total_raised: 0,
        finalized: false,
        successful: false,
    };

    if store_campaign(&ctx, &campaign).is_err() {
        return 0;
    }

    CampaignConfigured {
        owner,
        target: NeoInteger::new(target),
        deadline: NeoInteger::new(deadline),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn finalize(current_time: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(mut campaign) = load_campaign(&ctx) else {
        return 0;
    };
    if campaign.finalized {
        return 0;
    }

    if campaign.total_raised < campaign.target && current_time < campaign.deadline {
        return 0;
    }

    let successful = campaign.total_raised >= campaign.target;
    if successful {
        let contract_hash = match NeoRuntime::get_executing_script_hash() {
            Ok(hash) => hash,
            Err(_) => return 0,
        };
        if !call_transfer(
            &campaign.token,
            &contract_hash,
            &campaign.owner,
            campaign.total_raised,
        ) {
            return 0;
        }
    }

    campaign.finalized = true;
    campaign.successful = successful;

    if store_campaign(&ctx, &campaign).is_err() {
        return 0;
    }

    CampaignFinalized {
        successful: NeoBoolean::new(successful),
        total_raised: NeoInteger::new(campaign.total_raised),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn claimRefund(contrib_ptr: i64, contrib_len: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(campaign) = load_campaign(&ctx) else {
        return 0;
    };
    if !campaign.finalized || campaign.successful {
        return 0;
    }

    let Some(contributor) = read_address(contrib_ptr, contrib_len) else {
        return 0;
    };
    if !ensure_witness(&contributor) {
        return 0;
    }

    let amount = load_contribution(&ctx, &contributor);
    if amount <= 0 {
        return 0;
    }

    let contract_hash = match NeoRuntime::get_executing_script_hash() {
        Ok(hash) => hash,
        Err(_) => return 0,
    };

    if !call_transfer(&campaign.token, &contract_hash, &contributor, amount) {
        return 0;
    }

    if delete_contribution(&ctx, &contributor).is_err() {
        return 0;
    }

    RefundIssued {
        contributor,
        amount: NeoInteger::new(amount),
    }
    .emit()
    .ok();

    1
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn onNEP17Payment(from: NeoByteString, amount: i64, data: NeoByteString) {
    if amount <= 0 {
        return;
    }

    let Some(ctx) = storage_context() else {
        return;
    };
    let Some(mut campaign) = load_campaign(&ctx) else {
        return;
    };
    if campaign.finalized {
        return;
    }

    let Ok(call_hash) = NeoRuntime::get_calling_script_hash() else {
        return;
    };
    if !addresses_equal(&call_hash, &campaign.token) {
        return;
    }

    if !data.is_empty() && data.as_slice() != b"contribute" {
        return;
    }

    let new_total = match campaign.total_raised.checked_add(amount) {
        Some(value) => value,
        None => return,
    };
    campaign.total_raised = new_total;

    let existing = load_contribution(&ctx, &from);
    if store_contribution(&ctx, &from, existing + amount).is_err() {
        return;
    }

    if store_campaign(&ctx, &campaign).is_err() {
        return;
    }

    ContributionReceived {
        contributor: from,
        amount: NeoInteger::new(amount),
        total_raised: NeoInteger::new(new_total),
    }
    .emit()
    .ok();
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn load_campaign(ctx: &NeoStorageContext) -> Option<Campaign> {
    load_from_storage(ctx, CAMPAIGN_KEY)
}

fn store_campaign(ctx: &NeoStorageContext, campaign: &Campaign) -> NeoResult<()> {
    store_to_storage(ctx, CAMPAIGN_KEY, campaign)
}

fn contribution_key(address: &NeoByteString) -> Vec<u8> {
    let mut key = CONTRIBUTION_PREFIX.to_vec();
    key.extend_from_slice(address.as_slice());
    key
}

fn load_contribution(ctx: &NeoStorageContext, address: &NeoByteString) -> i64 {
    load_from_storage(ctx, &contribution_key(address)).unwrap_or(0i64)
}

fn store_contribution(
    ctx: &NeoStorageContext,
    address: &NeoByteString,
    amount: i64,
) -> NeoResult<()> {
    store_to_storage(ctx, &contribution_key(address), &amount)
}

fn delete_contribution(ctx: &NeoStorageContext, address: &NeoByteString) -> NeoResult<()> {
    let key = NeoByteString::from_slice(&contribution_key(address));
    NeoStorage::delete(ctx, &key)
}

fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    let bytes = read_bytes(ptr, len)?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

/// Reads bytes from a raw pointer.
/// 
/// # Safety
/// 
/// The caller must ensure that:
/// - `ptr` is a valid, non-null pointer allocated by the NeoVM runtime
/// - `len` bytes starting at `ptr` are valid for reads
/// 
/// These invariants are guaranteed when called from NeoVM contract entry points.
fn read_bytes(ptr: i64, len: i64) -> Option<Vec<u8>> {
    if ptr == 0 || len <= 0 {
        return None;
    }
    let len = len as usize;
    // SAFETY: We've validated ptr is non-null and len is positive.
    // The pointer validity is guaranteed by the NeoVM runtime.
    let slice = unsafe { slice::from_raw_parts(ptr as *const u8, len) };
    Some(slice.to_vec())
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn addresses_equal(left: &NeoByteString, right: &NeoByteString) -> bool {
    left.as_slice() == right.as_slice()
}

fn call_transfer(
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
            .unwrap_or(true),
        Err(_) => false,
    }
}

fn load_from_storage<T>(ctx: &NeoStorageContext, key: &[u8]) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let key_bytes = NeoByteString::from_slice(key);
    let data = NeoStorage::get(ctx, &key_bytes).ok()?;
    if data.is_empty() {
        return None;
    }
    codec::deserialize(data.as_slice()).ok()
}

fn store_to_storage<T>(ctx: &NeoStorageContext, key: &[u8], value: &T) -> NeoResult<()>
where
    T: Serialize,
{
    let encoded = codec::serialize(value)?;
    let key_bytes = NeoByteString::from_slice(key);
    let value_bytes = NeoByteString::from_slice(&encoded);
    NeoStorage::put(ctx, &key_bytes, &value_bytes)
}

fn serialize_value<T: Serialize>(value: &T) -> NeoByteString {
    match codec::serialize(value) {
        Ok(bytes) => NeoByteString::from_slice(&bytes),
        Err(_) => NeoByteString::new(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn address(byte: u8) -> Vec<u8> {
        vec![byte; 20]
    }

    fn reset_state() {
        let ctx = storage_context().unwrap();
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(CAMPAIGN_KEY)).ok();
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(CONTRIBUTION_PREFIX)) {
            let mut iterator = iter;
            while iterator.has_next() {
                if let Some(entry) = iterator.next() {
                    if let Some(key) = entry
                        .as_struct()
                        .and_then(|st| st.get_field("key"))
                        .and_then(NeoValue::as_byte_string)
                    {
                        NeoStorage::delete(&ctx, &key).ok();
                    }
                }
            }
        }
    }

    fn configure_sample(target: i64, deadline: i64) -> Campaign {
        reset_state();
        let owner = address(0x44);
        let token = address(0x00);

        let status = configure(
            owner.as_ptr() as i64,
            owner.len() as i64,
            token.as_ptr() as i64,
            token.len() as i64,
            target,
            deadline,
        );
        assert_eq!(status, 1);

        let campaign_bytes = getCampaign();
        codec::deserialize(campaign_bytes.as_slice()).expect("campaign decode")
    }

    #[test]
    fn configure_and_view_campaign() {
        let _guard = test_lock().lock().unwrap();
        let campaign = configure_sample(1_000, 100);
        assert_eq!(campaign.target, 1_000);
        assert_eq!(campaign.total_raised, 0);
    }

    #[test]
    fn contribution_updates_total() {
        let _guard = test_lock().lock().unwrap();
        let campaign = configure_sample(1_000, 100);
        let ctx = storage_context().unwrap();
        let contributor = campaign.owner.clone();

        onNEP17Payment(contributor.clone(), 400, NeoByteString::new(Vec::new()));

        let updated = load_campaign(&ctx).unwrap();
        assert_eq!(updated.total_raised, 400);
        assert_eq!(load_contribution(&ctx, &contributor), 400);
    }

    #[test]
    fn finalize_successful_campaign() {
        let _guard = test_lock().lock().unwrap();
        let campaign = configure_sample(500, 100);
        let ctx = storage_context().unwrap();
        let contributor = campaign.owner.clone();
        onNEP17Payment(contributor, 600, NeoByteString::new(Vec::new()));

        let status = finalize(0);
        assert_eq!(status, 1);

        let stored = load_campaign(&ctx).unwrap();
        assert!(stored.finalized);
        assert!(stored.successful);
    }

    #[test]
    fn refunds_after_failed_campaign() {
        let _guard = test_lock().lock().unwrap();
        let campaign = configure_sample(800, 10);
        let ctx = storage_context().unwrap();
        let contributor = campaign.owner.clone();
        onNEP17Payment(contributor.clone(), 300, NeoByteString::new(Vec::new()));

        assert_eq!(finalize(10), 1);
        let stored = load_campaign(&ctx).unwrap();
        assert!(stored.finalized);
        assert!(!stored.successful);

        let contributor_bytes = contributor.as_slice().to_vec();
        assert_eq!(
            claimRefund(
                contributor_bytes.as_ptr() as i64,
                contributor_bytes.len() as i64
            ),
            1
        );
        assert_eq!(load_contribution(&ctx, &contributor), 0);
    }
}
