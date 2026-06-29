// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Minimal NEP-17-style fungible token using the working export pattern.
//!
//! The `nep17!` declarative macro is currently unavailable on the wasm32
//! export ABI (it cannot marshal the string `symbol()` return or Hash160
//! `ByteString` accounts the standard requires; see
//! `neo_devpack::nep_macros`). This sample demonstrates the pattern that
//! actually compiles, exports, and is callable on-chain today: a
//! `#[neo_contract]` impl whose `#[neo_method]`s exchange scalar
//! integer/boolean values across the boundary, with account ids passed as
//! integers. It mirrors `contracts/nep17-token`.

use neo_devpack::prelude::*;

#[neo_contract]
pub struct MacroNep17Sample;

#[neo_contract]
impl MacroNep17Sample {
    pub fn new() -> Self {
        Self
    }

    /// Token precision (NEP-17 `decimals`). Scalar `u8` returns are
    /// carried over the i64 export ABI.
    #[neo_method(safe)]
    pub fn decimals() -> u8 {
        8
    }

    /// Total minted supply (NEP-17 `totalSupply`).
    #[neo_method(safe)]
    pub fn total_supply() -> i64 {
        1_000_000_000_000_000
    }

    /// Balance of an account, keyed by an integer account id.
    #[neo_method(safe)]
    pub fn balance_of(_account: i64) -> i64 {
        0
    }

    /// Transfer `amount` between two integer account ids.
    #[neo_method]
    pub fn transfer(_from: i64, _to: i64, _amount: i64) -> bool {
        false
    }
}

impl Default for MacroNep17Sample {
    fn default() -> Self {
        Self::new()
    }
}
