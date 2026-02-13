use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoNFTMarketplace"
}"#
);

#[neo_contract]
pub struct NeoNftMarketplaceContract;

#[neo_contract]
impl NeoNftMarketplaceContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(name = "createListing")]
    pub fn create_listing(
        seller: i64,
        token_contract: i64,
        token_id: i64,
        payment_token: i64,
        price: i64,
        fee_bps: i64,
        expiry: i64,
        listing_id: i64,
        nonce: i64,
    ) -> bool {
        seller > 0
            && token_contract > 0
            && token_id >= 0
            && payment_token > 0
            && price > 0
            && fee_bps >= 0
            && expiry > 0
            && listing_id >= 0
            && nonce >= 0
    }

    #[neo_method(name = "cancelListing")]
    pub fn cancel_listing(listing_id: i64, caller: i64, nonce: i64) -> bool {
        listing_id >= 0 && caller > 0 && nonce >= 0
    }

    #[neo_method(name = "onNEP11Payment")]
    pub fn on_nep11_payment(from: i64, amount: i64, token_id: i64, data: i64) -> bool {
        from > 0 && amount >= 0 && token_id >= 0 && data >= 0
    }

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(from: i64, amount: i64, data: i64) -> bool {
        from > 0 && amount >= 0 && data >= 0
    }

    #[neo_method(name = "getListing")]
    pub fn get_listing(_listing_id: i64, _unused: i64) {}
}

impl Default for NeoNftMarketplaceContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::NeoNftMarketplaceContract;

    #[test]
    fn create_and_cancel_listing_validate_core_fields() {
        assert!(NeoNftMarketplaceContract::create_listing(
            1, 1, 0, 1, 100, 0, 10, 0, 0
        ));
        assert!(!NeoNftMarketplaceContract::create_listing(
            0, 1, 0, 1, 100, 0, 10, 0, 0
        ));
        assert!(!NeoNftMarketplaceContract::create_listing(
            1, 1, 0, 1, 0, 0, 10, 0, 0
        ));

        assert!(NeoNftMarketplaceContract::cancel_listing(0, 1, 0));
        assert!(!NeoNftMarketplaceContract::cancel_listing(0, 0, 0));
    }

    #[test]
    fn payment_handlers_require_positive_sender() {
        assert!(NeoNftMarketplaceContract::on_nep11_payment(1, 0, 0, 0));
        assert!(!NeoNftMarketplaceContract::on_nep11_payment(0, 0, 0, 0));
        assert!(NeoNftMarketplaceContract::on_nep17_payment(1, 0, 0));
        assert!(!NeoNftMarketplaceContract::on_nep17_payment(0, 0, 0));
    }
}
