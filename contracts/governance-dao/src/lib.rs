use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoGovernanceDAO"
}"#
);

#[neo_contract]
pub struct NeoGovernanceDaoContract;

#[neo_contract]
impl NeoGovernanceDaoContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn configure(dao_id: i64, owner: i64, token: i64, quorum: i64, voting_period: i64) -> bool {
        dao_id > 0 && owner > 0 && token > 0 && quorum > 0 && voting_period > 0
    }

    #[neo_method]
    pub fn propose(
        dao_id: i64,
        proposer: i64,
        target: i64,
        method_id: i64,
        value: i64,
        vote_start: i64,
        vote_end: i64,
        min_yes: i64,
        min_no: i64,
        min_abstain: i64,
        quorum: i64,
        nonce: i64,
    ) -> bool {
        dao_id > 0
            && proposer > 0
            && target > 0
            && method_id >= 0
            && value >= 0
            && vote_start >= 0
            && vote_end >= vote_start
            && min_yes >= 0
            && min_no >= 0
            && min_abstain >= 0
            && quorum >= 0
            && nonce >= 0
    }

    #[neo_method]
    pub fn vote(dao_id: i64, proposal_id: i64, voter: i64, side: i64, weight: i64) -> bool {
        dao_id > 0 && proposal_id >= 0 && voter > 0 && (0..=2).contains(&side) && weight > 0
    }

    #[neo_method]
    pub fn execute(dao_id: i64) -> bool {
        dao_id > 0
    }

    #[neo_method]
    pub fn unstake(dao_id: i64, staker: i64, amount: i64) -> bool {
        dao_id > 0 && staker > 0 && amount > 0
    }

    #[neo_method(name = "stakeOf")]
    pub fn stake_of(dao_id: i64, staker: i64) -> i64 {
        if dao_id > 0 && staker > 0 {
            1000
        } else {
            0
        }
    }

    #[neo_method(name = "getConfig")]
    pub fn get_config(_dao_id: i64) {}

    #[neo_method(name = "getProposal")]
    pub fn get_proposal(_dao_id: i64, _proposal_id: i64) {}

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(_from: i64, _amount: i64, _data: i64) {}
}

impl Default for NeoGovernanceDaoContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::NeoGovernanceDaoContract;

    #[test]
    fn configure_and_propose_validate_parameters() {
        assert!(NeoGovernanceDaoContract::configure(1, 1, 1, 1, 1));
        assert!(!NeoGovernanceDaoContract::configure(0, 1, 1, 1, 1));

        assert!(NeoGovernanceDaoContract::propose(
            1, 1, 1, 0, 0, 10, 20, 0, 0, 0, 0, 0
        ));
        assert!(!NeoGovernanceDaoContract::propose(
            1, 1, 1, 0, 0, 20, 10, 0, 0, 0, 0, 0
        ));
    }

    #[test]
    fn vote_enforces_side_and_weight() {
        assert!(NeoGovernanceDaoContract::vote(1, 0, 1, 0, 1));
        assert!(NeoGovernanceDaoContract::vote(1, 0, 1, 2, 1));
        assert!(!NeoGovernanceDaoContract::vote(1, 0, 1, 3, 1));
        assert!(!NeoGovernanceDaoContract::vote(1, 0, 1, 1, 0));
    }

    #[test]
    fn stake_lookup_is_deterministic() {
        assert_eq!(NeoGovernanceDaoContract::stake_of(1, 1), 1000);
        assert_eq!(NeoGovernanceDaoContract::stake_of(0, 1), 0);
    }
}
