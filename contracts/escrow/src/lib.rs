use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoEscrow"
}"#
);

#[neo_contract]
pub struct NeoEscrowContract;

#[neo_contract]
impl NeoEscrowContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn configure(
        escrow_id: i64,
        payer: i64,
        payee: i64,
        arbiter: i64,
        token: i64,
        amount: i64,
        release_height: i64,
        refund_height: i64,
        nonce: i64,
    ) -> bool {
        escrow_id > 0
            && payer > 0
            && payee > 0
            && arbiter > 0
            && token > 0
            && amount > 0
            && release_height >= 0
            && refund_height >= release_height
            && nonce >= 0
    }

    #[neo_method]
    pub fn release(escrow_id: i64, caller: i64) -> bool {
        escrow_id > 0 && caller > 0
    }

    #[neo_method]
    pub fn refund(escrow_id: i64, caller: i64) -> bool {
        escrow_id > 0 && caller > 0
    }

    #[neo_method(name = "getState")]
    pub fn get_state(_escrow_id: i64) {}

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(_from: i64, _amount: i64, _data: i64) {}
}

impl Default for NeoEscrowContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::NeoEscrowContract;

    #[test]
    fn configure_validates_heights_and_amounts() {
        assert!(NeoEscrowContract::configure(1, 1, 2, 3, 4, 100, 10, 20, 0));
        assert!(!NeoEscrowContract::configure(1, 1, 2, 3, 4, 0, 10, 20, 0));
        assert!(!NeoEscrowContract::configure(1, 1, 2, 3, 4, 100, 20, 10, 0));
    }

    #[test]
    fn release_and_refund_require_valid_ids() {
        assert!(NeoEscrowContract::release(1, 1));
        assert!(NeoEscrowContract::refund(1, 1));
        assert!(!NeoEscrowContract::release(0, 1));
        assert!(!NeoEscrowContract::refund(1, 0));
    }
}
