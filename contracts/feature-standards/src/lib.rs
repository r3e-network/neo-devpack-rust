// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: the NEP standard traits.
//!
//! The NEP traits (`Nep17Token`, `Nep11Token`, `Nep24Royalty`,
//! `Nep27Receiver`, `Nep26Receiver`, `Nep29Deploy`, `Nep30Verify`,
//! `Nep31Destroy`, `Nep22Update`) are plain Rust traits whose methods use
//! `NeoString` / `NeoByteString` / `NeoArray` and so cannot themselves be
//! `#[neo_method]` exports (the export ABI is scalar). A contract instead
//! *implements* them on internal types and exposes scalar wrappers — which is
//! exactly what this sample does. The one directly-exportable NEP helper,
//! `compute_bps_royalty`, plus `NEP_BPS_DENOMINATOR`, are exercised directly.
//!
//! All trait bodies here are pure Rust (no syscalls), so the whole contract
//! compiles to plain wasm. `nep17!` / `nep11!` declarative macros are covered
//! by the dedicated `nep17-macro-sample` / `nep11-macro-sample` contracts.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureStandards" }"#);
neo_supported_standards!(["NEP-17", "NEP-11", "NEP-24"]);

#[neo_contract]
pub struct StandardsContract;

// ---- Fungible token implementing the NEP-17 + lifecycle traits ----------

struct FungibleToken;

impl Nep17Token for FungibleToken {
    fn symbol(&self) -> NeoResult<NeoString> {
        Ok(NeoString::from_str("FEAT"))
    }
    fn decimals(&self) -> NeoResult<u8> {
        Ok(8)
    }
    fn total_supply(&self) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(1_000_000))
    }
    fn balance_of(&self, _account: &NeoByteString) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(42))
    }
    fn transfer(
        &self,
        _from: &NeoByteString,
        _to: &NeoByteString,
        amount: &NeoInteger,
        _data: &NeoValue,
    ) -> NeoResult<bool> {
        Ok(amount.as_i64_saturating() >= 0)
    }
}

impl Nep27Receiver for FungibleToken {
    fn on_nep17_payment(
        &self,
        _from: NeoByteString,
        _amount: NeoInteger,
        _data: NeoValue,
    ) -> NeoResult<()> {
        Ok(())
    }
}

impl Nep29Deploy for FungibleToken {
    fn deploy(&self, _data: NeoValue, _update: NeoBoolean) -> NeoResult<()> {
        Ok(())
    }
}

impl Nep30Verify for FungibleToken {
    fn verify(&self) -> NeoResult<NeoBoolean> {
        Ok(NeoBoolean::TRUE)
    }
}

impl Nep31Destroy for FungibleToken {
    fn destroy(&self) -> NeoResult<()> {
        Ok(())
    }
}

impl Nep22Update for FungibleToken {
    fn update(
        &self,
        _nef_file: NeoByteString,
        _manifest: NeoString,
        _data: NeoValue,
    ) -> NeoResult<()> {
        Ok(())
    }
}

// ---- Non-fungible token implementing NEP-11 + NEP-24 + NEP-26 -----------

struct NonFungibleToken;

impl Nep11Token for NonFungibleToken {
    fn symbol(&self) -> NeoResult<NeoString> {
        Ok(NeoString::from_str("FNFT"))
    }
    fn decimals(&self) -> NeoResult<u8> {
        Ok(0)
    }
    fn total_supply(&self) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(3))
    }
    fn balance_of(&self, _account: &NeoByteString) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(1))
    }
    fn tokens_of(&self, _account: &NeoByteString) -> NeoResult<NeoArray<NeoValue>> {
        let mut a: NeoArray<NeoValue> = NeoArray::new();
        a.push(NeoValue::from(NeoByteString::from_slice(b"tok-1")));
        Ok(a)
    }
    fn transfer(
        &self,
        _to: &NeoByteString,
        _token_id: &NeoByteString,
        _data: &NeoValue,
    ) -> NeoResult<bool> {
        Ok(true)
    }
    fn owner_of(&self, _token_id: &NeoByteString) -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&[7u8; 20]))
    }
    fn properties(&self, _token_id: &NeoByteString) -> NeoResult<NeoArray<NeoValue>> {
        let mut a: NeoArray<NeoValue> = NeoArray::new();
        a.push(NeoValue::from(NeoString::from_str("name")));
        Ok(a)
    }
}

impl Nep24Royalty for NonFungibleToken {
    fn royalty_info(
        &self,
        _token_id: &NeoByteString,
        royalty_token: &NeoByteString,
        sale_price: &NeoInteger,
    ) -> NeoResult<Vec<Nep24RoyaltyRecipient>> {
        // 2.5% royalty (250 bps) to a single recipient.
        let amount = compute_bps_royalty(sale_price, 250)?;
        Ok(vec![Nep24RoyaltyRecipient {
            recipient: royalty_token.clone(),
            amount,
        }])
    }
}

impl Nep26Receiver for NonFungibleToken {
    fn on_nep11_payment(
        &self,
        _from: NeoByteString,
        _amount: NeoInteger,
        _token_id: NeoByteString,
        _data: NeoValue,
    ) -> NeoResult<()> {
        Ok(())
    }
}

