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

#[cfg(test)]
mod tests {
    use super::SampleNep11Contract;

    #[test]
    fn supply_and_read_paths_are_consistent() {
        assert_eq!(SampleNep11Contract::total_supply(), 1_000);
        assert_eq!(SampleNep11Contract::balance_of(1), 1);
        assert_eq!(SampleNep11Contract::balance_of(0), 0);
        assert_eq!(SampleNep11Contract::owner_of(0), 0);
        assert_eq!(SampleNep11Contract::owner_of(12), 3);
    }

    #[test]
    fn mint_and_transfer_validate_inputs() {
        assert!(SampleNep11Contract::mint(1, 1));
        assert!(!SampleNep11Contract::mint(0, 1));
        assert!(SampleNep11Contract::transfer(1, 2, 1));
        assert!(!SampleNep11Contract::transfer(1, 1, 1));
        assert!(!SampleNep11Contract::transfer(1, 2, 0));
    }
}
