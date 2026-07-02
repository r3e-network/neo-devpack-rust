// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Host-mode syscall dispatch: the registry-backed `neovm_syscall` entry
//! point plus the call-flags and registry lookup helpers it shares with the
//! typed wrappers in `wrapper.rs`.

use neo_types::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::storage::*;
// The registry (`SYSCALLS`) and its row type back the host-mode dispatch
// only; wasm32 wrappers call link-time externs directly.
#[cfg(not(target_arch = "wasm32"))]
use crate::syscalls::SYSCALLS;
#[cfg(not(target_arch = "wasm32"))]
use crate::NeoVMSyscallInfo;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) const CALL_FLAGS_VALID_MASK: i32 = 0x0F;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const CALL_FLAGS_READ_STATES: i32 = 0x01;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const CALL_FLAGS_WRITE_STATES: i32 = 0x02;

// Host-mode (non-wasm32) syscall dispatch helpers. On wasm32 the wrappers
// call link-time externs directly, so these registry lookups are unused.
#[cfg(not(target_arch = "wasm32"))]
fn find_syscall(name: &str) -> Option<&'static NeoVMSyscallInfo> {
    SYSCALLS.iter().find(|info| info.name == name)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn syscall_hash(name: &str) -> NeoResult<u32> {
    find_syscall(name)
        .map(|info| info.hash)
        .ok_or_else(|| NeoError::new(&format!("unknown syscall: {name}")))
}

fn default_value_for(return_type: &str) -> NeoValue {
    match return_type {
        "Void" => NeoValue::Null,
        // Fail-closed by default for unknown boolean-returning syscalls.
        "Boolean" => NeoBoolean::FALSE.into(),
        "Integer" => NeoInteger::new(0).into(),
        "Hash160" => NeoByteString::new(vec![0u8; 20]).into(),
        "ByteString" => NeoByteString::new(vec![0u8; 1]).into(),
        "String" => NeoString::from_str("Neo N3").into(),
        "Array" => NeoArray::<NeoValue>::new().into(),
        "Iterator" => NeoArray::<NeoValue>::new().into(),
        "StackItem" => NeoArray::<NeoValue>::new().into(),
        "StorageContext" => NeoValue::Null,
        _ => NeoValue::Null,
    }
}

