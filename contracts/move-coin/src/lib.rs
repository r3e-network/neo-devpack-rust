// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "MoveCoin"
}"#
);

const TOTAL_SUPPLY: i64 = 1_000_000;

#[neo_contract]
pub struct MoveCoinContract;

#[neo_contract]
impl MoveCoinContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(name = "total_supply")]
    pub fn total_supply() -> i64 {
        TOTAL_SUPPLY
    }

    #[neo_method(name = "has_coin")]
    pub fn has_coin(account: i64) -> bool {
        account > 0
    }

    #[neo_method]
    pub fn mint(account: i64, amount: i64) -> bool {
        account > 0 && amount > 0
    }

    #[neo_method]
    pub fn burn(account: i64, amount: i64) -> bool {
        account > 0 && amount > 0
    }

    #[neo_method]
    pub fn transfer(from: i64, to: i64, amount: i64) -> bool {
        from > 0 && to > 0 && from != to && amount > 0
    }

    #[neo_method]
    pub fn balance(account: i64) -> i64 {
        if account > 0 {
            1_000
        } else {
            0
        }
    }
}

impl Default for MoveCoinContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MoveCoinContract;

    #[test]
    fn supply_and_balance_queries_are_stable() {
        assert_eq!(MoveCoinContract::total_supply(), 1_000_000);
        assert_eq!(MoveCoinContract::balance(1), 1_000);
        assert_eq!(MoveCoinContract::balance(0), 0);
    }

    #[test]
    fn account_validations_guard_state_changes() {
        assert!(MoveCoinContract::has_coin(1));
        assert!(!MoveCoinContract::has_coin(0));
        assert!(MoveCoinContract::mint(1, 10));
        assert!(!MoveCoinContract::mint(0, 10));
        assert!(MoveCoinContract::burn(1, 10));
        assert!(!MoveCoinContract::burn(1, 0));
    }

    #[test]
    fn transfer_rejects_invalid_paths() {
        assert!(MoveCoinContract::transfer(1, 2, 1));
        assert!(!MoveCoinContract::transfer(1, 1, 1));
        assert!(!MoveCoinContract::transfer(1, 2, 0));
    }
}
