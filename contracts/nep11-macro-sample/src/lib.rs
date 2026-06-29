// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Minimal NEP-11-style NFT using the working export pattern.
//!
//! The `nep11!` declarative macro is currently unavailable on the wasm32
//! export ABI (string `symbol`/`name` returns and Hash160 `ByteString`
//! accounts/token ids cannot be marshalled through the scalar-only
//! `#[neo_method]` wrappers; see `neo_devpack::nep_macros`). This sample
//! shows the pattern that compiles and exports today, mirroring
//! `contracts/nep11-nft`.

use neo_devpack::prelude::*;

#[neo_contract]
pub struct MacroNep11Sample;

#[neo_contract]
impl MacroNep11Sample {
    pub fn new() -> Self {
        Self
    }

    /// NFTs are indivisible (NEP-11 `decimals` is always 0).
    #[neo_method(safe)]
    pub fn decimals() -> u8 {
        0
    }

    /// Total number of tokens in existence (NEP-11 `totalSupply`).
    #[neo_method(safe)]
    pub fn total_supply() -> i64 {
        0
    }

    /// Number of tokens owned by an account, keyed by an integer id.
    #[neo_method(safe)]
    pub fn balance_of(_account: i64) -> i64 {
        0
    }

    /// Transfer a token (identified by an integer id) to an account.
    #[neo_method]
    pub fn transfer(_to: i64, _token_id: i64) -> bool {
        false
    }
}

impl Default for MacroNep11Sample {
    fn default() -> Self {
        Self::new()
    }
}
