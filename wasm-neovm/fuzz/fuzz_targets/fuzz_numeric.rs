// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: numeric + primitive encoding helpers.
//! Verifies public encoding routines stay panic-free and internally consistent.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wasm_neovm::core::{
    decode_bytes, decode_string, decode_varint, encode_bytes, encode_int, encode_string,
    encode_varint, encoded_int_size,
};

#[derive(Debug, Arbitrary)]
struct FuzzNumericInput<'a> {
    integer_value: i64,
    byte_data: &'a [u8],
    text: &'a str,
    varint: u64,
}

fuzz_target!(|input: FuzzNumericInput| {
    let encoded_int = encode_int(input.integer_value);
    assert_eq!(
        encoded_int.len(),
        encoded_int_size(input.integer_value),
        "encoded_int_size must match the actual integer encoding length"
    );
    assert!(!encoded_int.is_empty(), "encode_int must produce output");

    let encoded_varint = encode_varint(input.varint);
    let (decoded_varint, consumed) =
        decode_varint(&encoded_varint).expect("encoded varints must decode");
    assert_eq!(decoded_varint, input.varint);
    assert_eq!(consumed, encoded_varint.len());

    let encoded_bytes = encode_bytes(input.byte_data);
    let (decoded_bytes, consumed) =
        decode_bytes(&encoded_bytes).expect("encoded byte vectors must decode");
    assert_eq!(decoded_bytes, input.byte_data);
    assert_eq!(consumed, encoded_bytes.len());

    let encoded_string = encode_string(input.text);
    let (decoded_string, consumed) =
        decode_string(&encoded_string).expect("encoded strings must decode");
    assert_eq!(decoded_string, input.text);
    assert_eq!(consumed, encoded_string.len());
});
