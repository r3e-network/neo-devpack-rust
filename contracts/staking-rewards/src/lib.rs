// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "StakingRewards"
}"#
);

const APR_BPS: i64 = 1_200;
const BPS_DENOMINATOR: i64 = 10_000;
const DAYS_PER_YEAR: i64 = 365;
const MAX_DAYS: i64 = 3_650;
const MAX_PREVIEW_AMOUNT: i64 = 1_000_000_000_000;

fn storage_put_i64(staker: i64, value: i64) -> bool {
    RawStorage::put_i64_key(staker, value);
    true
}

fn storage_get_i64(staker: i64) -> i64 {
    RawStorage::get_i64_key_or_zero(staker)
}

fn ensure_witness_i64(staker: i64) -> bool {
    NeoRuntime::check_witness_i64(staker)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

// Events
#[neo_event]
pub struct Staked {
    pub staker: i64,
    pub amount: i64,
}

#[neo_event]
pub struct Unstaked {
    pub staker: i64,
    pub amount: i64,
}

#[neo_event]
pub struct RewardClaimed {
    pub staker: i64,
    pub reward: i64,
}

#[neo_contract]
pub struct StakingRewardsContract;

#[neo_contract]
impl StakingRewardsContract {
    pub fn new() -> Self {
        Self
    }

    fn preview_reward_internal(amount: i64, days_staked: i64) -> i64 {
        if amount <= 0 || amount > MAX_PREVIEW_AMOUNT || days_staked <= 0 || days_staked > MAX_DAYS
        {
            return 0;
        }

        let amount_days = match amount.checked_mul(days_staked) {
            Some(v) => v,
            None => return 0,
        };
        let scaled = match amount_days.checked_mul(APR_BPS) {
            Some(v) => v,
            None => return 0,
        };
        scaled / (BPS_DENOMINATOR * DAYS_PER_YEAR)
    }

    #[neo_method]
    pub fn stake(staker: i64, amount: i64) -> bool {
        if amount <= 0 || staker == 0 {
            return false;
        }
        if !ensure_witness_i64(staker) {
            return false;
        }
        let current = storage_get_i64(staker);
        if amount > i64::MAX - current {
            return false;
        }
        let new_total = current + amount;
        storage_put_i64(staker, new_total);
        let _ = (Staked { staker, amount }).emit();
        true
    }

    #[neo_method]
    pub fn unstake(staker: i64, amount: i64) -> bool {
        if amount <= 0 || staker == 0 {
            return false;
        }
        if !ensure_witness_i64(staker) {
            return false;
        }
        let current = storage_get_i64(staker);
        if current < amount {
            return false;
        }
        storage_put_i64(staker, current - amount);
        let _ = (Unstaked { staker, amount }).emit();
        true
    }

    #[neo_method(safe)]
    pub fn preview_reward(amount: i64, days_staked: i64) -> i64 {
        Self::preview_reward_internal(amount, days_staked)
    }

    #[neo_method]
    pub fn claim(staker: i64, amount: i64, days_staked: i64) -> i64 {
        if staker == 0 {
            return 0;
        }
        if !ensure_witness_i64(staker) {
            return 0;
        }
        let reward = Self::preview_reward_internal(amount, days_staked);
        if reward > 0 {
            let _ = (RewardClaimed { staker, reward }).emit();
        }
        reward
    }
}

impl Default for StakingRewardsContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::StakingRewardsContract;

    #[test]
    fn reward_preview_handles_boundaries() {
        assert_eq!(StakingRewardsContract::preview_reward(10_000, 365), 1_200);
        assert_eq!(StakingRewardsContract::preview_reward(10_000, 0), 0);
        assert_eq!(StakingRewardsContract::preview_reward(10_000, 3_651), 0);
    }
}
