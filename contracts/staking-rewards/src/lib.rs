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

// Storage keys
const STAKE_PREFIX: &[u8] = b"stake:";

fn stake_key(account: &NeoByteString) -> Vec<u8> {
    let mut key = STAKE_PREFIX.to_vec();
    key.extend_from_slice(account.as_slice());
    key
}

fn storage_put_bytes(ctx: &NeoStorageContext, key: &[u8], value: &[u8]) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(value),
    )
    .is_ok()
}

fn storage_get_bytes(ctx: &NeoStorageContext, key: &[u8]) -> Option<Vec<u8>> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    if data.is_empty() {
        return None;
    }
    Some(data.as_slice().to_vec())
}

fn storage_put_i64(ctx: &NeoStorageContext, key: &[u8], value: i64) -> bool {
    storage_put_bytes(ctx, key, &value.to_le_bytes())
}

fn storage_get_i64(ctx: &NeoStorageContext, key: &[u8]) -> Option<i64> {
    let bytes = storage_get_bytes(ctx, key)?;
    if bytes.len() != 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes);
    Some(i64::from_le_bytes(buf))
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    if ptr == 0 || len != 20 {
        return None;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    Some(NeoByteString::from_slice(slice))
}

// Events
#[neo_event]
pub struct Staked {
    pub staker: NeoByteString,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct Unstaked {
    pub staker: NeoByteString,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct RewardClaimed {
    pub staker: NeoByteString,
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
        if amount <= 0 || days_staked <= 0 || days_staked > MAX_DAYS {
            return 0;
        }

        (amount * APR_BPS * days_staked) / (BPS_DENOMINATOR * DAYS_PER_YEAR)
    }

    #[neo_method]
    pub fn stake(staker_ptr: i64, staker_len: i64, amount: i64) -> bool {
        if amount <= 0 {
            return false;
        }
        let staker = match read_address(staker_ptr, staker_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&staker) {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let current = storage_get_i64(&ctx, &stake_key(&staker)).unwrap_or(0);
        storage_put_i64(&ctx, &stake_key(&staker), current + amount);
        let _ = (Staked {
            staker,
            amount: NeoInteger::new(amount),
        })
        .emit();
        true
    }

    #[neo_method]
    pub fn unstake(staker_ptr: i64, staker_len: i64, amount: i64) -> bool {
        if amount <= 0 {
            return false;
        }
        let staker = match read_address(staker_ptr, staker_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&staker) {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let current = storage_get_i64(&ctx, &stake_key(&staker)).unwrap_or(0);
        if current < amount {
            return false;
        }
        storage_put_i64(&ctx, &stake_key(&staker), current - amount);
        let _ = (Unstaked {
            staker,
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
    pub fn claim(staker_ptr: i64, staker_len: i64, amount: i64, days_staked: i64) -> i64 {
        let staker = match read_address(staker_ptr, staker_len) {
            Some(a) => a,
            None => return 0,
        };
        if !ensure_witness(&staker) {
            return 0;
        }
        let reward = Self::preview_reward_internal(amount, days_staked);
        if reward > 0 {
            let _ = (RewardClaimed {
                staker,
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
