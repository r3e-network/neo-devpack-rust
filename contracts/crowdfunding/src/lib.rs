use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoCrowdfund"
}"#
);

#[neo_contract]
pub struct NeoCrowdfundContract;

#[neo_contract]
impl NeoCrowdfundContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn configure(
        campaign_id: i64,
        owner: i64,
        token: i64,
        goal: i64,
        deadline: i64,
        min_contribution: i64,
    ) -> bool {
        campaign_id > 0
            && owner > 0
            && token > 0
            && goal > 0
            && deadline > 0
            && min_contribution > 0
    }

    #[neo_method(name = "contributionOf")]
    pub fn contribution_of(campaign_id: i64, contributor: i64) -> i64 {
        if campaign_id > 0 && contributor > 0 {
            100
        } else {
            0
        }
    }

    #[neo_method]
    pub fn finalize(campaign_id: i64) -> bool {
        campaign_id > 0
    }

    #[neo_method(name = "claimRefund")]
    pub fn claim_refund(campaign_id: i64, contributor: i64) -> bool {
        campaign_id > 0 && contributor > 0
    }

    #[neo_method(name = "getCampaign")]
    pub fn get_campaign(_campaign_id: i64) {}

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(_from: i64, _amount: i64, _data: i64) {}
}

impl Default for NeoCrowdfundContract {
    fn default() -> Self {
        Self::new()
    }
}