fn value_matches_param_type(value: &NeoValue, param_type: &str) -> bool {
    match param_type {
        "Boolean" => value.as_boolean().is_some(),
        "Integer" => value.as_integer().is_some(),
        "Hash160" => {
            value.is_null()
                || value
                    .as_byte_string()
                    .map(|bytes| bytes.len() == 20)
                    .unwrap_or(false)
        }
        "ByteString" => value.as_byte_string().is_some(),
        "String" => value.as_string().is_some(),
        "Array" => value.as_array().is_some(),
        "Iterator" => value.as_array().is_some(),
        "StorageContext" => value.is_null() || value.as_integer().is_some(),
        "StackItem" | "Any" | "ExecutionContext" => true,
        _ => true,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn call_flags_allow_write(flags: i32) -> bool {
    (flags & CALL_FLAGS_WRITE_STATES) != 0
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn call_flags_allow_read(flags: i32) -> bool {
    (flags & CALL_FLAGS_READ_STATES) != 0
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn hash160_prefix_i64(hash: &[u8; 20]) -> i64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&hash[..8]);
    i64::from_le_bytes(buf)
}

/// Neo N3 System Call Function
pub fn neovm_syscall(hash: u32, args: &[NeoValue]) -> NeoResult<NeoValue> {
    let registry = crate::NeoVMSyscallRegistry::get_instance();
    let info = registry
        .get_syscall_by_hash(hash)
        .ok_or_else(|| NeoError::new(&format!("unknown syscall hash: 0x{hash:08x}")))?;

    if args.len() != info.parameters.len() {
        return Err(NeoError::new(&format!(
            "invalid syscall argument count for {}: expected {}, got {}",
            info.name,
            info.parameters.len(),
            args.len()
        )));
    }

    for (index, (arg, expected_type)) in args.iter().zip(info.parameters.iter()).enumerate() {
        if !value_matches_param_type(arg, expected_type) {
            return Err(NeoError::new(&format!(
                "invalid syscall argument type for {} param #{}: expected {}",
                info.name, index, expected_type
            )));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if info.name == "System.Runtime.CheckWitness" {
            let has_witness = args
                .first()
                .and_then(NeoValue::as_byte_string)
                .map(|account| has_active_witness(account.as_slice()))
                .unwrap_or(false);
            return Ok(NeoBoolean::new(has_witness).into());
        }

        if info.name == "System.Crypto.CheckSig" {
            let results = active_crypto_verification_results();
            return Ok(NeoBoolean::new(results.check_sig).into());
        }

        if info.name == "System.Crypto.CheckMultisig" {
            let results = active_crypto_verification_results();
            return Ok(NeoBoolean::new(results.check_multisig).into());
        }

        if info.name == "Neo.Crypto.VerifyWithECDsa" {
            let results = active_crypto_verification_results();
            return Ok(NeoBoolean::new(results.verify_with_ecdsa).into());
        }

        if info.name == "System.Runtime.GetCallingScriptHash" {
            return Ok(NeoByteString::from_slice(&current_calling_script_hash()).into());
        }

        if info.name == "System.Runtime.GetEntryScriptHash" {
            return Ok(NeoByteString::from_slice(&current_entry_script_hash()).into());
        }

        if info.name == "System.Runtime.GetExecutingScriptHash" {
            return Ok(NeoByteString::from_slice(&current_executing_script_hash()).into());
        }

        if info.name == "System.Contract.GetCallFlags" {
            return Ok(NeoInteger::new(current_call_flags()).into());
        }

        // B5: get_random
        if info.name == "System.Runtime.GetRandom" {
            return Ok(NeoInteger::new(
                *crate::storage::ACTIVE_RANDOM
                    .read()
                    .expect("ACTIVE_RANDOM poisoned"),
            )
            .into());
        }

        // B6: get_time
        if info.name == "System.Runtime.GetTime" {
            return Ok(NeoInteger::new(
                *crate::storage::ACTIVE_TIME
                    .read()
                    .expect("ACTIVE_TIME poisoned"),
            )
            .into());
        }

        // B6: get_invocation_counter
        if info.name == "System.Runtime.GetInvocationCounter" {
            return Ok(NeoInteger::new(
                *crate::storage::ACTIVE_INVOCATION_COUNTER
                    .read()
                    .expect("ACTIVE_INVOCATION_COUNTER poisoned"),
            )
            .into());
        }

        // B7: gas_left
        if info.name == "System.Runtime.GasLeft" {
            return Ok(NeoInteger::new(
                *crate::storage::ACTIVE_GAS_LEFT
                    .read()
                    .expect("ACTIVE_GAS_LEFT poisoned"),
            )
            .into());
        }

        // B8: current_signers. The C# struct has Account + Scopes;
        // we serialise each signer as a 2-element array [account, scopes].
        if info.name == "System.Runtime.CurrentSigners" {
            let witnesses = crate::storage::ACTIVE_WITNESSES
                .read()
                .expect("ACTIVE_WITNESSES poisoned");
            let arr: NeoArray<NeoValue> = witnesses
                .iter()
                .map(|w| {
                    let entry: NeoArray<NeoValue> = vec![
                        NeoValue::from(NeoByteString::from_slice(w)),
                        NeoValue::from(NeoInteger::new(0x01)), // Global scope
                    ]
                    .into_iter()
                    .collect();
                    NeoValue::from(entry)
                })
                .collect();
            return Ok(NeoValue::from(arr));
        }

        // B9: get_notifications(hash?). Hash arg: NeoValue::Null
        // means "all notifications". Returns the recorded
        // notifications as a NeoArray.
        if info.name == "System.Runtime.GetNotifications" {
            use crate::host_notifications::take;
            let recorded = take();
            let arr: NeoArray<NeoValue> = recorded
                .into_iter()
                .map(|n| {
                    let entry: NeoArray<NeoValue> = vec![
                        NeoValue::from(NeoString::from_str(&n.event)),
                        NeoValue::from(n.state.into_iter().collect::<NeoArray<NeoValue>>()),
                    ]
                    .into_iter()
                    .collect();
                    NeoValue::from(entry)
                })
                .collect();
            return Ok(NeoValue::from(arr));
        }
    }

    Ok(default_value_for(info.return_type))
}
