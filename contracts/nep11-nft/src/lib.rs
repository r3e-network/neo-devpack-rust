use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "SampleNEP11",
    "supportedstandards": ["NEP-11"]
}"#
);

const TOTAL_SUPPLY: i64 = 1_000;

fn balance_of_internal(owner: i64) -> i64 {
    if owner > 0 {
        1
    } else {
        0
    }
}

fn owner_of_internal(token_id: i64) -> i64 {
    if token_id <= 0 {
        0
    } else {
        (token_id % 10) + 1
    }
}

fn mint_internal(owner: i64, token_id: i64) -> bool {
    owner > 0 && token_id > 0
}

fn transfer_internal(from: i64, to: i64, token_id: i64) -> bool {
    from > 0 && to > 0 && from != to && token_id > 0
}

#[neo_contract]
pub struct SampleNep11Contract;

#[neo_contract]
impl SampleNep11Contract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(safe)]
    pub fn total_supply() -> i64 {
        TOTAL_SUPPLY
    }

    #[neo_method(safe)]
    pub fn balance_of(owner: i64) -> i64 {
        balance_of_internal(owner)
    }

    #[neo_method(safe)]
    pub fn owner_of(token_id: i64) -> i64 {
        owner_of_internal(token_id)
    }

    #[neo_method]
    pub fn mint(owner: i64, token_id: i64) -> bool {
        mint_internal(owner, token_id)
    }

    #[neo_method]
    pub fn transfer(from: i64, to: i64, token_id: i64) -> bool {
        transfer_internal(from, to, token_id)
    }
}

impl Default for SampleNep11Contract {
    fn default() -> Self {
        Self::new()
    }
}
