// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! L6: cross-call wrapper contract.
//!
//! Demonstrates a contract that calls another contract's method
//! via `System.Contract.Call`. In production, the host's NeoVM
//! dispatches the SYSCALL emitted by the translator.
//!
//! The contract holds a single hardcoded target hash (a
//! placeholder; deploy-time tooling can fill in the real one
//! via the L8 chain-state lookup helper).

use neo_devpack::prelude::*;
use neo_devpack::NeoVMSyscall;

neo_manifest_overlay!(
    r#"{
    "name": "CrossCallWrapper"
}"#
);

const TARGET_CONTRACT_HASH: [u8; 20] = [0u8; 20];

#[neo_contract]
pub struct CrossCallWrapperContract;

#[neo_contract]
impl CrossCallWrapperContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(safe)]
    pub fn total_supply_of_target() -> i64 {
        let script_hash = NeoByteString::from_slice(&TARGET_CONTRACT_HASH);
        let method = NeoString::from_str("totalSupply");
        let call_flags = NeoInteger::new(0x0F);
        let args: NeoArray<NeoValue> = NeoArray::new();
        let result = NeoVMSyscall::contract_call(
            &script_hash,
            &method,
            &call_flags,
            &args,
        )
        .expect("cross-call should succeed");
        match result {
            NeoValue::Integer(i) => i.try_as_i64().unwrap_or(0),
            _ => 0,
        }
    }

    #[neo_method(safe)]
    pub fn balance_of_target(account: i64) -> i64 {
        let script_hash = NeoByteString::from_slice(&TARGET_CONTRACT_HASH);
        let method = NeoString::from_str("balanceOf");
        let call_flags = NeoInteger::new(0x05);
        let mut args: NeoArray<NeoValue> = NeoArray::new();
        args.push(NeoValue::Integer(NeoInteger::new(account)));
        let result = NeoVMSyscall::contract_call(
            &script_hash,
            &method,
            &call_flags,
            &args,
        )
        .expect("cross-call should succeed");
        match result {
            NeoValue::Integer(i) => i.try_as_i64().unwrap_or(0),
            _ => 0,
        }
    }
}

impl Default for CrossCallWrapperContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn wrapper_compiles() {
        let _ = super::CrossCallWrapperContract::new();
    }
}
