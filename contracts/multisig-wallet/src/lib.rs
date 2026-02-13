use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "SampleMultisig"
}"#
);

#[neo_contract]
pub struct SampleMultisigContract;

#[neo_contract]
impl SampleMultisigContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn configure(owner: i64, signer_count: i64, threshold: i64) -> bool {
        owner > 0 && signer_count > 0 && threshold > 0 && signer_count >= threshold
    }

    #[neo_method]
    pub fn propose(
        wallet_id: i64,
        proposer: i64,
        target: i64,
        method_id: i64,
        amount: i64,
        gas_limit: i64,
        expires_at: i64,
        nonce: i64,
    ) -> bool {
        wallet_id > 0
            && proposer > 0
            && target > 0
            && method_id >= 0
            && amount >= 0
            && gas_limit >= 0
            && expires_at > 0
            && nonce >= 0
    }

    #[neo_method]
    pub fn approve(wallet_id: i64, proposal_id: i64, signer: i64) -> bool {
        wallet_id > 0 && proposal_id >= 0 && signer > 0
    }

    #[neo_method]
    pub fn execute(wallet_id: i64, proposal_id: i64, executor: i64) -> bool {
        wallet_id > 0 && proposal_id >= 0 && executor > 0
    }

    #[neo_method(name = "getConfig")]
    pub fn get_config(_wallet_id: i64) {}
}

impl Default for SampleMultisigContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::SampleMultisigContract;

    #[test]
    fn configure_requires_threshold_not_exceeding_signers() {
        assert!(SampleMultisigContract::configure(1, 3, 2));
        assert!(!SampleMultisigContract::configure(1, 2, 3));
        assert!(!SampleMultisigContract::configure(0, 3, 2));
    }

    #[test]
    fn proposal_flow_validates_required_fields() {
        assert!(SampleMultisigContract::propose(1, 1, 1, 0, 0, 0, 1, 0));
        assert!(!SampleMultisigContract::propose(1, 1, 1, 0, 0, 0, 0, 0));
        assert!(SampleMultisigContract::approve(1, 0, 1));
        assert!(!SampleMultisigContract::approve(0, 0, 1));
        assert!(SampleMultisigContract::execute(1, 0, 1));
        assert!(!SampleMultisigContract::execute(1, 0, 0));
    }
}
