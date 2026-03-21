// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Reusable Neo N3 standard traits/constants for contract authors.

use neo_runtime::NeoContractRuntime;
use neo_types::{
    NeoArray, NeoBoolean, NeoByteString, NeoContractManifest, NeoError, NeoInteger, NeoResult,
    NeoString, NeoValue,
};

pub const NEP_11: &str = "NEP-11";
pub const NEP_17: &str = "NEP-17";
pub const NEP_22: &str = "NEP-22";
pub const NEP_24: &str = "NEP-24";
pub const NEP_26: &str = "NEP-26";
pub const NEP_27: &str = "NEP-27";
pub const NEP_29: &str = "NEP-29";
pub const NEP_30: &str = "NEP-30";
pub const NEP_31: &str = "NEP-31";

// Backward-compatible aliases.
pub const NEP11_STANDARD: &str = NEP_11;
pub const NEP17_STANDARD: &str = NEP_17;
pub const NEP22_STANDARD: &str = NEP_22;
pub const NEP24_STANDARD: &str = NEP_24;
pub const NEP26_STANDARD: &str = NEP_26;
pub const NEP27_STANDARD: &str = NEP_27;
pub const NEP29_STANDARD: &str = NEP_29;
pub const NEP30_STANDARD: &str = NEP_30;
pub const NEP31_STANDARD: &str = NEP_31;

pub const LIFECYCLE_STANDARDS: &[&str] = &[NEP_22, NEP_29, NEP_30, NEP_31];
pub const CALLBACK_STANDARDS: &[&str] = &[NEP_26, NEP_27];

/// Basis-point denominator used by royalty calculations (`10000 == 100%`).
pub const NEP_BPS_DENOMINATOR: u16 = 10_000;

/// A single NEP-24 royalty payout entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nep24RoyaltyRecipient {
    pub recipient: NeoByteString,
    pub amount: NeoInteger,
}

/// Computes a royalty amount from sale price and basis points.
pub fn compute_bps_royalty(sale_price: &NeoInteger, bps: u16) -> NeoResult<NeoInteger> {
    if bps > NEP_BPS_DENOMINATOR {
        return Err(NeoError::new("bps cannot exceed 10000"));
    }
    let numerator = sale_price.clone() * NeoInteger::from(u32::from(bps));
    Ok(numerator / NeoInteger::from(u32::from(NEP_BPS_DENOMINATOR)))
}

/// Common standards list for token + callback + lifecycle contracts.
pub fn common_supported_standards() -> Vec<&'static str> {
    vec![
        NEP17_STANDARD,
        NEP11_STANDARD,
        NEP24_STANDARD,
        NEP26_STANDARD,
        NEP27_STANDARD,
        NEP22_STANDARD,
        NEP29_STANDARD,
        NEP30_STANDARD,
        NEP31_STANDARD,
    ]
}

/// NEP-17 fungible token standard trait.
///
/// Contracts implementing this trait are compliant with the NEP-17 token standard,
/// which is the canonical fungible token interface on Neo N3.
pub trait Nep17Token {
    /// Returns the token symbol (e.g. "NEO", "GAS").
    fn symbol(&self) -> NeoResult<NeoString>;

    /// Returns the number of decimals the token uses.
    fn decimals(&self) -> NeoResult<u8>;

    /// Returns the total token supply.
    fn total_supply(&self) -> NeoResult<NeoInteger>;

    /// Returns the token balance of the given account.
    fn balance_of(&self, account: &NeoByteString) -> NeoResult<NeoInteger>;

    /// Transfers `amount` tokens from `from` to `to`.
    ///
    /// The implementation MUST:
    /// - Verify `from` witness via `check_witness`
    /// - Return `false` if the balance is insufficient
    /// - Fire a `Transfer` event on success
    /// - Call `onNEP17Payment` on `to` if it is a deployed contract
    fn transfer(
        &self,
        from: &NeoByteString,
        to: &NeoByteString,
        amount: &NeoInteger,
        data: &NeoValue,
    ) -> NeoResult<bool>;
}

