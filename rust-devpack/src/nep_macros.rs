// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! NEP standard-library macros (L5).
//!
//! These declarative macros were intended to emit the canonical NEP method
//! surface + Transfer event for a standard in one invocation.
//!
//! ## Status: unavailable on the current wasm32 export ABI
//!
//! A compliant NEP-17 / NEP-11 contract needs string method returns
//! (`symbol`, `name`) and Hash160 (`ByteString`) account/token-id
//! parameters. The `#[neo_method]` auto-export wrappers currently marshal
//! **scalar integer/boolean types only** across the wasm32 → NeoVM
//! boundary, so these macros cannot generate a working, standards-compliant
//! contract yet.
//!
//! Before v0.13.2 the macros emitted those signatures but the export
//! collector silently dropped every method (it matched the `#[neo_method]`
//! attribute by bare ident and missed the macro-emitted qualified
//! `#[$crate::neo_method]` form), so contracts built with them deployed
//! with **no callable methods**. As of v0.13.2 the attribute matching is
//! fixed and the macros fail to compile with actionable guidance instead of
//! producing a silently-broken contract.
//!
//! ## What to do instead
//!
//! Write the contract by hand with `#[neo_contract]` + `#[neo_method]`
//! using integer account ids — the same pattern every working example in
//! this repo uses. See `contracts/nep17-token` and `contracts/nep11-nft`
//! for complete, exported, callable token contracts.

/// NEP-17 (fungible token) standard macro.
///
/// **Currently unavailable on the wasm32 export ABI.** A compliant NEP-17
/// contract needs a `symbol()` method that returns a string and
/// `balanceOf`/`transfer` methods that take Hash160 (`ByteString`)
/// accounts. The `#[neo_method]` auto-export wrappers marshal scalar
/// integer/boolean types only, so this macro cannot emit a working,
/// standards-compliant contract yet. Any invocation therefore fails to
/// compile with guidance rather than silently exporting no methods (the
/// behaviour before v0.13.2).
///
/// Until the typed-parameter export ABI lands, write the contract by hand
/// with `#[neo_contract]` + `#[neo_method]` using integer account ids —
/// see `contracts/nep17-token` for a complete working example.
#[macro_export]
macro_rules! nep17 {
    ($($tokens:tt)*) => {
        ::core::compile_error!(
            "`nep17!` cannot auto-generate a NEP-17 contract on the current wasm32 export \
             ABI: `symbol()` must return a string and `balanceOf`/`transfer` take Hash160 \
             (ByteString) accounts, but exported `#[neo_method]` wrappers marshal scalar \
             integer/boolean types only. Write the contract by hand with `#[neo_contract]` \
             + `#[neo_method]` using integer account ids until the typed-parameter export \
             ABI is available — see `contracts/nep17-token` for a complete working example."
        );
    };
}

/// NEP-11 (non-fungible token) standard macro.
///
/// **Currently unavailable on the wasm32 export ABI**, for the same reason
/// as [`nep17!`]: the NEP-11 surface (`symbol`/`name` string returns,
/// Hash160/ByteString accounts and token ids) cannot be marshalled through
/// the scalar-only `#[neo_method]` export wrappers. Any invocation fails to
/// compile with guidance rather than silently exporting no methods.
///
/// Write the contract by hand with `#[neo_contract]` + `#[neo_method]`
/// until the typed-parameter export ABI lands — see `contracts/nep11-nft`.
#[macro_export]
macro_rules! nep11 {
    ($($tokens:tt)*) => {
        ::core::compile_error!(
            "`nep11!` cannot auto-generate a NEP-11 contract on the current wasm32 export \
             ABI: the NEP-11 method surface needs string (`symbol`/`name`) returns and \
             Hash160 (ByteString) accounts/token ids, but exported `#[neo_method]` wrappers \
             marshal scalar integer/boolean types only. Write the contract by hand with \
             `#[neo_contract]` + `#[neo_method]` until the typed-parameter export ABI is \
             available — see `contracts/nep11-nft` for a complete working example."
        );
    };
}

#[cfg(test)]
mod tests {
    // The `nep17!`/`nep11!` macros currently expand to `compile_error!`
    // (see the module docs), so they cannot be invoked in a passing test.
    // The working token pattern they point to is exercised by the
    // hand-written `contracts/nep17-token` and `contracts/nep11-nft`
    // examples (built to wasm32 in CI). Here we only confirm the
    // `macro_rules!` definitions are syntactically valid by loading the
    // module.
    #[test]
    fn nep_macros_module_loads() {}
}
