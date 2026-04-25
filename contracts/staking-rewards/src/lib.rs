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

/// Build a fixed-size storage key from a prefix byte and an i64 staker ID.
/// Layout: [prefix_byte | 8 bytes of staker_id in LE] = 9 bytes total.
/// No heap allocation -- uses a stack-allocated array.
fn stake_key(staker: i64) -> [u8; 9] {
    let mut key = [0u8; 9];
    key[0] = b's'; // "s" for stake
    let bytes = staker.to_le_bytes();
    key[1] = bytes[0];
    key[2] = bytes[1];
    key[3] = bytes[2];
    key[4] = bytes[3];
    key[5] = bytes[4];
    key[6] = bytes[5];
    key[7] = bytes[6];
    key[8] = bytes[7];
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
    let slice = data.as_slice();
    if slice.len() != 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(slice);
    Some(i64::from_le_bytes(buf))
}

fn ensure_witness_i64(staker: i64) -> bool {
    NeoRuntime::check_witness_i64(staker)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

// Events
#[neo_event]
pub struct Staked {
    pub staker: NeoInteger,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct Unstaked {
    pub staker: NeoInteger,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct RewardClaimed {
    pub staker: NeoInteger,
    pub reward: NeoInteger,
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

        let amount_days = amount * days_staked;
        (amount_days * APR_BPS) / (BPS_DENOMINATOR * DAYS_PER_YEAR)
    }

    #[neo_method]
    pub fn stake(staker: i64, amount: i64) -> bool {
        if amount <= 0 || staker == 0 {
            return false;
        }
        if !ensure_witness_i64(staker) {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let key = stake_key(staker);
        let current = storage_get_i64(&ctx, &key).unwrap_or(0);
        if amount > i64::MAX - current {
            return false;
        }
        let new_total = current + amount;
        storage_put_i64(&ctx, &key, new_total);
        let _ = (Staked {
            staker: NeoInteger::new(staker),
            amount: NeoInteger::new(amount),
        })
        .emit();
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
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let key = stake_key(staker);
        let current = storage_get_i64(&ctx, &key).unwrap_or(0);
        if current < amount {
            return false;
        }
        storage_put_i64(&ctx, &key, current - amount);
        let _ = (Unstaked {
            staker: NeoInteger::new(staker),
            amount: NeoInteger::new(amount),
        })
        .emit();
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
            let _ = (RewardClaimed {
                staker: NeoInteger::new(staker),
                reward: NeoInteger::new(reward),
            })
            .emit();
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
