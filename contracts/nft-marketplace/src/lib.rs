use core::slice;
use neo_devpack::serde::{Deserialize, Serialize};
use neo_devpack::{codec, prelude::*};

const LISTING_COUNTER_KEY: &[u8] = b"market:counter";
const LISTING_PREFIX: &[u8] = b"market:listing:";
const ESCROW_PREFIX: &[u8] = b"market:escrow:";

#[derive(Clone, Serialize, Deserialize)]
struct Listing {
    id: i64,
    seller: NeoByteString,
    buyer: Option<NeoByteString>,
    token_contract: NeoByteString,
    token_id: NeoByteString,
    payment_token: NeoByteString,
    price: i64,
    active: bool,
    escrowed: bool,
}

neo_manifest_overlay!(
    r#"{
    "name": "NeoNFTMarketplace",
    "supportedstandards": ["NEP-11", "NEP-17"],
    "features": { "storage": true }
}"#
);

#[neo_event]
pub struct ListingCreated {
    pub listing_id: NeoInteger,
    pub seller: NeoByteString,
    pub price: NeoInteger,
}

#[neo_event]
pub struct ListingEscrowed {
    pub listing_id: NeoInteger,
}

#[neo_event]
pub struct ListingSold {
    pub listing_id: NeoInteger,
    pub buyer: NeoByteString,
}

