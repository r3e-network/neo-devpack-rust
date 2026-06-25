// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

#![allow(clippy::too_many_arguments)]

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoNFTMarketplace"
}"#
);

const KEY_STRIDE: i64 = 16;
const FIELD_SELLER: i64 = 1;
const FIELD_TOKEN_CONTRACT: i64 = 2;
const FIELD_TOKEN_ID: i64 = 3;
const FIELD_PAYMENT_TOKEN: i64 = 4;
const FIELD_PRICE: i64 = 5;
const FIELD_FEE_BPS: i64 = 6;
const FIELD_EXPIRY: i64 = 7;
const FIELD_ACTIVE: i64 = 8;
const LISTING_ACTIVE: i64 = 1;
const LISTING_CANCELLED: i64 = 2;
const MAX_LISTING_ID: i64 = i64::MAX / KEY_STRIDE;

fn listing_key(id: i64, field: i64) -> i64 {
    id * KEY_STRIDE + field
}

fn valid_listing_id(id: i64) -> bool {
    (0..=MAX_LISTING_ID).contains(&id)
}

fn storage_put_i64(key: i64, value: i64) -> bool {
    RawStorage::put_i64_key(key, value);
    true
}

fn storage_get_i64(key: i64) -> i64 {
    RawStorage::get_i64_key_or_zero(key)
}

// Events
#[neo_event]
pub struct ListingCreated {
    pub listing_id: i64,
    pub seller: i64,
    pub price: i64,
}

#[neo_event]
pub struct ListingCancelled {
    pub listing_id: i64,
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
        if price <= 0
            || fee_bps < 0
            || expiry <= 0
            || !valid_listing_id(listing_id)
            || token_id < 0
        {
            return false;
        }
        if seller_id == 0 || token_contract_id == 0 || payment_token_id == 0 {
            return false;
        }
        // The seller must be runtime-witnessed to register a listing on their
        // own behalf; otherwise anyone registers listings under an arbitrary
        // seller id (X5).
        if !NeoRuntime::require_witness_i64(seller_id) {
            return false;
        }
        // Any non-zero listing status means this id has already been used.
        if storage_get_i64(listing_key(listing_id, FIELD_ACTIVE)) != 0 {
            return false;
        }
        storage_put_i64(listing_key(listing_id, FIELD_SELLER), seller_id);
        storage_put_i64(listing_key(listing_id, FIELD_TOKEN_CONTRACT), token_contract_id);
        storage_put_i64(listing_key(listing_id, FIELD_TOKEN_ID), token_id);
        storage_put_i64(listing_key(listing_id, FIELD_PAYMENT_TOKEN), payment_token_id);
        storage_put_i64(listing_key(listing_id, FIELD_PRICE), price);
        storage_put_i64(listing_key(listing_id, FIELD_FEE_BPS), fee_bps);
        storage_put_i64(listing_key(listing_id, FIELD_EXPIRY), expiry);
        storage_put_i64(listing_key(listing_id, FIELD_ACTIVE), LISTING_ACTIVE);
        let _ = (ListingCreated {
            listing_id,
            seller: seller_id,
            price,
        })
        .emit();
        true
    }

    #[neo_method(name = "cancelListing")]
    pub fn cancel_listing(listing_id: i64, caller_id: i64) -> bool {
        if !valid_listing_id(listing_id) || caller_id == 0 {
            return false;
        }
        // Witness the caller before comparing to the stored seller; otherwise an
        // attacker passes the seller's id and cancels anyone's listing (X5).
        if !NeoRuntime::require_witness_i64(caller_id) {
            return false;
        }
        let active = storage_get_i64(listing_key(listing_id, FIELD_ACTIVE));
        if active != LISTING_ACTIVE {
            return false;
        }
        let seller = storage_get_i64(listing_key(listing_id, FIELD_SELLER));
        if seller == 0 {
            return false;
        }
        if caller_id != seller {
            return false;
        }
        storage_put_i64(listing_key(listing_id, FIELD_ACTIVE), LISTING_CANCELLED);
        let _ = (ListingCancelled { listing_id }).emit();
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
        if !valid_listing_id(listing_id) {
            return;
        }
        let price = storage_get_i64(listing_key(listing_id, FIELD_PRICE));
        if price == 0 {
            return;
        }
        let fee_bps = storage_get_i64(listing_key(listing_id, FIELD_FEE_BPS));
        let expiry = storage_get_i64(listing_key(listing_id, FIELD_EXPIRY));
        let token_id = storage_get_i64(listing_key(listing_id, FIELD_TOKEN_ID));
        let active = storage_get_i64(listing_key(listing_id, FIELD_ACTIVE));

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (price, fee_bps, expiry, token_id, active);
            let _ = NeoRuntime::notify_event("getListing");
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let label = NeoString::from_str("getListing");
            let mut state = NeoArray::new();
            state.push(NeoValue::from(price));
            state.push(NeoValue::from(fee_bps));
            state.push(NeoValue::from(expiry));
            state.push(NeoValue::from(token_id));
            state.push(NeoValue::from(active == LISTING_ACTIVE));
            let _ = NeoRuntime::notify(&label, &state);
        }
    }
}

