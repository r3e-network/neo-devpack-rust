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
    pub fn stake(staker: i64, amount: i64) -> bool {
        staker > 0 && amount > 0
    }

    #[neo_method]
    pub fn unstake(staker: i64, amount: i64) -> bool {
        staker > 0 && amount > 0
    }

    #[neo_method(safe)]
    pub fn preview_reward(amount: i64, days_staked: i64) -> i64 {
        Self::preview_reward_internal(amount, days_staked)
    }

    #[neo_method]
    pub fn claim(staker: i64, amount: i64, days_staked: i64) -> i64 {
        if staker <= 0 {
            return 0;
        }

        Self::preview_reward_internal(amount, days_staked)
    }
}

impl Default for StakingRewardsContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::StakingRewardsContract;

    #[test]
    fn staking_and_unstaking_require_positive_inputs() {
        assert!(StakingRewardsContract::stake(1, 1));
        assert!(!StakingRewardsContract::stake(0, 1));
        assert!(StakingRewardsContract::unstake(1, 1));
        assert!(!StakingRewardsContract::unstake(1, 0));
    }

    #[test]
    fn reward_preview_handles_boundaries() {
        assert_eq!(StakingRewardsContract::preview_reward(10_000, 365), 1_200);
        assert_eq!(StakingRewardsContract::preview_reward(10_000, 0), 0);
        assert_eq!(StakingRewardsContract::preview_reward(10_000, 3_651), 0);
    }

    #[test]
    fn claim_requires_valid_staker() {
        assert_eq!(StakingRewardsContract::claim(1, 10_000, 365), 1_200);
        assert_eq!(StakingRewardsContract::claim(0, 10_000, 365), 0);
    }
}
