// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "ConstantProductAMM"
}"#
);

const RESERVE_X: i64 = 10_000;
const RESERVE_Y: i64 = 5_000;
const FEE_NUMERATOR: i64 = 997;
const FEE_DENOMINATOR: i64 = 1_000;

fn quote_internal(amount_in: i64) -> i64 {
    if amount_in <= 0 {
        return 0;
    }

    let amount_in_with_fee = match amount_in.checked_mul(FEE_NUMERATOR) {
        Some(v) => v,
        None => return 0,
    };
    let numerator = match amount_in_with_fee.checked_mul(RESERVE_Y) {
        Some(v) => v,
        None => return 0,
    };
    let denominator = match (RESERVE_X.checked_mul(FEE_DENOMINATOR))
        .and_then(|v| v.checked_add(amount_in_with_fee))
    {
        Some(v) if v > 0 => v,
        _ => return 0,
    };
    numerator / denominator
}

fn init_internal(initial_x: i64, initial_y: i64) -> bool {
    initial_x > 0 && initial_y > 0
}

fn reserve_pair() -> i64 {
    (RESERVE_X << 32) | (RESERVE_Y & 0xFFFF_FFFF)
}

fn swap_internal(trader: i64, amount_in: i64) -> i64 {
    if trader <= 0 {
        return 0;
    }
    quote_internal(amount_in)
}

#[neo_contract]
pub struct ConstantProductAmmContract;

#[neo_contract]
impl ConstantProductAmmContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn init(initial_x: i64, initial_y: i64) -> bool {
        init_internal(initial_x, initial_y)
    }

    #[neo_method(safe)]
    pub fn get_reserves() -> i64 {
        reserve_pair()
    }

    #[neo_method(safe)]
    pub fn quote(amount_in: i64) -> i64 {
        quote_internal(amount_in)
    }

    #[neo_method]
    pub fn swap(trader: i64, amount_in: i64) -> i64 {
        swap_internal(trader, amount_in)
    }
}

impl Default for ConstantProductAmmContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::ConstantProductAmmContract;

    #[test]
    fn init_requires_positive_reserves() {
        assert!(ConstantProductAmmContract::init(1, 1));
        assert!(!ConstantProductAmmContract::init(0, 1));
        assert!(!ConstantProductAmmContract::init(1, 0));
    }

    #[test]
    fn quote_and_swap_respect_validation() {
        assert_eq!(ConstantProductAmmContract::quote(0), 0);
        assert_eq!(ConstantProductAmmContract::swap(0, 100), 0);

        let out = ConstantProductAmmContract::quote(100);
        assert!(out > 0);
        assert_eq!(ConstantProductAmmContract::swap(1, 100), out);
    }

    #[test]
    fn reserves_are_packed_consistently() {
        let packed = ConstantProductAmmContract::get_reserves();
        assert_eq!(packed >> 32, 10_000);
        assert_eq!(packed & 0xFFFF_FFFF, 5_000);
    }
}
