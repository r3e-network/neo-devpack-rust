// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! CryptoLib native-contract call lowering for the `Neo.Crypto.*` aliases.
//!
//! The CryptoLib methods (`sha256`, `ripemd160`, `keccak256`, `murmur32`,
//! `verifyWithECDsa`) live on the CryptoLib *native contract* and must be
//! invoked via `System.Contract.Call` — there is no `Register("Neo.Crypto.*")`
//! interop on Neo N3, so a bare `SYSCALL <Neo.Crypto hash>` deploys but faults
//! with "InteropService not found" on first execution.
//!
//! `System.Contract.Call` pops, TOP-FIRST: `hash`, `method`, `callFlags`,
//! `args` (neo-go v0.105.1 `pkg/core/interop/contract/call.go::Call`), so the
//! args `Array` must be pushed first (deepest) and the 20-byte contract hash
//! last (on top) — the same frame `emit_stdlib_deserialize_call` in the
//! sibling `runtime` module emits.
//!
//! [`try_handle_crypto_import`] is the full lowering for `neo`-module imports
//! (it has `RuntimeHelpers` access): each `(ptr, len)` i32 pair is marshalled
//! out of wasm linear memory into a `ByteString` via the shared
//! `ExtractMemoryBytes` runtime helper (correct for both the compact and
//! chunked memory layouts), the N arguments are `PACK`ed into the args
//! `Array`, and the call frame is emitted. [`emit_cryptolib_call`] remains for
//! the legacy adapter/`syscall`-module paths that have no `RuntimeHelpers` in
//! scope and therefore cannot marshal — see its doc comment.

use super::*;

use crate::native_contracts::{crypto_lib_method, NativeMethod, CRYPTOLIB_HASH};

/// `CallFlags.ReadOnly = ReadStates | AllowCall = 0b0101` (neo-go
/// `pkg/smartcontract/callflag`: ReadStates = 0b0001, AllowCall = 0b0100).
/// Least privilege for the pure CryptoLib methods (their required flags are
/// none), while still permitting the `System.Contract.Call` hop itself.
/// NOTE: this is NOT `0b0100` — that bit alone is `AllowCall`.
const CALLFLAGS_READ_ONLY: i128 = 0b0101;

/// The `neo`-module import names [`try_handle_crypto_import`] recognizes
/// (via [`resolve_cryptolib_target`]), with the descriptor each lowers to.
/// The ABI parity test in `super` chains this list.
#[cfg(test)]
pub(super) const HANDLED_IMPORTS: &[(&str, &str)] = &[
    ("sha256", "Neo.Crypto.SHA256"),
    ("crypto_sha256", "Neo.Crypto.SHA256"),
    ("ripemd160", "Neo.Crypto.RIPEMD160"),
    ("crypto_ripemd160", "Neo.Crypto.RIPEMD160"),
    ("keccak256", "Neo.Crypto.Keccak256"),
    ("crypto_keccak256", "Neo.Crypto.Keccak256"),
    ("murmur32", "Neo.Crypto.Murmur32"),
    ("crypto_murmur32", "Neo.Crypto.Murmur32"),
    ("verify_with_ecdsa", "Neo.Crypto.VerifyWithECDsa"),
    ("crypto_verify_with_ecdsa", "Neo.Crypto.VerifyWithECDsa"),
];

/// The wasm-side argument shape of a CryptoLib method import: `buffer_pairs`
/// leading `(ptr: i32, len: i32)` byte-buffer pairs followed by
/// `trailing_ints` plain i32 scalars, in the same order as the native
/// method's parameters.
struct CryptoCallShape {
    buffer_pairs: usize,
    trailing_ints: usize,
}

impl CryptoCallShape {
    fn param_count(&self) -> usize {
        self.buffer_pairs * 2 + self.trailing_ints
    }
    fn native_arg_count(&self) -> usize {
        self.buffer_pairs + self.trailing_ints
    }
}

/// The expected import shape per CryptoLib method (parameter lists per
/// `CRYPTOLIB_DESCRIPTOR` / neo-go v0.105.1 `native/crypto.go`):
///   sha256/ripemd160/keccak256(data)            -> (ptr, len)
///   murmur32(data, seed)                        -> (ptr, len, seed)
///   verifyWithECDsa(msg, pubkey, sig, curve)    -> 3 pairs + curve
fn cryptolib_call_shape(method: &str) -> Option<CryptoCallShape> {
    match method {
        "sha256" | "ripemd160" | "keccak256" => Some(CryptoCallShape {
            buffer_pairs: 1,
            trailing_ints: 0,
        }),
        "murmur32" => Some(CryptoCallShape {
            buffer_pairs: 1,
            trailing_ints: 1,
        }),
        "verifyWithECDsa" => Some(CryptoCallShape {
            buffer_pairs: 3,
            trailing_ints: 1,
        }),
        _ => None,
    }
}

