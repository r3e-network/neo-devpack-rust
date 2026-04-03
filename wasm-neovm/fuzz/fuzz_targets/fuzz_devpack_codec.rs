// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: exercise neo-devpack codec roundtrips and failure handling.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Arbitrary)]
enum CodecValue {
    Unit,
    Bool(bool),
    I32(i32),
    I64(i64),
    Text(String),
    Bytes(Vec<u8>),
    Numbers(Vec<i16>),
    MaybePair(Option<(u16, bool)>),
    Record(CodecRecord),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Arbitrary)]
struct CodecRecord {
    enabled: bool,
    counter: u32,
    label: String,
    payload: Vec<u8>,
    small_numbers: Vec<u8>,
}

#[derive(Debug, Arbitrary)]
struct CodecFuzzInput<'a> {
    value: CodecValue,
    raw_bytes: &'a [u8],
    flip_serialized_byte: bool,
    truncate_to: u8,
}

fuzz_target!(|input: CodecFuzzInput<'_>| {
    let encoded = neo_devpack::codec::serialize(&input.value)
        .expect("supported fuzz value shapes must serialize");
    let decoded: CodecValue =
        neo_devpack::codec::deserialize(&encoded).expect("valid encodings must round-trip");
    assert_eq!(decoded, input.value);

    if input.flip_serialized_byte && !encoded.is_empty() {
        let mut corrupted = encoded.clone();
        let index = usize::from(input.truncate_to) % corrupted.len();
        corrupted[index] ^= 0xA5;
        let _ = neo_devpack::codec::deserialize::<CodecValue>(&corrupted);
    }

    let truncate_to = usize::from(input.truncate_to).min(encoded.len());
    let _ = neo_devpack::codec::deserialize::<CodecValue>(&encoded[..truncate_to]);

    let _ = neo_devpack::codec::deserialize::<CodecValue>(input.raw_bytes);
    let _ = neo_devpack::codec::deserialize::<CodecRecord>(input.raw_bytes);
});