/// NEP-11 non-fungible token standard trait.
///
/// Contracts implementing this trait are compliant with the NEP-11 NFT standard on Neo N3.
pub trait Nep11Token {
    /// Returns the token symbol.
    fn symbol(&self) -> NeoResult<NeoString>;

    /// Returns the number of decimals (0 for indivisible NFTs).
    fn decimals(&self) -> NeoResult<u8>;

    /// Returns the total supply of issued tokens.
    fn total_supply(&self) -> NeoResult<NeoInteger>;

    /// Returns the token balance of the given account.
    fn balance_of(&self, account: &NeoByteString) -> NeoResult<NeoInteger>;

    /// Returns an iterator of token IDs owned by the account.
    fn tokens_of(&self, account: &NeoByteString) -> NeoResult<NeoArray<NeoValue>>;

    /// Transfers the NFT identified by `token_id` to `to`.
    fn transfer(
        &self,
        to: &NeoByteString,
        token_id: &NeoByteString,
        data: &NeoValue,
    ) -> NeoResult<bool>;

    /// Returns the owner of the given token.
    fn owner_of(&self, token_id: &NeoByteString) -> NeoResult<NeoByteString>;

    /// Returns properties/metadata for the given token.
    fn properties(&self, token_id: &NeoByteString) -> NeoResult<NeoArray<NeoValue>>;
}

/// Minimal NEP-24 royalty trait.
pub trait Nep24Royalty {
    fn royalty_info(
        &self,
        token_id: &NeoByteString,
        royalty_token: &NeoByteString,
        sale_price: &NeoInteger,
    ) -> NeoResult<Vec<Nep24RoyaltyRecipient>>;
}

/// StackItem-oriented NEP-24 trait for low-level interoperability.
pub trait Nep24RoyaltyStack {
    fn royalty_info_stack(
        &self,
        token_id: NeoByteString,
        royalty_token: NeoByteString,
        sale_price: NeoInteger,
    ) -> NeoResult<NeoArray<NeoValue>>;
}

/// Legacy lifecycle helper wrapping runtime update/destroy calls.
pub trait Nep26Lifecycle {
    fn update_contract(
        &self,
        script_hash: &NeoByteString,
        nef_script: &NeoByteString,
        manifest: &NeoContractManifest,
    ) -> NeoResult<()> {
        NeoContractRuntime::update(script_hash, nef_script, manifest)
    }

    fn destroy_contract(&self, script_hash: &NeoByteString) -> NeoResult<()> {
        NeoContractRuntime::destroy(script_hash)
    }
}

/// NEP-22 update interface.
pub trait Nep22Update {
    fn update(&self, nef_file: NeoByteString, manifest: NeoString, data: NeoValue)
        -> NeoResult<()>;
}

/// NEP-26 NEP-11 payment callback.
pub trait Nep26Receiver {
    fn on_nep11_payment(
        &self,
        from: NeoByteString,
        amount: NeoInteger,
        token_id: NeoByteString,
        data: NeoValue,
    ) -> NeoResult<()>;
}

/// NEP-27 NEP-17 payment callback.
pub trait Nep27Receiver {
    fn on_nep17_payment(
        &self,
        from: NeoByteString,
        amount: NeoInteger,
        data: NeoValue,
    ) -> NeoResult<()>;
}

/// NEP-29 deployment callback.
pub trait Nep29Deploy {
    fn deploy(&self, data: NeoValue, update: NeoBoolean) -> NeoResult<()>;
}

/// NEP-30 verification callback.
pub trait Nep30Verify {
    fn verify(&self) -> NeoResult<NeoBoolean>;
}

/// NEP-31 destroy interface.
pub trait Nep31Destroy {
    fn destroy(&self) -> NeoResult<()>;
}
