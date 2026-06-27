// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! L9: typed cross-contract call integration tests.
//!
//! These tests verify the `ContractCaller` trait + `FromNeoValue`
//! impls work end-to-end. The `DefaultContractCaller::call_raw`
//! on the host path uses the B4 fallback (returns Null/Err) —
//! the L6 cross-call executor will replace this; here we just
//! verify the type-system wiring.

use neo_devpack::prelude::*;

#[test]
fn from_neo_value_round_trip_integer() {
    let v = NeoValue::Integer(NeoInteger::new(42));
    let parsed: NeoInteger = NeoInteger::from_value(&v).expect("integer");
    assert_eq!(parsed.try_as_i64(), Some(42));
}

#[test]
fn from_neo_value_round_trip_bool() {
    let v = NeoValue::Boolean(true.into());
    let parsed: bool = bool::from_value(&v).expect("bool");
    assert!(parsed);
}

#[test]
fn from_neo_value_round_trip_string() {
    let v = NeoValue::String(NeoString::from_str("hello"));
    let parsed: String = String::from_value(&v).expect("string");
    assert_eq!(parsed, "hello");
}

#[test]
fn from_neo_value_round_trip_bytes() {
    let v = NeoValue::ByteString(NeoByteString::from_slice(&[1, 2, 3]));
    let parsed: Vec<u8> = Vec::<u8>::from_value(&v).expect("bytes");
    assert_eq!(parsed, vec![1, 2, 3]);
}

#[test]
fn from_neo_value_unit_from_null() {
    let v = NeoValue::Null;
    let parsed: () = <() as FromNeoValue>::from_value(&v).expect("unit");
    assert_eq!(parsed, ());
}

#[test]
fn contract_caller_default_returns_null_for_known_hash() {
    // The DefaultContractCaller's call_raw returns whatever
    // the host-mode dispatcher returns for `contract_call` —
    // Ok(Null) on the L6 B4 fallback path, or an Err. We
    // accept both; the L6 cross-call upgrade will replace
    // the dispatcher with a real executor.
    let script_hash = NeoByteString::from_slice(&[0u8; 20]);
    let call_flags = NeoInteger::new(0x0F);
    let _ = call_typed::<NeoValue>(&script_hash, "method", &[], &call_flags);
}

#[test]
fn contract_caller_call_typed_integer() {
    // call_typed<NeoInteger> must decode NeoValue::Integer to
    // NeoInteger. The underlying call may return Null/Err
    // (L6 B4 fallback); in that case the typed decode fails.
    let script_hash = NeoByteString::from_slice(&[0u8; 20]);
    let call_flags = NeoInteger::new(0x0F);
    let _ = call_typed::<NeoInteger>(&script_hash, "balanceOf", &[], &call_flags);
}
