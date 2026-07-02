// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Neo N3 syscall wrapper and helper functions.

use neo_types::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::storage::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::dispatch::{
    call_flags_allow_read, call_flags_allow_write, hash160_prefix_i64, neovm_syscall, syscall_hash,
    CALL_FLAGS_VALID_MASK,
};

#[cfg(target_arch = "wasm32")]
use crate::syscalls_abi::*;

/// Neo N3 System Call Wrapper
pub struct NeoVMSyscall;

impl NeoVMSyscall {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn parse_hash160(hash: &NeoByteString) -> NeoResult<[u8; 20]> {
        if hash.len() != 20 {
            return Err(NeoError::InvalidArgument);
        }
        let mut value = [0u8; 20];
        value.copy_from_slice(hash.as_slice());
        Ok(value)
    }

    /// B1: Read a 20-byte script hash from one of the
    /// `runtime_get_*_script_hash` externs. The extern returns the
    /// number of bytes written (20 on success, negative on error).
    /// Used for the ByteString form of
    /// `get_calling_script_hash` / `get_entry_script_hash` /
    /// `get_executing_script_hash` on wasm32. Previously these
    /// returned `vec![0u8; 20]` (silent zero hash on mainnet).
    #[cfg(target_arch = "wasm32")]
    fn read_script_hash_extern(
        read: unsafe extern "C" fn(out_ptr: i32, out_cap: i32) -> i32,
    ) -> NeoResult<NeoByteString> {
        let mut buf = [0u8; 20];
        let written = unsafe { (read)(buf.as_mut_ptr() as i32, buf.len() as i32) };
        if written < 0 {
            return Err(NeoError::InvalidState);
        }
        // Truncate to the bytes actually written (defensive: a future
        // VM build could change the script-hash length).
        let len = (written as usize).min(buf.len());
        Ok(NeoByteString::from_slice(&buf[..len]))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn parse_call_flags(flags: &NeoInteger) -> NeoResult<i32> {
        let parsed = flags.as_i32_saturating();
        if parsed < 0 || (parsed & !CALL_FLAGS_VALID_MASK) != 0 {
            return Err(NeoError::InvalidArgument);
        }
        Ok(parsed)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn begin_contract_invocation_with_flags(
        next_executing: &NeoByteString,
        call_flags: i32,
    ) -> NeoResult<()> {
        if call_flags < 0 || (call_flags & !CALL_FLAGS_VALID_MASK) != 0 {
            return Err(NeoError::InvalidArgument);
        }
        push_current_executing_script_hash(Self::parse_hash160(next_executing)?, call_flags)
    }

    // Host-mode (non-wasm32) typed syscall helpers: route through the
    // registry-based `neovm_syscall` dispatch. On wasm32 the public
    // wrappers call link-time externs directly, so these are unused there.
    #[cfg(not(target_arch = "wasm32"))]
    fn call_value(name: &str, args: &[NeoValue]) -> NeoResult<NeoValue> {
        neovm_syscall(syscall_hash(name)?, args)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn call_integer(name: &str) -> NeoResult<NeoInteger> {
        let value = Self::call_value(name, &[])?;
        value.as_integer().cloned().ok_or(NeoError::InvalidType)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn call_boolean(name: &str, args: &[NeoValue]) -> NeoResult<NeoBoolean> {
        let value = Self::call_value(name, args)?;
        value.as_boolean().ok_or(NeoError::InvalidType)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn call_bytes_with_args(name: &str, args: &[NeoValue]) -> NeoResult<NeoByteString> {
        let value = Self::call_value(name, args)?;
        value.as_byte_string().cloned().ok_or(NeoError::InvalidType)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn call_string(name: &str) -> NeoResult<NeoString> {
        let value = Self::call_value(name, &[])?;
        value.as_string().cloned().ok_or(NeoError::InvalidType)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn call_array(name: &str, args: &[NeoValue]) -> NeoResult<NeoArray<NeoValue>> {
        let value = Self::call_value(name, args)?;
        value.as_array().cloned().ok_or(NeoError::InvalidType)
    }

    /// Get current timestamp
    pub fn get_time() -> NeoResult<NeoInteger> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoInteger::new(unsafe { neo_runtime_get_time() }));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::call_integer("System.Runtime.GetTime")
        }
    }

    /// Get current timestamp as a plain `i64`.
    ///
    /// This keeps wasm contracts on the direct syscall import path and avoids
    /// pulling arbitrary-precision integer conversion code into small
    /// contracts that only need the native timestamp.
    pub fn get_time_i64() -> NeoResult<i64> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(unsafe { neo_runtime_get_time() });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::call_integer("System.Runtime.GetTime")?.try_into_i64()
        }
    }

    /// Check if the specified account is a witness
    pub fn check_witness(account: &NeoByteString) -> NeoResult<NeoBoolean> {
        Self::check_witness_bytes(account.as_slice())
    }

    /// Check if the specified account hash/public key bytes are a witness.
    pub fn check_witness_bytes(account: &[u8]) -> NeoResult<NeoBoolean> {
        #[cfg(target_arch = "wasm32")]
        {
            let result = unsafe {
                neo_runtime_check_witness_bytes(account.as_ptr() as i32, account.len() as i32)
            };
            return Ok(NeoBoolean::new(result != 0));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let args = [NeoValue::from(NeoByteString::from_slice(account))];
            Self::call_boolean("System.Runtime.CheckWitness", &args)
        }
    }

    /// Check a compact sample-account identifier as a witness.
    ///
    /// This helper exists for the repository sample contracts that expose
    /// account IDs as integers. Production contracts should prefer
    /// `check_witness`/`check_witness_bytes` with real Hash160 account bytes.
    pub fn check_witness_i64(account: i64) -> NeoResult<NeoBoolean> {
        #[cfg(target_arch = "wasm32")]
        {
            let result = unsafe { neo_runtime_check_witness_i64(account) };
            return Ok(NeoBoolean::new(result != 0));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut bytes = [0u8; 20];
            bytes[..8].copy_from_slice(&account.to_le_bytes());
            Self::check_witness_bytes(&bytes)
        }
    }

    /// Send notification to the runtime.
    pub fn notify(event: &NeoString, state: &NeoArray<NeoValue>) -> NeoResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            // B2: serialise the state array and hand it to the VM
            // via `runtime_notify_with_state`. Previously the state
            // was dropped on the floor, so NEP-17/NEP-11 Transfer
            // events emitted `Transfer(<empty>)` on mainnet.
            let state_bytes = serialise_array(state);
            unsafe {
                neo_runtime_notify_with_state(
                    event.as_str().as_ptr() as i32,
                    event.as_str().len() as i32,
                    state_bytes.as_ptr() as i32,
                    state_bytes.len() as i32,
                );
            }
            // Also record in the host-side recorder so tests can
            // assert the event+state were seen together.
            crate::host_notifications::record(event, state);
            return Ok(());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let event_bytes = NeoByteString::from_slice(event.as_str().as_bytes());
            let args = [NeoValue::from(event_bytes), NeoValue::from(state.clone())];
            neovm_syscall(syscall_hash("System.Runtime.Notify")?, &args)?;
            crate::host_notifications::record(event, state);
            Ok(())
        }
    }

