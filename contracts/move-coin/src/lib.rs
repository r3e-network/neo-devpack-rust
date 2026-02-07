use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "MoveCoin"
}"#
);

const TOTAL_SUPPLY: i64 = 1_000_000;

#[neo_contract]
pub struct MoveCoinContract;

#[neo_contract]
impl MoveCoinContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(name = "total_supply")]
    pub fn total_supply() -> i64 {
        TOTAL_SUPPLY
    }

    #[neo_method(name = "has_coin")]
    pub fn has_coin(account: i64) -> bool {
        account > 0
    }

    #[neo_method]
    pub fn mint(account: i64, amount: i64) -> bool {
        account > 0 && amount > 0
    }

    #[neo_method]
    pub fn burn(account: i64, amount: i64) -> bool {
        account > 0 && amount > 0
    }

    #[neo_method]
    pub fn transfer(from: i64, to: i64, amount: i64) -> bool {
        from > 0 && to > 0 && from != to && amount > 0
    }

    #[neo_method]
    pub fn balance(account: i64) -> i64 {
        if account > 0 {
            1_000
        } else {
            0
        }
    }
}

impl Default for MoveCoinContract {
    fn default() -> Self {
        Self::new()
    }
}
