// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Standard-oriented helpers for Neo N3 contracts.
//!
//! This module provides lightweight, reusable building blocks for standards
//! commonly used by Neo contracts, with a focus on:
//! - NEP-24 royalty calculations/interfaces
//! - NEP-26 upgrade lifecycle helpers

use neo_runtime::NeoContractRuntime;
use neo_types::{NeoByteString, NeoContractManifest, NeoError, NeoInteger, NeoResult};

/// Supported standard labels commonly used in manifest metadata.
pub const NEP17_STANDARD: &str = "NEP-17";
pub const NEP11_STANDARD: &str = "NEP-11";
pub const NEP24_STANDARD: &str = "NEP-24";
pub const NEP26_STANDARD: &str = "NEP-26";

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

/// Canonical supported standards list for token + royalty + lifecycle contracts.
pub fn common_supported_standards() -> Vec<&'static str> {
    vec![
        NEP17_STANDARD,
        NEP11_STANDARD,
        NEP24_STANDARD,
        NEP26_STANDARD,
    ]
}

/// Minimal NEP-24 royalty trait.
///
/// Implementers can return one or more royalty recipients and amounts for the
/// requested token/sale price pair.
pub trait Nep24Royalty {
    fn royalty_info(
        &self,
        token_id: &NeoByteString,
        royalty_token: &NeoByteString,
        sale_price: &NeoInteger,
    ) -> NeoResult<Vec<Nep24RoyaltyRecipient>>;
}

/// NEP-26 lifecycle helper trait.
///
/// This trait offers default wrappers around Neo contract update/destroy
/// operations so contracts can expose consistent lifecycle methods.
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
