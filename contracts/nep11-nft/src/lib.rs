// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "SampleNEP11",
    "supportedstandards": ["NEP-11"]
}"#
);

const TOTAL_SUPPLY: i64 = 1_000;

fn balance_of_internal(owner: i64) -> i64 {
    if owner > 0 {
        1
    } else {
        0
    }
}

fn owner_of_internal(token_id: i64) -> i64 {
    if token_id <= 0 {
        0
    } else {
        (token_id % 10) + 1
    }
}

fn mint_internal(owner: i64, token_id: i64) -> bool {
    owner > 0 && token_id > 0
}

fn transfer_internal(from: i64, to: i64, token_id: i64) -> bool {
    from > 0 && to > 0 && from != to && token_id > 0
}

#[neo_contract]
pub struct SampleNep11Contract;

#[neo_contract]
impl SampleNep11Contract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(safe)]
    pub fn total_supply() -> i64 {
        TOTAL_SUPPLY
    }

    #[neo_method(safe)]
    pub fn balance_of(owner: i64) -> i64 {
        balance_of_internal(owner)
    }

    #[neo_method(safe)]
    pub fn owner_of(token_id: i64) -> i64 {
        owner_of_internal(token_id)
    }

    #[neo_method]
    pub fn mint(owner: i64, token_id: i64) -> bool {
        // Only a witnessed minter/owner may mint (X4: otherwise anyone mints
        // for free). The contract's sample owner is the witnessed caller.
        if !NeoRuntime::require_witness_i64(owner) {
            return false;
        }
        mint_internal(owner, token_id)
    }

    #[neo_method]
    pub fn transfer(from: i64, to: i64, token_id: i64) -> bool {
        // The current owner (`from`) must authorize the transfer; otherwise
        // anyone moves anyone's NFT (X4 token theft).
        if !NeoRuntime::require_witness_i64(from) {
            return false;
        }
        transfer_internal(from, to, token_id)
    }
}

impl Default for SampleNep11Contract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::SampleNep11Contract;
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
    fn supply_and_read_paths_are_consistent() {
        assert_eq!(SampleNep11Contract::total_supply(), 1_000);
        assert_eq!(SampleNep11Contract::balance_of(1), 1);
        assert_eq!(SampleNep11Contract::balance_of(0), 0);
        assert_eq!(SampleNep11Contract::owner_of(0), 0);
        assert_eq!(SampleNep11Contract::owner_of(12), 3);
    }

    #[test]
    fn mint_and_transfer_validate_inputs() {
        // Owner=1 is witnessed so the input-validation assertions still hold.
        let _g = setup_witnesses(&[1]);
        assert!(SampleNep11Contract::mint(1, 1));
        assert!(!SampleNep11Contract::mint(0, 1));
        assert!(SampleNep11Contract::transfer(1, 2, 1));
        assert!(!SampleNep11Contract::transfer(1, 1, 1));
        assert!(!SampleNep11Contract::transfer(1, 2, 0));
    }

    #[test]
    fn mint_and_transfer_require_witness() {
        // Without owner=1 in the witness set, mint and transfer are rejected
        // even with otherwise-valid inputs (X4: free mint / token theft).
        {
            let _g = setup_witnesses(&[]);
            assert!(!SampleNep11Contract::mint(1, 1));
            assert!(!SampleNep11Contract::transfer(1, 2, 1));
        }
        // With owner=1 witnessed, both succeed.
        {
            let _g = setup_witnesses(&[1]);
            assert!(SampleNep11Contract::mint(1, 1));
            assert!(SampleNep11Contract::transfer(1, 2, 1));
        }
    }
}
