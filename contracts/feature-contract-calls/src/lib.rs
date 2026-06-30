// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: the cross-contract-call surface, the native
//! contract hash constants/helpers, and `NeoError` / `NeoResult`.
//!
//! ## wasm32 ceiling for cross-calls
//!
//! On `wasm32` the cross-call path emits the correct `SYSCALL
//! System.Contract.Call` (the translator bridges the `neo_contract_call`
//! import), but the call arguments and flags are not yet marshalled across the
//! boundary and the decoded return is `NeoValue::Null`. These methods are
//! therefore written to exercise the **emission / compile path** and to verify
//! the native hash constants — they never depend on a decoded cross-call
//! result, returning a length or sentinel instead. This is the honest current
//! ceiling.

use neo_devpack::prelude::*;
use neo_devpack::NeoVMSyscall;

neo_manifest_overlay!(r#"{ "name": "FeatureContractCalls" }"#);
// Cross-calls require a permission entry; "*"/"*" is the broadest.
neo_permission!("*", ["*"]);

#[neo_contract]
pub struct ContractCallsContract;

#[neo_contract]
impl ContractCallsContract {
    pub fn new() -> Self {
        Self
    }

    /// `NeoVMSyscall::contract_call` to GAS `symbol` + `gas_contract_hash()`.
    /// Returns 0 (Null on wasm32) or -1 on error — exercises the call path.
    #[neo_method(safe)]
    pub fn call_gas_symbol() -> i64 {
        let r = NeoVMSyscall::contract_call(
            &gas_contract_hash(),
            &NeoString::from_str("symbol"),
            &NeoInteger::new(0x0F),
            &NeoArray::new(),
        );
        match r {
            Ok(v) => v.is_null() as i64, // 1 on wasm32 (Null), proves the path ran
            Err(_) => -1,
        }
    }

    /// `call_typed::<NeoValue>` free helper against NEO `decimals`.
    #[neo_method(safe)]
    pub fn call_typed_decimals() -> NeoResult<NeoInteger> {
        let v: NeoValue = call_typed::<NeoValue>(
            &neo_contract_hash(),
            "decimals",
            &[],
            &NeoInteger::new(0x0F),
        )?;
        Ok(NeoInteger::new(v.is_null() as i64))
    }

    /// `DefaultContractCaller::call_raw` (the `ContractCaller` trait method).
    #[neo_method(safe)]
    pub fn caller_raw() -> NeoResult<NeoInteger> {
        let v = DefaultContractCaller.call_raw(
            &gas_contract_hash(),
            "balanceOf",
            &[NeoValue::from(NeoByteString::from_slice(&[0u8; 20]))],
            &NeoInteger::new(0x0F),
        )?;
        Ok(NeoInteger::new(v.is_null() as i64))
    }

    /// `ContractCaller::call_typed` (trait default method, via the default
    /// caller).
    #[neo_method(safe)]
    pub fn caller_typed() -> NeoResult<NeoInteger> {
        let v: NeoValue = DefaultContractCaller.call_typed::<NeoValue>(
            &neo_contract_hash(),
            "symbol",
            &[],
            &NeoInteger::new(0x0F),
        )?;
        Ok(NeoInteger::new(v.is_null() as i64))
    }

    /// `NeoVMSyscall::contract_call_native` by native id (call-path only).
    #[neo_method(safe)]
    pub fn call_native() -> NeoResult<NeoInteger> {
        let v = NeoVMSyscall::contract_call_native(&NeoInteger::new(0))?;
        Ok(NeoInteger::new(v.is_null() as i64))
    }

    /// Reference every native-contract hash: the 11 `*_CONTRACT_LE` byte
    /// constants and the 11 `*_hash()` helpers. Sums their lengths
    /// (11*20 + 11*20 = 440) — proves each symbol exists and is 20 bytes.
    #[neo_method(safe)]
    pub fn all_hashes() -> i64 {
        let consts_len = (NEO_CONTRACT_LE.len()
            + GAS_CONTRACT_LE.len()
            + CONTRACT_MANAGEMENT_LE.len()
            + POLICY_CONTRACT_LE.len()
            + ORACLE_CONTRACT_LE.len()
            + ROLE_MANAGEMENT_LE.len()
            + NOTARY_CONTRACT_LE.len()
            + TREASURY_CONTRACT_LE.len()
            + LEDGER_CONTRACT_LE.len()
            + CRYPTO_LIB_LE.len()
            + STD_LIB_LE.len()) as i64;
        let helpers_len = (neo_contract_hash().len()
            + gas_contract_hash().len()
            + contract_management_hash().len()
            + policy_contract_hash().len()
            + oracle_contract_hash().len()
            + role_management_hash().len()
            + notary_contract_hash().len()
            + treasury_contract_hash().len()
            + ledger_contract_hash().len()
            + crypto_lib_hash().len()
            + std_lib_hash().len()) as i64;
        consts_len + helpers_len
    }

    /// `NeoError` / `NeoResult`: success and error paths, plus the
    /// `NeoError::InvalidOperation` variant.
    #[neo_method(safe)]
    pub fn err_path(x: i64) -> NeoResult<NeoInteger> {
        if x < 0 {
            Err(NeoError::InvalidOperation)
        } else if x == 0 {
            Err(NeoError::new("zero is not allowed"))
        } else {
            Ok(NeoInteger::new(x.wrapping_mul(2)))
        }
    }
}
