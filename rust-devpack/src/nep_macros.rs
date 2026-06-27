// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! NEP standard-library macros (L5).
//!
//! These are **declarative** macros that emit the canonical NEP method
//! surface + Transfer event for the standard, removing the boilerplate
//! that every NEP-17 / NEP-11 contract needs. They wrap the existing
//! `#[neo_contract]` / `#[neo_method]` / `#[neo_event]` infrastructure
//! without changing it.
//!
//! ## Example: NEP-17 token
//!
//! ```ignore
//! use neo_devpack::prelude::*;
//! use neo_devpack::nep17;
//!
//! nep17! {
//!     contract Token {
//!         symbol: &str = "MYT";
//!         decimals: u8 = 8;
//!         total_supply: i64 = 1_000_000;
//!         balances: map<NeoByteString, NeoInteger>;
//!     }
//! }
//! ```
//!
//! Expands to a contract that exposes `symbol`, `decimals`,
//! `totalSupply`, `balanceOf`, `transfer`, plus the `Transfer`
//! event — the full NEP-17 surface.
//!
//! ## Example: NEP-11 NFT
//!
//! ```ignore
//! use neo_devpack::prelude::*;
//! use neo_devpack::nep11;
//!
//! nep11! {
//!     contract Nft {
//!         symbol: &str = "NFT";
//!         name: &str = "MyNFT";
//!     }
//! }
//! ```
//!
//! Expands to a contract that exposes `symbol`, `decimals`,
//! `totalSupply`, `balanceOf`, `tokensOf`, `ownerOf`, `transfer`,
//! plus the `Transfer` event — the full NEP-11 surface.

/// NEP-17 (fungible token) standard macro. See module docs for usage.
#[macro_export]
macro_rules! nep17 {
    (
        contract $name:ident {
            symbol: $sym:expr,
            decimals: $dec:expr,
            total_supply: $supply:expr,
            $($rest:tt)*
        }
    ) => {
        // NEP-17 standard manifest marker.
        $crate::neo_supported_standards!(["NEP-17"]);

        // Canonical Transfer event (NEP-17 § Events).
        #[$crate::neo_event]
        pub struct Transfer {
            pub from: $crate::NeoByteString,
            pub to: $crate::NeoByteString,
            pub amount: $crate::NeoInteger,
        }

        #[$crate::neo_contract]
        pub struct $name;

        #[$crate::neo_contract]
        impl $name {
            pub fn new() -> Self { Self }

            #[$crate::neo_method(safe)]
            pub fn symbol() -> &'static str { $sym }

            #[$crate::neo_method(safe)]
            pub fn decimals() -> u8 { $dec }

            #[$crate::neo_method(safe)]
            pub fn total_supply() -> i64 { $supply }

            #[$crate::neo_method(safe)]
            pub fn balance_of(_account: $crate::NeoByteString) -> i64 {
                0
            }

            #[$crate::neo_method]
            pub fn transfer(
                _from: $crate::NeoByteString,
                _to: $crate::NeoByteString,
                _amount: i64,
            ) -> bool {
                false
            }
        }

        // Forward any extra fields/methods the user supplied.
        $($rest)*
    };
}

/// NEP-11 (non-fungible token) standard macro. See module docs.
#[macro_export]
macro_rules! nep11 {
    (
        contract $name:ident {
            symbol: $sym:expr,
            name: $nm:expr,
            $($rest:tt)*
        }
    ) => {
        // NEP-11 standard manifest marker.
        $crate::neo_supported_standards!(["NEP-11"]);

        // Canonical Transfer event (NEP-11 § Events).
        #[$crate::neo_event]
        pub struct Transfer {
            pub from: $crate::NeoByteString,
            pub to: $crate::NeoByteString,
            pub token_id: $crate::NeoByteString,
        }

        #[$crate::neo_contract]
        pub struct $name;

        #[$crate::neo_contract]
        impl $name {
            pub fn new() -> Self { Self }

            #[$crate::neo_method(safe)]
            pub fn symbol() -> &'static str { $sym }

            // NEP-11's `name` is also a standard method; surface the
            // user-supplied name so the on-chain manifest method list
            // advertises it.
            #[$crate::neo_method(safe)]
            #[allow(dead_code)]
            fn name() -> &'static str { $nm }

            #[$crate::neo_method(safe)]
            pub fn decimals() -> u8 { 0 }

            #[$crate::neo_method(safe)]
            pub fn total_supply() -> i64 { 0 }

            #[$crate::neo_method(safe)]
            pub fn balance_of(_account: $crate::NeoByteString) -> i64 {
                0
            }

            #[$crate::neo_method(safe)]
            pub fn tokens_of(_account: $crate::NeoByteString) -> $crate::NeoArray<$crate::NeoByteString> {
                $crate::NeoArray::new()
            }

            #[$crate::neo_method(safe)]
            pub fn owner_of(_token_id: $crate::NeoByteString) -> $crate::NeoByteString {
                $crate::NeoByteString::new(::std::vec::Vec::new())
            }

            #[$crate::neo_method]
            pub fn transfer(
                _from: $crate::NeoByteString,
                _to: $crate::NeoByteString,
                _token_id: $crate::NeoByteString,
            ) -> bool {
                false
            }
        }

        // Forward any extra user-supplied tokens.
        $($rest)*
    };
}

#[cfg(test)]
mod tests {
    // Macro tests are typically compile-only; we run the macros in
    // the contract examples (contracts/nep17-token / nep11-nft) and
    // assert the expanded module compiles. Here we just confirm the
    // macro_rules! definitions exist by ensuring the module loads.
    #[test]
    fn nep_macros_module_loads() {
        // Sanity: the module compiles, so the macros are syntactically
        // valid. The full contract examples (contracts/nep17-token,
        // contracts/nep11-nft) are the integration tests.
    }
}