/// Map a canonical CryptoLib method name back to its static descriptor.
fn cryptolib_descriptor_for_method(method: &str) -> Result<&'static str> {
    Ok(match method {
        "sha256" => "Neo.Crypto.SHA256",
        "ripemd160" => "Neo.Crypto.RIPEMD160",
        "keccak256" => "Neo.Crypto.Keccak256",
        "murmur32" => "Neo.Crypto.Murmur32",
        "verifyWithECDsa" => "Neo.Crypto.VerifyWithECDsa",
        other => bail!("unknown CryptoLib method '{other}'"),
    })
}

/// Resolve a `neo`-module import name to a single-call CryptoLib native
/// method: directly (`sha256`, `crypto_sha256`, `Neo.Crypto.SHA256`, ...) or
/// through the general alias map (`lookup_neo_syscall`). `None` for
/// non-crypto names and for the composite `hash160`/`hash256` (which have no
/// single native method and must be lowered as explicit call sequences).
fn resolve_cryptolib_target(name: &str) -> Option<(&'static str, NativeMethod)> {
    let method = crypto_lib_method(name).or_else(|| {
        neo_syscalls::lookup_neo_syscall(name).and_then(crypto_lib_method)
    })?;
    let descriptor = cryptolib_descriptor_for_method(method.method).ok()?;
    Some((descriptor, method))
}

/// Move the stack item at 0-based depth `n` to the top (NeoVM `ROLL` with the
/// small-depth `SWAP`/`ROT` forms). `n == 0` is a no-op.
fn emit_roll(script: &mut Vec<u8>, n: usize) -> Result<()> {
    match n {
        0 => {}
        1 => script.push(lookup_opcode("SWAP")?.byte),
        2 => script.push(lookup_opcode("ROT")?.byte),
        _ => {
            let _ = emit_push_int(script, n as i128);
            script.push(lookup_opcode("ROLL")?.byte);
        }
    }
    Ok(())
}

/// Emit the `System.Contract.Call` frame tail. Expects the args `Array`
/// already on top of the stack; pushes `callFlags` (ReadOnly = 0b0101), the
/// method name, then the 20-byte contract hash ON TOP — matching neo-go
/// `contract/call.go::Call`'s top-first pop order (hash, method, callFlags,
/// args) — and issues the SYSCALL. Leaves the native method's return value on
/// the stack.
fn emit_contract_call_frame(
    script: &mut Vec<u8>,
    contract_hash: &[u8; 20],
    method: &str,
) -> Result<()> {
    let _ = emit_push_int(script, CALLFLAGS_READ_ONLY);
    emit_push_data(script, method.as_bytes())?;
    emit_push_data(script, contract_hash)?;

    let call_syscall = syscalls::lookup_extended("System.Contract.Call")
        .ok_or_else(|| anyhow!("System.Contract.Call syscall not found"))?;
    let syscall_op =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
    if syscall_op.operand_size != 4 || syscall_op.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }
    script.push(syscall_op.byte);
    script.extend_from_slice(&call_syscall.hash.to_le_bytes());
    Ok(())
}

