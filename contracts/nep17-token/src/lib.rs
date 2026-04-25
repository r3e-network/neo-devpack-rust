// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "SampleNEP17",
    "supportedstandards": ["NEP-17"]
}"#
);

const TOTAL_SUPPLY: i64 = 1_000_000;
const HOLDER_ONE: i64 = 1;
const HOLDER_TWO: i64 = 2;

#[cfg(test)]
fn witness_account_hash(account: i64) -> &'static [u8; 20] {
    const ACCOUNT_0: [u8; 20] = [0; 20];
    const ACCOUNT_1: [u8; 20] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    const ACCOUNT_2: [u8; 20] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let bytes = match account {
        1 => &ACCOUNT_1,
        2 => &ACCOUNT_2,
        _ => &ACCOUNT_0,
    };
    bytes
}

fn ensure_witness_i64(account: i64) -> bool {
    NeoRuntime::check_witness_i64(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn init_internal(owner: i64, amount: i64) -> bool {
    owner > 0 && amount > 0
}

fn balance_of_internal(account: i64) -> i64 {
    if account == HOLDER_ONE {
        750_000
    } else if account == HOLDER_TWO {
        250_000
    } else {
        0
    }
}

fn transfer_internal(from: i64, to: i64, amount: i64) -> bool {
    from > 0 && to > 0 && from != to && amount > 0
}

#[neo_contract]
pub struct SampleNep17Contract;

#[neo_contract]
impl SampleNep17Contract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn init(owner: i64, amount: i64) -> bool {
        init_internal(owner, amount) && ensure_witness_i64(owner)
    }

    #[neo_method(safe)]
    pub fn total_supply() -> i64 {
        TOTAL_SUPPLY
    }

    #[neo_method(safe)]
    pub fn balance_of(account: i64) -> i64 {
        balance_of_internal(account)
    }

    #[neo_method]
    pub fn transfer(from: i64, to: i64, amount: i64) -> bool {
        transfer_internal(from, to, amount) && ensure_witness_i64(from)
    }

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(_from: i64, _amount: i64, _data: i64) {}
}

impl Default for SampleNep17Contract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::SampleNep17Contract;
    use neo_devpack::{prelude::NeoByteString, NeoVMSyscall};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn runtime_test_lock() -> MutexGuard<'static, ()> {
        static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        match TEST_LOCK.get_or_init(|| Mutex::new(())).lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn setup_runtime_test(witnesses: &[i64]) -> MutexGuard<'static, ()> {
        let guard = runtime_test_lock();
        NeoVMSyscall::reset_host_state().expect("host syscall state should reset");
        let witness_bytes: Vec<NeoByteString> = witnesses
            .iter()
            .map(|account| NeoByteString::from_slice(super::witness_account_hash(*account)))
            .collect();
        NeoVMSyscall::set_active_witnesses(&witness_bytes).expect("active witnesses should update");
        guard
    }

    #[test]
    fn init_and_supply_paths_match_contract_rules() {
        let _guard = setup_runtime_test(&[1]);
        assert!(SampleNep17Contract::init(1, 1));
        assert!(!SampleNep17Contract::init(0, 1));
        assert_eq!(SampleNep17Contract::total_supply(), 1_000_000);
    }

    #[test]
    fn balance_distribution_is_deterministic() {
        assert_eq!(SampleNep17Contract::balance_of(1), 750_000);
        assert_eq!(SampleNep17Contract::balance_of(2), 250_000);
        assert_eq!(SampleNep17Contract::balance_of(3), 0);
    }

    #[test]
    fn transfer_rejects_invalid_paths() {
        let _guard = setup_runtime_test(&[1]);
        assert!(SampleNep17Contract::transfer(1, 2, 1));
        assert!(!SampleNep17Contract::transfer(1, 1, 1));
        assert!(!SampleNep17Contract::transfer(0, 2, 1));
        assert!(!SampleNep17Contract::transfer(1, 2, 0));
        assert!(!SampleNep17Contract::transfer(2, 1, 1));
    }
}