#[neo_contract]
impl StandardsContract {
    pub fn new() -> Self {
        Self
    }

    /// The directly-exportable NEP helper: `compute_bps_royalty`.
    #[neo_method(safe)]
    pub fn royalty_bps(price: i64) -> NeoResult<NeoInteger> {
        compute_bps_royalty(&NeoInteger::new(price), 250)
    }

    /// `NEP_BPS_DENOMINATOR` constant (10_000).
    #[neo_method(safe)]
    pub fn bps_denominator() -> i64 {
        NEP_BPS_DENOMINATOR as i64
    }

    /// `Nep30Verify::verify` reduced to a bool.
    #[neo_method(safe)]
    pub fn verify_wrap() -> bool {
        FungibleToken
            .verify()
            .map(|b| b.as_bool())
            .unwrap_or(false)
    }

    /// Exercise the full `Nep17Token` surface and fold to a scalar.
    #[neo_method(safe)]
    pub fn nep17_surface() -> i64 {
        let t = FungibleToken;
        let acct = NeoByteString::from_slice(&[0u8; 20]);
        let mut acc = t.symbol().map(|s| s.len() as i64).unwrap_or(-1);
        acc += t.decimals().map(|d| d as i64).unwrap_or(-1);
        acc += t.total_supply().map(|n| n.as_i64_saturating()).unwrap_or(-1);
        acc += t.balance_of(&acct).map(|n| n.as_i64_saturating()).unwrap_or(-1);
        acc += t
            .transfer(&acct, &acct, &NeoInteger::new(5), &NeoValue::Null)
            .map(|ok| ok as i64)
            .unwrap_or(-1);
        acc
    }

    /// Exercise the full `Nep11Token` surface and fold to a scalar.
    #[neo_method(safe)]
    pub fn nep11_surface() -> i64 {
        let t = NonFungibleToken;
        let acct = NeoByteString::from_slice(&[0u8; 20]);
        let tok = NeoByteString::from_slice(b"tok-1");
        let mut acc = t.symbol().map(|s| s.len() as i64).unwrap_or(-1);
        acc += t.decimals().map(|d| d as i64).unwrap_or(-1);
        acc += t.total_supply().map(|n| n.as_i64_saturating()).unwrap_or(-1);
        acc += t.balance_of(&acct).map(|n| n.as_i64_saturating()).unwrap_or(-1);
        acc += t.tokens_of(&acct).map(|a| a.len() as i64).unwrap_or(-1);
        acc += t
            .transfer(&acct, &tok, &NeoValue::Null)
            .map(|ok| ok as i64)
            .unwrap_or(-1);
        acc += t.owner_of(&tok).map(|h| h.len() as i64).unwrap_or(-1);
        acc += t.properties(&tok).map(|a| a.len() as i64).unwrap_or(-1);
        acc
    }

    /// `Nep24Royalty::royalty_info` → number of recipients + summed amounts.
    #[neo_method(safe)]
    pub fn royalty_surface(price: i64) -> i64 {
        let t = NonFungibleToken;
        let tok = NeoByteString::from_slice(b"tok-1");
        let rt = NeoByteString::from_slice(&[9u8; 20]);
        match t.royalty_info(&tok, &rt, &NeoInteger::new(price)) {
            Ok(recips) => {
                let total: i64 = recips.iter().map(|r| r.amount.as_i64_saturating()).sum();
                (recips.len() as i64).wrapping_mul(1_000_000).wrapping_add(total)
            }
            Err(_) => -1,
        }
    }

    /// Lifecycle/receiver traits (Nep29Deploy / Nep31Destroy / Nep22Update /
    /// Nep27Receiver / Nep26Receiver) — each returns `Ok(())`; count successes.
    #[neo_method(safe)]
    pub fn lifecycle_surface() -> i64 {
        let ft = FungibleToken;
        let nft = NonFungibleToken;
        let acct = NeoByteString::from_slice(&[1u8; 20]);
        let mut ok = 0i64;
        ok += ft.deploy(NeoValue::Null, NeoBoolean::FALSE).is_ok() as i64;
        ok += ft.destroy().is_ok() as i64;
        ok += ft
            .update(NeoByteString::from_slice(b"nef"), NeoString::from_str("m"), NeoValue::Null)
            .is_ok() as i64;
        ok += ft
            .on_nep17_payment(acct.clone(), NeoInteger::new(1), NeoValue::Null)
            .is_ok() as i64;
        ok += nft
            .on_nep11_payment(acct.clone(), NeoInteger::new(1), acct, NeoValue::Null)
            .is_ok() as i64;
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standards_surface() {
        assert_eq!(StandardsContract::bps_denominator(), 10_000);
        // 2.5% of 10_000 = 250.
        assert_eq!(
            StandardsContract::royalty_bps(10_000).unwrap().as_i64_saturating(),
            250
        );
        assert!(StandardsContract::verify_wrap());
        assert!(StandardsContract::nep17_surface() > 0);
        assert!(StandardsContract::nep11_surface() > 0);
        assert_eq!(StandardsContract::lifecycle_surface(), 5);
    }
}