/// Lower a `neo`-module CryptoLib import to a fully-marshalled
/// `System.Contract.Call(contractHash, method, callFlags, args)`.
///
/// Entry stack (wasm push order, top-first): the LAST wasm argument is on
/// top. For `verifyWithECDsa(msg_ptr, msg_len, pk_ptr, pk_len, sig_ptr,
/// sig_len, curve)` that is `curve, sig_len, sig_ptr, pk_len, pk_ptr,
/// msg_len, msg_ptr`. Buffer pairs are therefore extracted LAST-first: each
/// round `ROLL`s the current topmost remaining pair's `ptr` then `len` to the
/// top (`len` above `ptr`, the `ExtractMemoryBytes` convention) and gathers
/// the bytes out of linear memory via the shared runtime helper (correct for
/// both the compact and chunked layouts). After the loop the extracted
/// `ByteString`s sit FIRST-arg-on-top above the trailing scalars, which is
/// exactly `PACK`'s pop order for `args[0..N]`; then the `callFlags = 5`,
/// method and contract hash (ON TOP) frame is emitted — mirroring how the
/// sibling `runtime` module lowers `StdLib.deserialize`.
///
/// The finalizer auto-inserts the scoped CryptoLib manifest permission for
/// the methods recorded via [`RuntimeHelpers::mark_cryptolib_used`]; without
/// it Neo N3 denies the contract call at runtime.
pub(super) fn try_handle_crypto_import(
    import: &FunctionImport,
    func_type: &FuncType,
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
    let Some((descriptor, method)) = resolve_cryptolib_target(&import.name) else {
        return Ok(None);
    };
    let shape = cryptolib_call_shape(method.method).ok_or_else(|| {
        anyhow!(
            "CryptoLib method '{}' (import '{}::{}') has no wasm buffer-ABI shape",
            method.method,
            import.module,
            import.name
        )
    })?;

    if func_type.params().len() != shape.param_count()
        || func_type.params().iter().any(|ty| *ty != ValType::I32)
    {
        bail!(
            "neo import '{}::{}' lowers to CryptoLib '{}' and must take exactly {} i32 \
             parameter(s): {} (ptr, len) byte-buffer pair(s){}",
            import.module,
            import.name,
            method.method,
            shape.param_count(),
            shape.buffer_pairs,
            if shape.trailing_ints > 0 {
                format!(" followed by {} integer scalar(s)", shape.trailing_ints)
            } else {
                String::new()
            }
        );
    }
    if func_type.results().len() != 1
        || !matches!(func_type.results()[0], ValType::I32 | ValType::I64)
    {
        bail!(
            "neo import '{}::{}' must return a single i32/i64 (the CryptoLib '{}' result \
             stays on the NeoVM stack)",
            import.module,
            import.name,
            method.method
        );
    }

    ensure_memory_access(runtime, 0)?;
    runtime.emit_memory_init_call(script)?;

    // Marshal the buffer pairs LAST-first (the last pair is nearest the top).
    // Invariant per round: `extracted` ByteStrings sit on top, then the
    // `trailing_ints` scalars, then the remaining pairs — so the current
    // pair's `len` is at depth `extracted + trailing_ints` and its `ptr` one
    // below. Two ROLLs restore `(ptr, len)` on top (len above ptr) without
    // disturbing anything else; depth 0 means the pair is already on top in
    // extract order (wasm pushed ptr then len).
    for extracted in 0..shape.buffer_pairs {
        let depth = extracted + shape.trailing_ints;
        if depth > 0 {
            emit_roll(script, depth + 1)?; // ptr -> top
            emit_roll(script, depth + 1)?; // len -> top (above ptr)
        }
        runtime.emit_storage_helper(
            script,
            crate::translator::runtime::StorageHelperKind::ExtractMemoryBytes,
        )?;
    }

    // PACK pops top-first, so the first native argument (extracted last,
    // now on top) lands in args[0] and the trailing scalars in the tail.
    let _ = emit_push_int(script, shape.native_arg_count() as i128);
    script.push(lookup_opcode("PACK")?.byte);

    emit_contract_call_frame(script, &method.contract_hash, method.method)?;
    runtime.mark_cryptolib_used(method.method);

    Ok(Some(descriptor))
}

/// Legacy CryptoLib lowering for call sites WITHOUT `RuntimeHelpers` access
/// (chain-adapter resolved imports — Solana `sol_sha256`, Move `hash_sha256`
/// — and `syscall`-module descriptor imports, which reach here through
/// `emit_descriptor_syscall` / `emit_neo_syscall` where no argument
/// marshalling is possible).
///
/// Emits a structurally-correct `System.Contract.Call` frame with an EMPTY
/// args array (`NEWARRAY0`), in neo-go `contract/call.go::Call`'s top-first
/// pop order — args (deepest), `callFlags = ReadOnly = ReadStates|AllowCall =
/// 0b0101`, method, contract hash ON TOP. The caller's raw wasm operands (an
/// out-pointer ABI for the adapter paths) are left beneath the frame and are
/// NOT marshalled, so on-chain the call FAULTS loudly at CryptoLib's
/// argument-count check instead of silently mis-binding hash/method/flags
/// (the previous emission pushed the hash DEEPEST, so the VM popped the flags
/// integer as the contract hash).
///
/// Full marshalling for these adapter ABIs needs a result copy-back helper
/// (the digest must be written to the caller's out-pointer) and is tracked
/// separately; the `neo`-module import surface is fully lowered by
/// [`try_handle_crypto_import`] above.
pub(super) fn emit_cryptolib_call(
    script: &mut Vec<u8>,
    contract_hash: [u8; 20],
    method: &str,
    import_name: &str,
) -> Result<&'static str> {
    debug_assert_eq!(
        contract_hash, CRYPTOLIB_HASH,
        "emit_cryptolib_call is only routed CryptoLib methods (import '{import_name}')"
    );
    script.push(lookup_opcode("NEWARRAY0")?.byte);
    emit_contract_call_frame(script, &contract_hash, method)?;
    cryptolib_descriptor_for_method(method)
}
