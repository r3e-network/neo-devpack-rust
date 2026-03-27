// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoNFTMarketplace"
}"#
);

// Storage keys
const LISTING_PREFIX: &[u8] = b"listing:";
const SELLER_SUFFIX: &[u8] = b":seller";
const TOKEN_CONTRACT_SUFFIX: &[u8] = b":tc";
const TOKEN_ID_SUFFIX: &[u8] = b":tid";
const PAYMENT_TOKEN_SUFFIX: &[u8] = b":pt";
const PRICE_SUFFIX: &[u8] = b":price";
const FEE_BPS_SUFFIX: &[u8] = b":fee";
const EXPIRY_SUFFIX: &[u8] = b":expiry";
const ACTIVE_SUFFIX: &[u8] = b":active";

fn listing_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = LISTING_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(suffix);
    key
}

fn storage_put_i64(ctx: &NeoStorageContext, key: &[u8], value: i64) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(&value.to_le_bytes()),
    )
    .is_ok()
}

fn storage_get_i64(ctx: &NeoStorageContext, key: &[u8]) -> Option<i64> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    if data.len() != 8 {
        return None;
    }
    let s = data.as_slice();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(s);
    Some(i64::from_le_bytes(buf))
}

// Events
#[neo_event]
pub struct ListingCreated {
    pub listing_id: NeoInteger,
    pub seller: NeoInteger,
    pub price: NeoInteger,
}

#[neo_event]
pub struct ListingCancelled {
    pub listing_id: NeoInteger,
}

#[neo_contract]
pub struct NeoNftMarketplaceContract;

#[neo_contract]
impl NeoNftMarketplaceContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(name = "createListing")]
    pub fn create_listing(
        seller_id: i64,
        token_contract_id: i64,
        token_id: i64,
        payment_token_id: i64,
        price: i64,
        fee_bps: i64,
        expiry: i64,
        listing_id: i64,
    ) -> bool {
        if price <= 0 || fee_bps < 0 || expiry <= 0 || listing_id < 0 || token_id < 0 {
            return false;
        }
        if seller_id == 0 || token_contract_id == 0 || payment_token_id == 0 {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        // Check listing not already active
        if storage_get_i64(&ctx, &listing_key(listing_id, ACTIVE_SUFFIX)).is_some() {
            return false;
        }
        storage_put_i64(&ctx, &listing_key(listing_id, SELLER_SUFFIX), seller_id);
        storage_put_i64(&ctx, &listing_key(listing_id, TOKEN_CONTRACT_SUFFIX), token_contract_id);
        storage_put_i64(&ctx, &listing_key(listing_id, TOKEN_ID_SUFFIX), token_id);
        storage_put_i64(&ctx, &listing_key(listing_id, PAYMENT_TOKEN_SUFFIX), payment_token_id);
        storage_put_i64(&ctx, &listing_key(listing_id, PRICE_SUFFIX), price);
        storage_put_i64(&ctx, &listing_key(listing_id, FEE_BPS_SUFFIX), fee_bps);
        storage_put_i64(&ctx, &listing_key(listing_id, EXPIRY_SUFFIX), expiry);
        storage_put_i64(&ctx, &listing_key(listing_id, ACTIVE_SUFFIX), 1);
        let _ = (ListingCreated {
            listing_id: NeoInteger::new(listing_id),
            seller: NeoInteger::new(seller_id),
            price: NeoInteger::new(price),
        })
        .emit();
        true
    }

    #[neo_method(name = "cancelListing")]
    pub fn cancel_listing(listing_id: i64, caller_id: i64) -> bool {
        if listing_id < 0 || caller_id == 0 {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let active = storage_get_i64(&ctx, &listing_key(listing_id, ACTIVE_SUFFIX)).unwrap_or(0);
        if active != 1 {
            return false;
        }
        let seller = match storage_get_i64(&ctx, &listing_key(listing_id, SELLER_SUFFIX)) {
            Some(s) => s,
            None => return false,
        };
        if caller_id != seller {
            return false;
        }
        storage_put_i64(&ctx, &listing_key(listing_id, ACTIVE_SUFFIX), 0);
        let _ = (ListingCancelled {
            listing_id: NeoInteger::new(listing_id),
        })
        .emit();
        true
    }

    #[neo_method(name = "onNEP11Payment")]
    pub fn on_nep11_payment(_from: i64, _amount: i64, _token_id: i64, _data: i64) -> bool {
        true
    }

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(_from: i64, _amount: i64, _data: i64) -> bool {
        true
    }

    /// Return listing data via notify: [price, fee_bps, expiry, token_id, active]
    #[neo_method(safe, name = "getListing")]
    pub fn get_listing(listing_id: i64) {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let price = match storage_get_i64(&ctx, &listing_key(listing_id, PRICE_SUFFIX)) {
            Some(v) => v,
            None => return,
        };
        let fee_bps = storage_get_i64(&ctx, &listing_key(listing_id, FEE_BPS_SUFFIX)).unwrap_or(0);
        let expiry = storage_get_i64(&ctx, &listing_key(listing_id, EXPIRY_SUFFIX)).unwrap_or(0);
        let token_id = storage_get_i64(&ctx, &listing_key(listing_id, TOKEN_ID_SUFFIX)).unwrap_or(0);
        let active = storage_get_i64(&ctx, &listing_key(listing_id, ACTIVE_SUFFIX)).unwrap_or(0);
        let label = NeoString::from_str("getListing");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoInteger::new(price)));
        state.push(NeoValue::from(NeoInteger::new(fee_bps)));
        state.push(NeoValue::from(NeoInteger::new(expiry)));
        state.push(NeoValue::from(NeoInteger::new(token_id)));
        state.push(NeoValue::from(NeoBoolean::new(active != 0)));
        let _ = NeoRuntime::notify(&label, &state);
    }
}

impl Default for NeoNftMarketplaceContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }
}