#[neo_event]
pub struct ListingCancelled {
    pub listing_id: NeoInteger,
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn getListing(listing_id: i64) -> NeoByteString {
    storage_context()
        .and_then(|ctx| load_listing(&ctx, listing_id))
        .map(|listing| serialize_value(&listing))
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[no_mangle]
pub extern "C" fn createListing(
    seller_ptr: i64,
    seller_len: i64,
    token_contract_ptr: i64,
    token_contract_len: i64,
    token_id_ptr: i64,
    token_id_len: i64,
    payment_token_ptr: i64,
    payment_token_len: i64,
    price: i64,
) -> i64 {
    if price <= 0 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };

    let Some(seller) = read_address(seller_ptr, seller_len) else {
        return 0;
    };
    if !ensure_witness(&seller) {
        return 0;
    }

    let Some(token_contract) = read_address(token_contract_ptr, token_contract_len) else {
        return 0;
    };
    let Some(token_id_bytes) = read_bytes(token_id_ptr, token_id_len) else {
        return 0;
    };
    let token_id = NeoByteString::from_slice(&token_id_bytes);
    let Some(payment_token) = read_address(payment_token_ptr, payment_token_len) else {
        return 0;
    };

    let listing_id = match next_listing_id(&ctx) {
        Some(id) => id,
        None => return 0,
    };

    let listing = Listing {
        id: listing_id,
        seller: seller.clone(),
        buyer: None,
        token_contract: token_contract.clone(),
        token_id: token_id.clone(),
        payment_token: payment_token.clone(),
        price,
        active: true,
        escrowed: false,
    };

    if store_listing(&ctx, listing_id, &listing).is_err() {
        return 0;
    }
    if store_mapping(&ctx, &token_contract, &token_id, listing_id).is_err() {
        return 0;
    }

    ListingCreated {
        listing_id: NeoInteger::new(listing_id),
        seller,
        price: NeoInteger::new(price),
    }
    .emit()
    .ok();

    listing_id
}

#[no_mangle]
pub extern "C" fn cancelListing(listing_id: i64, seller_ptr: i64, seller_len: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(mut listing) = load_listing(&ctx, listing_id) else {
        return 0;
    };
    if !listing.active {
        return 0;
    }

    let Some(seller) = read_address(seller_ptr, seller_len) else {
        return 0;
    };
    if !addresses_equal(&seller, &listing.seller) || !ensure_witness(&seller) {
        return 0;
    }

    if listing.escrowed {
        if !return_nft(&listing.token_contract, &seller, &listing.token_id) {
            return 0;
        }
        if let Some(ref buyer) = listing.buyer {
            let contract_hash = match NeoRuntime::get_executing_script_hash() {
                Ok(hash) => hash,
                Err(_) => return 0,
            };
            if !transfer_payment(&listing.payment_token, &contract_hash, buyer, listing.price) {
                return 0;
            }
        }
        listing.escrowed = false;
        listing.buyer = None;
    }

    listing.active = false;
    if store_listing(&ctx, listing_id, &listing).is_err() {
        return 0;
    }
    remove_mapping(&ctx, &listing.token_contract, &listing.token_id).ok();

    ListingCancelled {
        listing_id: NeoInteger::new(listing_id),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn commitToPurchase(listing_id: i64, buyer_ptr: i64, buyer_len: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(mut listing) = load_listing(&ctx, listing_id) else {
        return 0;
    };
    if !listing.active || !listing.escrowed {
        return 0;
    }
    if listing.buyer.is_some() {
        return 0;
    }
    let Some(buyer) = read_address(buyer_ptr, buyer_len) else {
        return 0;
    };
    if !ensure_witness(&buyer) {
        return 0;
    }

    listing.buyer = Some(buyer.clone());
    if store_listing(&ctx, listing_id, &listing).is_err() {
        return 0;
    }

    1
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn onNEP11Payment(
    from: NeoByteString,
    token_id: NeoByteString,
    amount: i64,
    _data: NeoByteString,
) -> i64 {
    if amount != 1 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Ok(call_hash) = NeoRuntime::get_calling_script_hash() else {
        return 0;
    };

    let Some(listing_id) = load_mapping(&ctx, &call_hash, &token_id) else {
        return 0;
    };
    let Some(mut listing) = load_listing(&ctx, listing_id) else {
        return 0;
    };
    if !listing.active || listing.escrowed {
        return 0;
    }
    if !addresses_equal(&from, &listing.seller) {
        return 0;
    }

    listing.escrowed = true;
    if store_listing(&ctx, listing_id, &listing).is_err() {
        return 0;
    }

    ListingEscrowed {
        listing_id: NeoInteger::new(listing_id),
    }
    .emit()
    .ok();

    1
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn onNEP17Payment(from: NeoByteString, amount: i64, data: NeoByteString) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(listing_id) = parse_i64(&data) else {
        return 0;
    };
    let Some(mut listing) = load_listing(&ctx, listing_id) else {
        return 0;
    };
    if !listing.active || !listing.escrowed {
        return 0;
    }
    let Some(ref committed_buyer) = listing.buyer else {
        return 0;
    };
    if !addresses_equal(&from, committed_buyer) {
        return 0;
    }
    if amount != listing.price {
        return 0;
    }

    let Ok(payment_hash) = NeoRuntime::get_calling_script_hash() else {
        return 0;
    };
    if !addresses_equal(&payment_hash, &listing.payment_token) {
        return 0;
    }

    let contract_hash = match NeoRuntime::get_executing_script_hash() {
        Ok(hash) => hash,
        Err(_) => return 0,
    };

    if !transfer_payment(
        &listing.payment_token,
        &contract_hash,
        &listing.seller,
        amount,
    ) {
        return 0;
    }

    if !return_nft(&listing.token_contract, &from, &listing.token_id) {
        let _ = transfer_payment(
            &listing.payment_token,
            &listing.seller,
            &contract_hash,
            amount,
        );
        return 0;
    }

    listing.active = false;
    listing.escrowed = false;
    if store_listing(&ctx, listing_id, &listing).is_err() {
        return 0;
    }
    remove_mapping(&ctx, &listing.token_contract, &listing.token_id).ok();

    ListingSold {
        listing_id: NeoInteger::new(listing_id),
        buyer: from,
    }
    .emit()
    .ok();

    1
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn next_listing_id(ctx: &NeoStorageContext) -> Option<i64> {
    let current: i64 = load_from_storage(ctx, LISTING_COUNTER_KEY).unwrap_or(0);
    let next = current.checked_add(1)?;
    store_to_storage(ctx, LISTING_COUNTER_KEY, &next).ok()?;
    Some(next)
}

fn listing_key(id: i64) -> Vec<u8> {
    let mut key = LISTING_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key
}

fn load_listing(ctx: &NeoStorageContext, id: i64) -> Option<Listing> {
    load_from_storage(ctx, &listing_key(id))
}

fn store_listing(ctx: &NeoStorageContext, id: i64, listing: &Listing) -> NeoResult<()> {
    store_to_storage(ctx, &listing_key(id), listing)
}

fn mapping_key(contract: &NeoByteString, token_id: &NeoByteString) -> Vec<u8> {
    let mut key = ESCROW_PREFIX.to_vec();
    key.extend_from_slice(contract.as_slice());
    key.push(b':');
    key.extend_from_slice(token_id.as_slice());
    key
}

fn store_mapping(
    ctx: &NeoStorageContext,
    contract: &NeoByteString,
    token_id: &NeoByteString,
    listing_id: i64,
) -> NeoResult<()> {
    store_to_storage(ctx, &mapping_key(contract, token_id), &listing_id)
}

fn load_mapping(
    ctx: &NeoStorageContext,
    contract: &NeoByteString,
    token_id: &NeoByteString,
) -> Option<i64> {
    load_from_storage(ctx, &mapping_key(contract, token_id))
}

fn remove_mapping(
    ctx: &NeoStorageContext,
    contract: &NeoByteString,
    token_id: &NeoByteString,
) -> NeoResult<()> {
    let key = NeoByteString::from_slice(&mapping_key(contract, token_id));
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

fn parse_i64(data: &NeoByteString) -> Option<i64> {
    if data.len() != 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(data.as_slice());
    Some(i64::from_le_bytes(buf))
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn addresses_equal(left: &NeoByteString, right: &NeoByteString) -> bool {
    left.as_slice() == right.as_slice()
}

fn transfer_payment(
    payment_token: &NeoByteString,
    from: &NeoByteString,
    to: &NeoByteString,
    amount: i64,
) -> bool {
    let mut args = NeoArray::new();
    args.push(NeoValue::from(from.clone()));
    args.push(NeoValue::from(to.clone()));
    args.push(NeoValue::from(NeoInteger::new(amount)));

    match NeoContractRuntime::call(payment_token, &NeoString::from_str("transfer"), &args) {
        Ok(value) => value
            .as_boolean()
            .map(|flag| flag.as_bool())
            .unwrap_or(false),
        Err(_) => false,
    }
}

fn return_nft(
    token_contract: &NeoByteString,
    recipient: &NeoByteString,
    token_id: &NeoByteString,
) -> bool {
    let mut args = NeoArray::new();
    args.push(NeoValue::from(recipient.clone()));
    args.push(NeoValue::from(token_id.clone()));
    args.push(NeoValue::from(NeoByteString::new(Vec::new())));

    NeoContractRuntime::call(token_contract, &NeoString::from_str("transfer"), &args).is_ok()
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
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(LISTING_COUNTER_KEY)).ok();
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(LISTING_PREFIX)) {
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
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(ESCROW_PREFIX)) {
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

    fn create_sample_listing(price: i64) -> Listing {
        reset_state();
        let seller = address(0x01);
        let token_contract = address(0x00);
        let payment_token = address(0x00);
        let token_id = vec![0xAB, 0xCD];
        let id = createListing(
            seller.as_ptr() as i64,
            seller.len() as i64,
            token_contract.as_ptr() as i64,
            token_contract.len() as i64,
            token_id.as_ptr() as i64,
            token_id.len() as i64,
            payment_token.as_ptr() as i64,
            payment_token.len() as i64,
            price,
        );
        assert!(id > 0);
        let listing_bytes = getListing(id);
        codec::deserialize(listing_bytes.as_slice()).expect("listing decode")
    }

    #[test]
    fn create_and_fetch_listing() {
        let _guard = test_lock().lock().unwrap();
        let listing = create_sample_listing(500);
        assert_eq!(listing.price, 500);
        assert!(listing.active);
        assert!(!listing.escrowed);
    }

    #[test]
    fn escrow_and_sale_flow() {
        let _guard = test_lock().lock().unwrap();
        let listing = create_sample_listing(750);
        let ctx = storage_context().unwrap();

        let token_id = listing.token_id.clone();
        let seller = listing.seller.clone();
        assert_eq!(
            onNEP11Payment(
                seller.clone(),
                token_id.clone(),
                1,
                NeoByteString::new(Vec::new())
            ),
            1
        );

        let stored = load_listing(&ctx, listing.id).unwrap();
        assert!(stored.escrowed);

        let id_bytes = listing.id.to_le_bytes().to_vec();
        let buyer = address(0x02);
        assert_eq!(
            onNEP17Payment(
                NeoByteString::from_slice(&buyer),
                listing.price,
                NeoByteString::from_slice(&id_bytes)
            ),
            1
        );

        let final_listing = load_listing(&ctx, listing.id).unwrap();
        assert!(!final_listing.active);
        assert!(!final_listing.escrowed);
    }
}
