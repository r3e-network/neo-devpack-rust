// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT
//
// !! ILLUSTRATIVE SAMPLE — NOT AUDITED FOR PRODUCTION !!
// This flashloan pool implements fee math only. It does NOT move tokens, track
// per-loan debt, or enforce atomic same-transaction repayment, and therefore
// must not be deployed as a real lending pool. A production implementation
// additionally requires: borrower witness (added), a debt ledger, and a
// reentrancy guard around the flash callback.

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "FlashLoanPool"
}"#
);

const AVAILABLE_LIQUIDITY: i64 = 1_000_000;
const FEE_BPS: i64 = 9;
const BPS_DENOMINATOR: i64 = 10_000;

#[neo_contract]
pub struct FlashLoanPoolContract;

#[neo_contract]
impl FlashLoanPoolContract {
    pub fn new() -> Self {
        Self
    }

    fn flash_fee_internal(amount: i64) -> i64 {
        if amount <= 0 {
            return 0;
        }
        if amount > i64::MAX / FEE_BPS {
            return 0;
        }
        (amount * FEE_BPS) / BPS_DENOMINATOR
    }

    #[neo_method(safe)]
    pub fn max_flash_loan() -> i64 {
        AVAILABLE_LIQUIDITY
    }

    #[neo_method(safe)]
    pub fn flash_fee(amount: i64) -> i64 {
        Self::flash_fee_internal(amount)
    }

    #[neo_method]
    pub fn flash_loan(borrower: i64, amount: i64) -> i64 {
        if borrower <= 0 || amount <= 0 || amount > AVAILABLE_LIQUIDITY {
            return 0;
        }
        // The borrower must be runtime-witnessed (X20). This sample is
        // fee-math only (no tokens move, no debt ledger, no atomic repay
        // enforcement) — see the module banner below. A real pool must also
        // record per-loan debt and enforce same-transaction repayment.
        if !NeoRuntime::require_witness_i64(borrower) {
            return 0;
        }

        Self::flash_fee_internal(amount)
    }

    #[neo_method]
    pub fn repay(amount: i64, repaid_amount: i64) -> bool {
        if amount <= 0 || amount > AVAILABLE_LIQUIDITY {
            return false;
        }

        let fee = Self::flash_fee_internal(amount);
        if fee > i64::MAX - amount {
            return false;
        }
        let required = amount + fee;
        repaid_amount >= required
    }
}

impl Default for FlashLoanPoolContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::FlashLoanPoolContract;
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
    fn flash_fee_and_capacity_are_consistent() {
        assert_eq!(FlashLoanPoolContract::max_flash_loan(), 1_000_000);
        assert_eq!(FlashLoanPoolContract::flash_fee(0), 0);
        assert_eq!(FlashLoanPoolContract::flash_fee(10_000), 9);
        assert_eq!(FlashLoanPoolContract::flash_fee(i64::MAX), 0);
    }

    #[test]
    fn flash_loan_requires_valid_borrower_and_bounds() {
        // Borrower 1 is witnessed so the positive-path assertion still holds.
        let _g = setup_witnesses(&[1]);
        assert_eq!(FlashLoanPoolContract::flash_loan(1, 10_000), 9);
        assert_eq!(FlashLoanPoolContract::flash_loan(0, 10_000), 0);
        assert_eq!(FlashLoanPoolContract::flash_loan(1, 1_000_001), 0);
    }

    #[test]
    fn flash_loan_requires_borrower_witness() {
        // Borrower 1 not witnessed -> flash_loan returns 0 (X20).
        {
            let _g = setup_witnesses(&[]);
            assert_eq!(FlashLoanPoolContract::flash_loan(1, 10_000), 0);
        }
        // Borrower 1 witnessed -> fee returned.
        {
            let _g = setup_witnesses(&[1]);
            assert_eq!(FlashLoanPoolContract::flash_loan(1, 10_000), 9);
        }
    }

    #[test]
    fn repay_enforces_fee_coverage() {
        assert!(FlashLoanPoolContract::repay(10_000, 10_009));
        assert!(!FlashLoanPoolContract::repay(10_000, 10_008));
        assert!(!FlashLoanPoolContract::repay(0, 0));
    }
}
