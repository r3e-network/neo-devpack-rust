use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "TimelockVault"
}"#
);

#[neo_contract]
pub struct TimelockVaultContract;

#[neo_contract]
impl TimelockVaultContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn queue_release(beneficiary: i64, amount: i64, unlock_time: i64) -> bool {
        beneficiary > 0 && amount > 0 && unlock_time > 0
    }

    #[neo_method(safe)]
    pub fn is_mature(unlock_time: i64, current_time: i64) -> bool {
        current_time >= unlock_time
    }

    #[neo_method]
    pub fn release(beneficiary: i64, amount: i64, unlock_time: i64, current_time: i64) -> bool {
        beneficiary > 0 && amount > 0 && unlock_time > 0 && current_time >= unlock_time
    }
}

impl Default for TimelockVaultContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::TimelockVaultContract;

    #[test]
    fn queue_release_requires_positive_values() {
        assert!(TimelockVaultContract::queue_release(1, 100, 10));
        assert!(!TimelockVaultContract::queue_release(0, 100, 10));
        assert!(!TimelockVaultContract::queue_release(1, 0, 10));
    }

    #[test]
    fn maturity_and_release_follow_time_guardrails() {
        assert!(TimelockVaultContract::is_mature(10, 10));
        assert!(!TimelockVaultContract::is_mature(11, 10));

        assert!(TimelockVaultContract::release(1, 100, 10, 10));
        assert!(!TimelockVaultContract::release(1, 100, 10, 9));
    }
}
