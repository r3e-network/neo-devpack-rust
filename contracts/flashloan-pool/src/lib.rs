use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "FlashLoanPool"
}"#
);

const AVAILABLE_LIQUIDITY: i64 = 1_000_000;
const FEE_BPS: i64 = 9;
const BPS_DENOMINATOR: i64 = 10_000;

#[neo_contract]
pub struct FlashLoanPoolContract;

#[neo_contract]
impl FlashLoanPoolContract {
    pub fn new() -> Self {
        Self
    }

    fn flash_fee_internal(amount: i64) -> i64 {
        if amount <= 0 {
            return 0;
        }
        (amount * FEE_BPS) / BPS_DENOMINATOR
    }

    #[neo_method(safe)]
    pub fn max_flash_loan() -> i64 {
        AVAILABLE_LIQUIDITY
    }

    #[neo_method(safe)]
    pub fn flash_fee(amount: i64) -> i64 {
        Self::flash_fee_internal(amount)
    }

    #[neo_method]
    pub fn flash_loan(borrower: i64, amount: i64) -> i64 {
        if borrower <= 0 || amount <= 0 || amount > AVAILABLE_LIQUIDITY {
            return 0;
        }

        Self::flash_fee_internal(amount)
    }

    #[neo_method]
    pub fn repay(amount: i64, repaid_amount: i64) -> bool {
        if amount <= 0 || amount > AVAILABLE_LIQUIDITY {
            return false;
        }

        let required = match amount.checked_add(Self::flash_fee_internal(amount)) {
            Some(value) => value,
            None => return false,
        };
        repaid_amount >= required
    }
}

impl Default for FlashLoanPoolContract {
    fn default() -> Self {
        Self::new()
    }
}
