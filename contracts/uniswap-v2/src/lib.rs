use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "UniswapV2Router"
}"#
);

const RESERVE_0: i64 = 1_000_000;
const RESERVE_1: i64 = 500_000;
const FEE_NUMERATOR: i64 = 997;
const FEE_DENOMINATOR: i64 = 1_000;

#[neo_contract]
pub struct UniswapV2RouterContract;

#[neo_contract]
impl UniswapV2RouterContract {
    pub fn new() -> Self {
        Self
    }

    fn amount_out_internal(amount_in: i64) -> i64 {
        if amount_in <= 0 {
            return 0;
        }

        let amount_in_with_fee = amount_in * FEE_NUMERATOR;
        let numerator = amount_in_with_fee * RESERVE_1;
        let denominator = RESERVE_0 * FEE_DENOMINATOR + amount_in_with_fee;
        if denominator <= 0 {
            0
        } else {
            numerator / denominator
        }
    }

    #[neo_method]
    pub fn add_liquidity(amount_0: i64, amount_1: i64) -> bool {
        if amount_0 <= 0 || amount_1 <= 0 {
            return false;
        }

        let lhs = amount_0 * RESERVE_1;
        let rhs = amount_1 * RESERVE_0;
        let delta = if lhs > rhs { lhs - rhs } else { rhs - lhs };
        delta <= 50_000
    }

    #[neo_method(safe)]
    pub fn get_reserves() -> i64 {
        (RESERVE_0 << 32) | (RESERVE_1 & 0xFFFF_FFFF)
    }

    #[neo_method(safe)]
    pub fn quote(amount_in: i64) -> i64 {
        Self::amount_out_internal(amount_in)
    }

    #[neo_method]
    pub fn swap_exact_tokens_for_tokens(amount_in: i64, min_out: i64) -> i64 {
        let out = Self::amount_out_internal(amount_in);
        if out > 0 && out >= min_out {
            out
        } else {
            0
        }
    }
}

impl Default for UniswapV2RouterContract {
    fn default() -> Self {
        Self::new()
    }
}
