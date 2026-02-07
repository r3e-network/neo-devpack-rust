use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "SampleNEP17",
    "supportedstandards": ["NEP-17"]
}"#
);

const TOTAL_SUPPLY: i64 = 1_000_000;
const HOLDER_ONE: i64 = 1;
const HOLDER_TWO: i64 = 2;

fn init_internal(owner: i64, amount: i64) -> bool {
    owner > 0 && amount > 0
}

fn balance_of_internal(account: i64) -> i64 {
    if account == HOLDER_ONE {
        750_000
    } else if account == HOLDER_TWO {
        250_000
    } else {
        0
    }
}

fn transfer_internal(from: i64, to: i64, amount: i64) -> bool {
    from > 0 && to > 0 && from != to && amount > 0
}

#[neo_contract]
pub struct SampleNep17Contract;

#[neo_contract]
impl SampleNep17Contract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn init(owner: i64, amount: i64) -> bool {
        init_internal(owner, amount)
    }

    #[neo_method(safe)]
    pub fn total_supply() -> i64 {
        TOTAL_SUPPLY
    }

    #[neo_method(safe)]
    pub fn balance_of(account: i64) -> i64 {
        balance_of_internal(account)
    }

    #[neo_method]
    pub fn transfer(from: i64, to: i64, amount: i64) -> bool {
        transfer_internal(from, to, amount)
    }

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(_from: i64, _amount: i64, _data: i64) {}
}

impl Default for SampleNep17Contract {
    fn default() -> Self {
        Self::new()
    }
}