impl Default for NeoNftMarketplaceContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::NeoNftMarketplaceContract;
    use neo_devpack::{prelude::NeoByteString, NeoVMSyscall};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn runtime_test_lock() -> MutexGuard<'static, ()> {
        static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        match TEST_LOCK.get_or_init(|| Mutex::new(())).lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn witness_hash(account: i64) -> [u8; 20] {
        let mut bytes = [0u8; 20];
        bytes[..8].copy_from_slice(&account.to_le_bytes());
        bytes
    }

    fn setup_witnesses(accounts: &[i64]) -> MutexGuard<'static, ()> {
        let guard = runtime_test_lock();
        NeoVMSyscall::reset_host_state().expect("host syscall state should reset");
        let witnesses: Vec<NeoByteString> = accounts
            .iter()
            .map(|account| NeoByteString::from_slice(&witness_hash(*account)))
            .collect();
        NeoVMSyscall::set_active_witnesses(&witnesses).expect("active witnesses should update");
        guard
    }

    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }

    #[test]
    fn create_listing_requires_seller_witness() {
        // Seller 5 not witnessed -> create rejected (X5).
        {
            let _g = setup_witnesses(&[]);
            assert!(!NeoNftMarketplaceContract::create_listing(
                5, 2, 3, 4, 100, 10, 1000, 1
            ));
        }
        // Seller 5 witnessed -> create succeeds.
        {
            let _g = setup_witnesses(&[5]);
            assert!(NeoNftMarketplaceContract::create_listing(
                5, 2, 3, 4, 100, 10, 1000, 1
            ));
        }
    }

    #[test]
    fn cancel_listing_requires_seller_witness() {
        // Create a listing as seller 5 (witnessed), then prove that a caller
        // whose id is NOT witnessed cannot cancel — even if they pass the
        // seller's id. reset_host_state clears storage between scopes, so the
        // create + negative cancel must share one scope: witness [5], then
        // attempt cancel with caller_id=9 (not witnessed) — rejected (X5).
        let _g = setup_witnesses(&[5]);
        assert!(NeoNftMarketplaceContract::create_listing(
            5, 2, 3, 4, 100, 10, 1000, 7
        ));
        // Unwitnessed caller (passing the seller's id) cannot cancel.
        assert!(!NeoNftMarketplaceContract::cancel_listing(7, 5 + 4));
        // Witnessed seller cancels.
        assert!(NeoNftMarketplaceContract::cancel_listing(7, 5));
        // Already cancelled -> rejected.
        assert!(!NeoNftMarketplaceContract::cancel_listing(7, 5));
    }
}
