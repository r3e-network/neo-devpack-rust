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