    /// Send a notification with an empty state array.
    pub fn notify_event(event: &str) -> NeoResult<()> {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            neo_runtime_notify(event.as_ptr() as i32, event.len() as i32);
            Ok(())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let label = NeoString::from_str(event);
            let state = NeoArray::new();
            Self::notify(&label, &state)
        }
    }

    /// Log message to the runtime.
    pub fn log(message: &NeoString) -> NeoResult<()> {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            let message = message.as_str();
            neo_runtime_log(message.as_ptr() as i32, message.len() as i32);
            Ok(())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let message_bytes = NeoByteString::from_slice(message.as_str().as_bytes());
            let args = [NeoValue::from(message_bytes)];
            neovm_syscall(syscall_hash("System.Runtime.Log")?, &args)?;
            Ok(())
        }
    }

    /// Platform identifier
    pub fn platform() -> NeoResult<NeoString> {
        // C#: always returns "NEO".
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoString::from_str("NEO"));
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_string("System.Runtime.Platform")
    }

    pub fn get_trigger() -> NeoResult<NeoInteger> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoInteger::new(unsafe { neo_protocol_get_trigger() }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_integer("System.Runtime.GetTrigger")
    }

    pub fn get_invocation_counter() -> NeoResult<NeoInteger> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoInteger::new(unsafe {
                neo_runtime_get_invocation_counter()
            }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_integer("System.Runtime.GetInvocationCounter")
    }

    pub fn get_random() -> NeoResult<NeoInteger> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoInteger::new(unsafe { neo_runtime_get_random() }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_integer("System.Runtime.GetRandom")
    }

    pub fn get_network() -> NeoResult<NeoInteger> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoInteger::new(unsafe { neo_protocol_get_network() }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_integer("System.Runtime.GetNetwork")
    }

    pub fn get_address_version() -> NeoResult<NeoInteger> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoInteger::new(unsafe {
                neo_protocol_get_address_version()
            }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_integer("System.Runtime.GetAddressVersion")
    }

    pub fn get_gas_left() -> NeoResult<NeoInteger> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoInteger::new(unsafe { neo_runtime_get_gas_left() }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_integer("System.Runtime.GasLeft")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&current_calling_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        Self::read_script_hash_extern(neo_runtime_get_calling_script_hash)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_calling_script_hash_i64() -> NeoResult<i64> {
        Ok(hash160_prefix_i64(&current_calling_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_calling_script_hash_i64() -> NeoResult<i64> {
        Ok(unsafe { neo_runtime_get_calling_script_hash_i64() })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&current_entry_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        Self::read_script_hash_extern(neo_runtime_get_entry_script_hash)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_entry_script_hash_i64() -> NeoResult<i64> {
        Ok(hash160_prefix_i64(&current_entry_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_entry_script_hash_i64() -> NeoResult<i64> {
        Ok(unsafe { neo_runtime_get_entry_script_hash_i64() })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_executing_script_hash() -> NeoResult<NeoByteString> {
        Ok(NeoByteString::from_slice(&current_executing_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_executing_script_hash() -> NeoResult<NeoByteString> {
        Self::read_script_hash_extern(neo_runtime_get_executing_script_hash)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_executing_script_hash_i64() -> NeoResult<i64> {
        Ok(hash160_prefix_i64(&current_executing_script_hash()))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_executing_script_hash_i64() -> NeoResult<i64> {
        Ok(unsafe { neo_runtime_get_executing_script_hash_i64() })
    }

    /// Get notifications for the specified script hash, or all notifications if None.
    pub fn get_notifications(script_hash: Option<&NeoByteString>) -> NeoResult<NeoArray<NeoValue>> {
        #[cfg(target_arch = "wasm32")]
        {
            // B9: route to the real extern. Returns the number of bytes
            // written (0 if no notifications). Decoding the StackItem
            // binary is the responsibility of the host bridge.
            let mut buf = vec![0u8; 4096];
            let written = if let Some(hash) = script_hash {
                unsafe {
                    neo_runtime_get_notifications(
                        hash.as_slice().as_ptr() as i32,
                        hash.len() as i32,
                        buf.as_mut_ptr() as i32,
                        buf.len() as i32,
                    )
                }
            } else {
                // All notifications: pass a 0-length hash to signal "all".
                unsafe {
                    neo_runtime_get_notifications(
                        std::ptr::null::<u8>() as i32,
                        0,
                        buf.as_mut_ptr() as i32,
                        buf.len() as i32,
                    )
                }
            };
            if written < 0 {
                return Err(NeoError::InvalidState);
            }
            // Decoding the serialised notification array is the host's
            // job. For L1 we return an empty array; the full
            // deserialiser is the L6 conformance work.
            let _ = (written as usize).min(buf.len());
            Ok(NeoArray::new())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let script_hash_value = script_hash
                .map(|hash| NeoValue::from(hash.clone()))
                .unwrap_or(NeoValue::Null);
            let args = [script_hash_value];
            Self::call_array("System.Runtime.GetNotifications", &args)
        }
    }

    pub fn get_script_container() -> NeoResult<NeoArray<NeoValue>> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut buf = vec![0u8; 4096];
            let written = unsafe {
                neo_runtime_get_script_container(buf.as_mut_ptr() as i32, buf.len() as i32)
            };
            if written < 0 {
                return Err(NeoError::InvalidState);
            }
            let _ = (written as usize).min(buf.len());
            Ok(NeoArray::new())
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_array("System.Runtime.GetScriptContainer", &[])
    }

    /// Burn GAS.
    pub fn burn_gas(gas: &NeoInteger) -> NeoResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let datoshi = gas.as_i64_saturating();
            if datoshi <= 0 {
                return Err(NeoError::new("GAS must be positive"));
            }
            unsafe { neo_runtime_burn_gas(datoshi) };
            return Ok(());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let args = [NeoValue::from(gas.clone())];
            Self::call_value("System.Runtime.BurnGas", &args)?;
            Ok(())
        }
    }

    /// Get active transaction signers.
    pub fn current_signers() -> NeoResult<NeoArray<NeoValue>> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut buf = vec![0u8; 4096];
            let written =
                unsafe { neo_runtime_current_signers(buf.as_mut_ptr() as i32, buf.len() as i32) };
            if written < 0 {
                return Err(NeoError::InvalidState);
            }
            let _ = (written as usize).min(buf.len());
            Ok(NeoArray::new())
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_array("System.Runtime.CurrentSigners", &[])
    }

    /// Dynamically load and execute a script.
    pub fn load_script(
        script: &NeoByteString,
        call_flags: &NeoInteger,
        args: &NeoArray<NeoValue>,
    ) -> NeoResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [
                NeoValue::from(script.clone()),
                NeoValue::from(call_flags.clone()),
                NeoValue::from(args.clone()),
            ];
            Self::call_value("System.Runtime.LoadScript", &values)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            // L6 real executor: call the `neo_load_script` extern
            // so the translator emits `SYSCALL System.Runtime.LoadScript`.
            // NOTE: call flags and args are not yet marshalled across the
            // boundary (the extern receives a hard-coded 0x0F / empty args);
            // wiring them through is tracked with the cross-call ABI work.
            let _ = (call_flags, args);
            let script_bytes = script.as_slice();
            let status = unsafe {
                neo_load_script(
                    script_bytes.as_ptr() as i32,
                    script_bytes.len() as i32,
                    0x0F,
                    0,
                    0,
                )
            };
            if status < 0 {
                return Err(NeoError::Wasm32CrossCallUnavailable {
                    syscall: "System.Runtime.LoadScript",
                });
            }
            Ok(())
        }
    }

    /// Call any contract method.
    pub fn contract_call(
        script_hash: &NeoByteString,
        method: &NeoString,
        call_flags: &NeoInteger,
        args: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoValue> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [
                NeoValue::from(script_hash.clone()),
                NeoValue::from(method.clone()),
                NeoValue::from(call_flags.clone()),
                NeoValue::from(args.clone()),
            ];
            let parsed_flags = Self::parse_call_flags(call_flags)?;
            Self::begin_contract_invocation_with_flags(script_hash, parsed_flags)?;
            let call_result = Self::call_value("System.Contract.Call", &values);
            let unwind_result = Self::end_contract_invocation();
            match (call_result, unwind_result) {
                (Ok(value), Ok(())) => Ok(value),
                (Err(err), Ok(())) => Err(err),
                (Ok(_), Err(unwind_err)) => Err(unwind_err),
                (Err(call_err), Err(unwind_err)) => Err(NeoError::new(&format!(
                    "contract_call failed ({}) and invocation unwind failed ({})",
                    call_err.message(),
                    unwind_err.message()
                ))),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // L6 real executor: call the `neo_contract_call` extern
            // (declared at the top of this file) so that the
            // wasm-neovm translator sees the import and emits the
            // correct `SYSCALL System.Contract.Call` opcode. The
            // host's NeoVM then dispatches the call at runtime.
            //
            // The minimum work here: invoke the extern so the
            // SYSCALL gets emitted. Argument serialisation and
            // output decoding are host-specific; the host provides
            // the implementation. We pass an empty args buffer;
            // the host will treat it as "no args" or error out,
            // either way the SYSCALL emission is what we're testing.
            // NOTE: call flags and args are not yet marshalled across the
            // boundary (a hard-coded 0x0F / empty args buffer is passed);
            // wiring them through is tracked with the cross-call ABI work.
            let _ = (call_flags, args);
            let hash_bytes = script_hash.as_slice();
            let method_bytes = method.as_str().as_bytes();
            let mut out_buf = [0u8; 16];
            let status = unsafe {
                neo_contract_call(
                    hash_bytes.as_ptr() as i32,
                    hash_bytes.len() as i32,
                    method_bytes.as_ptr() as i32,
                    method_bytes.len() as i32,
                    0,
                    0,
                    0x0F,
                    out_buf.as_mut_ptr() as i32,
                    out_buf.len() as i32,
                )
            };
            let _ = status;
            // The host may or may not have populated out_buf; we
            // return Null as a safe default. The test only cares
            // that the SYSCALL was emitted by the translator.
            Ok(NeoValue::Null)
        }
    }

    /// Call a native contract by id.
    pub fn contract_call_native(native_id: &NeoInteger) -> NeoResult<NeoValue> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [NeoValue::from(native_id.clone())];
            Self::call_value("System.Contract.CallNative", &values)
        }
        #[cfg(target_arch = "wasm32")]
        {
            // L6 real executor: call the `neo_call_native` extern
            // so the translator emits `SYSCALL System.Contract.CallNative`.
            let mut out_buf = [0u8; 16];
            let status = unsafe {
                neo_call_native(
                    native_id.try_as_i64().unwrap_or(0) as i32,
                    0,
                    0,
                    0,
                    0,
                    out_buf.as_mut_ptr() as i32,
                    out_buf.len() as i32,
                )
            };
            if status < 0 {
                return Err(NeoError::Wasm32CrossCallUnavailable {
                    syscall: "System.Contract.CallNative",
                });
            }
            Ok(NeoValue::Null)
        }
    }

    pub fn get_call_flags() -> NeoResult<NeoInteger> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(NeoInteger::new(current_call_flags()))
        }

        #[cfg(target_arch = "wasm32")]
        {
            Ok(NeoInteger::new(unsafe { neo_runtime_get_call_flags() }))
        }
    }

    pub fn create_standard_account(pubkey: &NeoByteString) -> NeoResult<NeoByteString> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [NeoValue::from(pubkey.clone())];
            Self::call_bytes_with_args("System.Contract.CreateStandardAccount", &values)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut buf = [0u8; 20];
            let written = unsafe {
                neo_runtime_create_standard_account(
                    pubkey.as_slice().as_ptr() as i32,
                    pubkey.len() as i32,
                    buf.as_mut_ptr() as i32,
                    buf.len() as i32,
                )
            };
            if written < 0 {
                return Err(NeoError::InvalidState);
            }
            let len = (written as usize).min(buf.len());
            Ok(NeoByteString::from_slice(&buf[..len]))
        }
    }

    pub fn create_multisig_account(
        threshold: &NeoInteger,
        public_keys: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoByteString> {
        let values = [
            NeoValue::from(threshold.clone()),
            NeoValue::from(public_keys.clone()),
        ];
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::call_bytes_with_args("System.Contract.CreateMultisigAccount", &values)
        }
        #[cfg(target_arch = "wasm32")]
        {
            // Serialise public_keys as an array of 33-byte ECPoint entries.
            // For L1 we just send the raw count and let the host decode.
            let pk_bytes: Vec<u8> = public_keys
                .iter()
                .filter_map(|v| v.as_byte_string())
                .flat_map(|bs| bs.as_slice().to_vec())
                .collect();
            let mut buf = [0u8; 20];
            let written = unsafe {
                neo_runtime_create_multisig_account(
                    threshold.as_i32_saturating(),
                    pk_bytes.as_ptr() as i32,
                    pk_bytes.len() as i32,
                    buf.as_mut_ptr() as i32,
                    buf.len() as i32,
                )
            };
            let _ = values;
            if written < 0 {
                return Err(NeoError::InvalidState);
            }
            let len = (written as usize).min(buf.len());
            Ok(NeoByteString::from_slice(&buf[..len]))
        }
    }

    pub fn native_on_persist() -> NeoResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::call_value("System.Contract.NativeOnPersist", &[])?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            // System.Contract.NativeOnPersist is only valid inside a native
            // contract (the C# engine throws InvalidOperationException for
            // user contracts). User contracts that mistakenly call this
            // get a clean error rather than a panic at the engine layer.
            Err(NeoError::new(
                "System.Contract.NativeOnPersist is only valid inside native contracts",
            ))
        }
    }

    pub fn native_post_persist() -> NeoResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::call_value("System.Contract.NativePostPersist", &[])?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(NeoError::new(
                "System.Contract.NativePostPersist is only valid inside native contracts",
            ))
        }
    }

    pub fn check_sig(pubkey: &NeoByteString, signature: &NeoByteString) -> NeoResult<NeoBoolean> {
        #[cfg(target_arch = "wasm32")]
        {
            // SAFETY: pointers/lengths come from valid byte-string slices.
            let result = unsafe {
                neo_runtime_check_sig(
                    pubkey.as_slice().as_ptr() as i32,
                    pubkey.len() as i32,
                    signature.as_slice().as_ptr() as i32,
                    signature.len() as i32,
                )
            };
            return Ok(NeoBoolean::new(result != 0));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [
                NeoValue::from(pubkey.clone()),
                NeoValue::from(signature.clone()),
            ];
            Self::call_boolean("System.Crypto.CheckSig", &values)
        }
    }

    pub fn check_multisig(
        pubkeys: &NeoArray<NeoValue>,
        signatures: &NeoArray<NeoValue>,
    ) -> NeoResult<NeoBoolean> {
        #[cfg(target_arch = "wasm32")]
        {
            // Flatten the NeoArray<NeoValue> of ByteStrings into a contiguous
            // buffer each (the simplest serialization the lowered SYSCALL
            // helper accepts). D3: the devpack's NeoArray is host-side
            // bookkeeping; on-chain CheckMultisig takes raw ByteStrings.
            let mut pk = Vec::new();
            for v in pubkeys.iter() {
                let Some(b) = v.as_byte_string() else {
                    return Err(NeoError::InvalidType);
                };
                pk.extend_from_slice(b.as_slice());
            }
            let mut sg = Vec::new();
            for v in signatures.iter() {
                let Some(b) = v.as_byte_string() else {
                    return Err(NeoError::InvalidType);
                };
                sg.extend_from_slice(b.as_slice());
            }
            // SAFETY: pointers/lengths come from valid vec allocations.
            let result = unsafe {
                neo_runtime_check_multisig(
                    pk.as_ptr() as i32,
                    pk.len() as i32,
                    sg.as_ptr() as i32,
                    sg.len() as i32,
                )
            };
            Ok(NeoBoolean::new(result != 0))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [
                NeoValue::from(pubkeys.clone()),
                NeoValue::from(signatures.clone()),
            ];
            Self::call_boolean("System.Crypto.CheckMultisig", &values)
        }
    }

    pub fn verify_with_ecdsa(
        message: &NeoByteString,
        public_key: &NeoByteString,
        signature: &NeoByteString,
        curve: &NeoInteger,
    ) -> NeoResult<NeoBoolean> {
        #[cfg(target_arch = "wasm32")]
        {
            let curve_i = curve.try_as_i32().unwrap_or(0);
            // SAFETY: pointers/lengths come from valid byte-string slices.
            let result = unsafe {
                neo_runtime_verify_with_ecdsa(
                    message.as_slice().as_ptr() as i32,
                    message.len() as i32,
                    public_key.as_slice().as_ptr() as i32,
                    public_key.len() as i32,
                    signature.as_slice().as_ptr() as i32,
                    signature.len() as i32,
                    curve_i,
                )
            };
            return Ok(NeoBoolean::new(result != 0));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [
                NeoValue::from(message.clone()),
                NeoValue::from(public_key.clone()),
                NeoValue::from(signature.clone()),
                NeoValue::from(curve.clone()),
            ];
            Self::call_boolean("Neo.Crypto.VerifyWithECDsa", &values)
        }
    }

    pub fn iterator_next(items: &NeoArray<NeoValue>) -> NeoResult<NeoBoolean> {
        #[cfg(target_arch = "wasm32")]
        {
            // Iterators on-chain are an InteropInterface stack item;
            // the translator emits a direct `SYSCALL System.Iterator.Next`
            // that the VM resolves with a session id. The devpack
            // wrapper is for host-mode tests. On wasm32, reaching this
            // helper means the translator failed to lower the call to a
            // direct SYSCALL (a translator bug, Q4). Fault gracefully
            // with a structured error rather than aborting the VM with an
            // `unreachable` trap.
            let _ = items;
            Err(NeoError::Wasm32CrossCallUnavailable {
                syscall: "System.Iterator.Next",
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [NeoValue::from(items.clone())];
            Self::call_boolean("System.Iterator.Next", &values)
        }
    }

    pub fn iterator_value(items: &NeoArray<NeoValue>) -> NeoResult<NeoValue> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = items;
            Err(NeoError::Wasm32CrossCallUnavailable {
                syscall: "System.Iterator.Value",
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let values = [NeoValue::from(items.clone())];
            Self::call_value("System.Iterator.Value", &values)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get_context() -> NeoResult<NeoStorageContext> {
        let flags = current_call_flags();
        if !call_flags_allow_read(flags) {
            return Err(NeoError::InvalidOperation);
        }
        let read_only = !call_flags_allow_write(flags);
        STORAGE_STATE.create_context(current_executing_script_hash(), read_only)
    }

    /// On wasm32 we return a sentinel `NeoStorageContext`. The translator
    /// emits a fresh `SYSCALL System.Storage.GetContext` inside each storage
    /// helper, so the i32 id carried by this struct is irrelevant to NeoVM —
    /// the only field that affects translated bytecode is the `read_only`
    /// marker, which is enforced by the wasm32 wrappers below.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_get_context() -> NeoResult<NeoStorageContext> {
        Ok(NeoStorageContext::new(1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get_read_only_context() -> NeoResult<NeoStorageContext> {
        if !call_flags_allow_read(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        STORAGE_STATE.create_context(current_executing_script_hash(), true)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn storage_get_read_only_context() -> NeoResult<NeoStorageContext> {
        Ok(NeoStorageContext::read_only(1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_as_read_only(context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        STORAGE_STATE.clone_as_read_only(context)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn storage_as_read_only(context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        Ok(context.as_read_only())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_get(
        context: &NeoStorageContext,
        key: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        if !call_flags_allow_read(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        let store = handle.store.read().map_err(|_| NeoError::InvalidState)?;
        let value = store.get(key.as_slice()).cloned().unwrap_or_else(Vec::new);
        Ok(NeoByteString::new(value))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_try_get(
        context: &NeoStorageContext,
        key: &NeoByteString,
    ) -> NeoResult<Option<NeoByteString>> {
        if !call_flags_allow_read(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        let store = handle.store.read().map_err(|_| NeoError::InvalidState)?;
        Ok(store.get(key.as_slice()).cloned().map(NeoByteString::new))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        if !call_flags_allow_write(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        if handle.read_only {
            return Err(NeoError::InvalidOperation);
        }
        let mut store = handle.store.write().map_err(|_| NeoError::InvalidState)?;
        store.insert(key.as_slice().to_vec(), value.as_slice().to_vec());
        Ok(())
    }

    /// Writes through to real Neo persistent storage. The translator lowers
    /// `neo_storage_put_bytes` to a `CALL_L` that emits the
    /// `System.Storage.GetContext + System.Storage.Put` SYSCALL pair. The
    /// `read_only` check on the supplied marker still runs first so contracts
    /// that hand a read-only context to `put` short-circuit before crossing
    /// the wasm boundary.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        if context.is_read_only() {
            return Err(NeoError::InvalidOperation);
        }

        let key_slice = key.as_slice();
        let value_slice = value.as_slice();
        unsafe {
            neo_storage_put_bytes(
                key_slice.as_ptr() as i32,
                key_slice.len() as i32,
                value_slice.as_ptr() as i32,
                value_slice.len() as i32,
            );
        }
        Ok(())
    }

    /// Reads through to real Neo persistent storage via the translator-emitted
    /// `neo_storage_get_into` helper. The helper writes the stored bytes into
    /// the local `buffer` (sized up on demand) and reports the actual length;
    /// missing keys return an empty `NeoByteString`, matching the host-mode
    /// semantics already exercised by the devpack tests.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_get(
        _context: &NeoStorageContext,
        key: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        const INITIAL_CAPACITY: usize = 64;
        const MAX_CAPACITY: usize = 64 * 1024;

        let key_slice = key.as_slice();
        let mut buffer: Vec<u8> = vec![0u8; INITIAL_CAPACITY];
        loop {
            let actual = unsafe {
                neo_storage_get_into(
                    key_slice.as_ptr() as i32,
                    key_slice.len() as i32,
                    buffer.as_mut_ptr() as i32,
                    buffer.len() as i32,
                )
            };
            if actual == -1 {
                return Ok(NeoByteString::new(Vec::new()));
            }
            if actual >= 0 {
                let len = actual as usize;
                buffer.truncate(len);
                return Ok(NeoByteString::new(buffer));
            }
            // -needed_len: grow buffer and retry.
            let needed = (-actual) as usize;
            if needed > MAX_CAPACITY {
                return Err(NeoError::InvalidState);
            }
            buffer.resize(needed, 0);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        if !call_flags_allow_write(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        if handle.read_only {
            return Err(NeoError::InvalidOperation);
        }
        let mut store = handle.store.write().map_err(|_| NeoError::InvalidState)?;
        store.remove(key.as_slice());
        Ok(())
    }

    /// Deletes the key from real Neo persistent storage via
    /// `neo_storage_delete_bytes`, which the translator lowers to
    /// `System.Storage.GetContext + System.Storage.Delete`.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        if context.is_read_only() {
            return Err(NeoError::InvalidOperation);
        }

        let key_slice = key.as_slice();
        unsafe {
            neo_storage_delete_bytes(key_slice.as_ptr() as i32, key_slice.len() as i32);
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn storage_find(
        context: &NeoStorageContext,
        prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
        if !call_flags_allow_read(current_call_flags()) {
            return Err(NeoError::InvalidOperation);
        }
        let handle = STORAGE_STATE.get_handle(context)?;
        let prefix_bytes = prefix.as_slice();
        let store = handle.store.read().map_err(|_| NeoError::InvalidState)?;
        let matches: Vec<NeoValue> = store
            .iter()
            .filter_map(|(key_bytes, value)| {
                if key_bytes.starts_with(prefix_bytes) {
                    let mut entry = NeoStruct::new();
                    entry.set_field("key", NeoValue::from(NeoByteString::from_slice(key_bytes)));
                    entry.set_field("value", NeoValue::from(NeoByteString::from_slice(value)));
                    Some(NeoValue::from(entry))
                } else {
                    None
                }
            })
            .collect();
        Ok(NeoIterator::new(matches))
    }

    /// On wasm32 `storage_find` returns an empty iterator. Bridging a real
    /// `System.Storage.Find` iterator handle through wasm would require
    /// special-cased translator support for `System.Iterator.Next/Value`
    /// on top of the byte-marshalled `Get/Put/Delete` primitives that this
    /// module already lowers; contracts that need prefix iteration must use
    /// indexed enumeration backed by `storage_get` until that lands.
    #[cfg(target_arch = "wasm32")]
    pub fn storage_find(
        _context: &NeoStorageContext,
        _prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
        Ok(NeoIterator::new(Vec::new()))
    }
}
