// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! CryptoLib native-contract call lowering for the `Neo.Crypto.*` aliases.

use super::*;

/// Lower a `CryptoLib` native-contract method call to
/// `System.Contract.Call(contractHash, method, callFlags, args)`.
///
/// The import is expected to take the same arguments the CryptoLib method
/// takes (e.g. `sha256(data)`). We emit, bottom-to-top: `contractHash`,
/// `method`, `callFlags` (AllowCall + AllowNotify = 0b1010 = 0x0E... actually
/// read-only calls use CallFlags.ReadOnly = 0b0100 for the pure hashes; crypto
/// verification also needs no state mutation so ReadOnly is correct), then the
/// caller's argument list already on the Wasm stack is wrapped via the call
/// convention used by the translator. For the import-bridge path the runtime
/// helper owns arg marshalling, so we only need to emit the call frame: push
/// hash, push method, push flags, `SYSCALL System.Contract.Call`.
pub(super) fn emit_cryptolib_call(
    script: &mut Vec<u8>,
    contract_hash: [u8; 20],
    method: &str,
    import_name: &str,
) -> Result<&'static str> {
    let call_syscall = syscalls::lookup_extended("System.Contract.Call")
        .ok_or_else(|| anyhow!("System.Contract.Call syscall not found"))?;
    let syscall_op =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
    if syscall_op.operand_size != 4 || syscall_op.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }

    // Stack order required by System.Contract.Call (top to bottom):
    //   args[]  (already supplied by the caller/import bridge)
    //   callFlags
    //   method  (ByteString)
    //   contractHash (ByteString, 20 bytes)
    // Emit bottom-up so the final top matches the syscall's expectation.
    emit_push_data(script, &contract_hash)?;
    emit_push_data(script, method.as_bytes())?;
    // CallFlags.ReadOnly = 0b0100 = 4. CryptoLib hash/verify methods are pure
    // and do not mutate contract state, so ReadOnly is sufficient and least
    // privilege.
    let _ = emit_push_int(script, 4);
    script.push(syscall_op.byte);
    script.extend_from_slice(&call_syscall.hash.to_le_bytes());
    Ok(Box::leak(format!("Neo.Crypto.{method} (via {import_name})").into_boxed_str()) as &str)
}
