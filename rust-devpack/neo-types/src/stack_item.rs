// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! NeoVM StackItem binary serialisation.
//!
//! The Neo N3 VM passes `StackItem` values across the host boundary as a
//! binary form defined in C# `neo-project/neo/src/Neo/SmartContract/BinarySerializer.cs`.
//! Contracts and devpack wrappers need to serialise args arrays (for
//! `notify`, `Contract.Call`, etc.) into this form so the Neo VM host
//! can read them.
//!
//! This module is the Rust devpack's re-implementation of that format.
//! The tag bytes are the canonical `Neo.VM.Types.StackItemType` enum
//! values; the integer payload is little-endian two's-complement to
//! match .NET `BigInteger.ToByteArray()`:
//!
//! | Tag (1 byte) | Type |
//! |--------------|------|
//! | `0x00` | Any/Null (no payload) |
//! | `0x20` | Boolean (followed by 0x00/0x01) |
//! | `0x21` | Integer (varint length, then little-endian signed bytes) |
//! | `0x28` | ByteString (varint length, then bytes) |
//! | `0x30` | Buffer (same wire form as ByteString; writable buffer) |
//! | `0x40` | Array (varint count, then nested items) |
//! | `0x41` | Struct (varint count, then nested items) |
//! | `0x48` | Map (varint count, then key/value item pairs) |
//!
//! Reference: C# `ApplicationEngine.Runtime.cs` `RuntimeNotify` calls
//! `BinarySerializer.Serialize(writer, state, MaxNotificationSize, ...)`,
//! and `Neo.VM.Types.StackItemType` for the tag values.

use crate::{NeoArray, NeoString, NeoValue};

/// Max serialised size for a notification (C#: `MaxNotificationSize = 1024`).
pub const MAX_NOTIFICATION_SIZE: usize = 1024;

/// Max items in a serialised Array/Struct (C#: `Limits.MaxStackSize`).
pub const MAX_STACK_SIZE: usize = 1024;

// Canonical `Neo.VM.Types.StackItemType` discriminants.
const TAG_NULL: u8 = 0x00;
const TAG_BOOLEAN: u8 = 0x20;
const TAG_INTEGER: u8 = 0x21;
const TAG_BYTESTRING: u8 = 0x28;
const TAG_ARRAY: u8 = 0x40;
const TAG_STRUCT: u8 = 0x41;

/// Append a Neo `VarInt` length/count prefix.
///
/// Neo's `BinaryWriter.WriteVarInt` (used by `BinarySerializer` via
/// `WriteVarBytes`) is **not** LEB128: values `< 0xFD` are a single
/// byte, otherwise a `0xFD`/`0xFE`/`0xFF` marker is followed by a
/// little-endian `u16`/`u32`/`u64`. Using LEB128 here would mis-encode
/// any length `>= 0x80` and desynchronise the entire downstream stream.
fn push_varint(out: &mut Vec<u8>, value: usize) {
    let value = value as u64;
    if value < 0xFD {
        out.push(value as u8);
    } else if value <= u16::MAX as u64 {
        out.push(0xFD);
        out.extend_from_slice(&(value as u16).to_le_bytes());
    } else if value <= u32::MAX as u64 {
        out.push(0xFE);
        out.extend_from_slice(&(value as u32).to_le_bytes());
    } else {
        out.push(0xFF);
        out.extend_from_slice(&value.to_le_bytes());
    }
}

fn push_integer(out: &mut Vec<u8>, n: &crate::NeoInteger) {
    let bigint = n.as_bigint();
    out.push(TAG_INTEGER);
    // Neo serialises integers as little-endian two's-complement, matching
    // .NET `BigInteger.ToByteArray()`. Zero is the empty byte string (the
    // VM's `Integer.GetSpan()` returns `Empty` for zero); `num-bigint`
    // would otherwise emit a single `0x00`.
    if bigint.sign() == num_bigint::Sign::NoSign {
        push_varint(out, 0);
        return;
    }
    let bytes = bigint.to_signed_bytes_le();
    push_varint(out, bytes.len());
    out.extend_from_slice(&bytes);
}

fn push_bytestring(out: &mut Vec<u8>, bytes: &[u8]) {
    out.push(TAG_BYTESTRING);
    push_varint(out, bytes.len());
    out.extend_from_slice(bytes);
}

fn push_boolean(out: &mut Vec<u8>, b: bool) {
    out.push(TAG_BOOLEAN);
    out.push(if b { 0x01 } else { 0x00 });
}

fn push_stack_item(out: &mut Vec<u8>, value: &NeoValue) {
    match value {
        NeoValue::Null => out.push(TAG_NULL),
        NeoValue::Boolean(b) => push_boolean(out, b.as_bool()),
        NeoValue::Integer(i) => push_integer(out, i),
        NeoValue::ByteString(bs) => push_bytestring(out, bs.as_slice()),
        NeoValue::String(s) => push_bytestring(out, s.as_str().as_bytes()),
        NeoValue::Array(arr) => {
            out.push(TAG_ARRAY);
            push_varint(out, arr.len());
            for item in arr.iter() {
                push_stack_item(out, item);
            }
        }
        NeoValue::Struct(items) => {
            // Structs serialise the same as Arrays but with a different
            // outer tag (per C# `BinarySerializer.Serialize` for
            // `StackItemType.Struct` = 0x41). The NeoVM distinguishes
            // struct from array at the tag level; the contents are
            // field values in declaration order.
            out.push(TAG_STRUCT);
            push_varint(out, items.len());
            for (_name, value) in items.iter() {
                push_stack_item(out, value);
            }
        }
        NeoValue::Map(_) => {
            // Maps cannot appear in a notification state (C# raises
            // `InvalidOperationException`). Encode as Null so the VM
            // receives a deterministic payload.
            out.push(TAG_NULL);
        }
    }
}

/// Serialise a `NeoArray<NeoValue>` as a NeoVM `Array` StackItem.
///
/// The returned bytes match the binary form the C# Neo VM produces
/// for an Array StackItem. Used by `System.Runtime.Notify` (B2 fix) and
/// `System.Contract.Call` (B4 follow-up).
pub fn serialise_array(items: &NeoArray<NeoValue>) -> Vec<u8> {
    let mut out = Vec::with_capacity(items.len() * 4 + 2);
    out.push(TAG_ARRAY);
    push_varint(&mut out, items.len());
    for item in items.iter() {
        push_stack_item(&mut out, item);
    }
    out
}

/// Serialise a single StackItem (used for things like `Contract.Call`
/// args that aren't wrapped in an outer array).
pub fn serialise_value(value: &NeoValue) -> Vec<u8> {
    let mut out = Vec::with_capacity(8);
    push_stack_item(&mut out, value);
    out
}

/// Serialise a UTF-8 event name + state array as a notification body.
/// The body has the same shape as C# `RuntimeNotify` expects:
/// `[event_name as NeoVM ByteString, state as Array StackItem]`.
pub fn serialise_notification(event: &NeoString, state: &NeoArray<NeoValue>) -> Vec<u8> {
    let mut out = Vec::with_capacity(event.as_str().len() + state.len() * 4 + 4);
    // Outer container is an Array of 2 items.
    out.push(TAG_ARRAY);
    push_varint(&mut out, 2);
    push_bytestring(&mut out, event.as_str().as_bytes());
    push_stack_item(&mut out, &NeoValue::Array(state.clone()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NeoBoolean, NeoByteString, NeoInteger};

    #[test]
    fn varint_single_byte() {
        // Neo VarInt encodes everything below 0xFD (253) as one byte,
        // including 128..252 (where LEB128 would use two bytes).
        let mut out = Vec::new();
        push_varint(&mut out, 0);
        push_varint(&mut out, 127);
        push_varint(&mut out, 128);
        push_varint(&mut out, 252);
        assert_eq!(out, vec![0, 127, 128, 252]);
    }

    #[test]
    fn varint_0xfd_marker() {
        // 253 needs the 0xFD marker + little-endian u16.
        let mut out = Vec::new();
        push_varint(&mut out, 253);
        assert_eq!(out, vec![0xFD, 0xFD, 0x00]);

        let mut out = Vec::new();
        push_varint(&mut out, 0x1234);
        assert_eq!(out, vec![0xFD, 0x34, 0x12]);
    }

    #[test]
    fn varint_0xfe_marker() {
        // 0x10000 (just past u16) needs the 0xFE marker + little-endian u32.
        let mut out = Vec::new();
        push_varint(&mut out, 0x0001_0000);
        assert_eq!(out, vec![0xFE, 0x00, 0x00, 0x01, 0x00]);
    }

    #[test]
    fn bytestring_length_128_uses_single_byte_prefix() {
        // Regression: a 128-byte payload must use a one-byte VarInt
        // length (0x80), not the two-byte LEB128 form [0x80, 0x01].
        let payload = vec![0xABu8; 128];
        let bytes = serialise_value(&NeoValue::ByteString(crate::NeoByteString::from_slice(
            &payload,
        )));
        assert_eq!(bytes[0], TAG_BYTESTRING);
        assert_eq!(bytes[1], 0x80); // single-byte length prefix
        assert_eq!(&bytes[2..], &payload[..]);
        assert_eq!(bytes.len(), 130);
    }

    #[test]
    fn tag_values_match_neo_stack_item_type() {
        // Pin the canonical `Neo.VM.Types.StackItemType` discriminants so a
        // future edit cannot silently reintroduce the 0x01/0x21 swap bug.
        assert_eq!(TAG_NULL, 0x00);
        assert_eq!(TAG_BOOLEAN, 0x20);
        assert_eq!(TAG_INTEGER, 0x21);
        assert_eq!(TAG_BYTESTRING, 0x28);
        assert_eq!(TAG_ARRAY, 0x40);
        assert_eq!(TAG_STRUCT, 0x41);
    }

    #[test]
    fn integer_positive() {
        let n = NeoInteger::new(42i32);
        let mut out = Vec::new();
        push_integer(&mut out, &n);
        // tag + varint(len=1) + 0x2A
        assert_eq!(out, vec![0x21, 0x01, 0x2A]);
    }

    #[test]
    fn integer_zero_is_empty() {
        // Neo's `Integer.GetSpan()` returns Empty for zero.
        let mut out = Vec::new();
        push_integer(&mut out, &NeoInteger::new(0i32));
        assert_eq!(out, vec![0x21, 0x00]);
    }

    #[test]
    fn integer_multibyte_is_little_endian() {
        // 1000 = 0x03E8 → little-endian two's-complement minimal form
        // is [0xE8, 0x03]. This is the NEP-17 `Transfer` amount shape;
        // a big-endian bug would emit [0x03, 0xE8] and corrupt the value.
        let mut out = Vec::new();
        push_integer(&mut out, &NeoInteger::new(1000i32));
        assert_eq!(out, vec![0x21, 0x02, 0xE8, 0x03]);

        // 128 needs a zero sign byte so it stays positive: [0x80, 0x00].
        let mut out = Vec::new();
        push_integer(&mut out, &NeoInteger::new(128i32));
        assert_eq!(out, vec![0x21, 0x02, 0x80, 0x00]);
    }

    #[test]
    fn integer_negative_minimum_length() {
        // -1 in two's complement is 0xFF (1 byte, endianness-agnostic).
        let n = NeoInteger::new(-1i32);
        let mut out = Vec::new();
        push_integer(&mut out, &n);
        assert_eq!(out, vec![0x21, 0x01, 0xFF]);

        // -256 = 0xFF00 → little-endian two's-complement is [0x00, 0xFF].
        let mut out = Vec::new();
        push_integer(&mut out, &NeoInteger::new(-256i32));
        assert_eq!(out, vec![0x21, 0x02, 0x00, 0xFF]);
    }

    #[test]
    fn boolean() {
        let mut out = Vec::new();
        push_boolean(&mut out, true);
        assert_eq!(out, vec![0x20, 0x01]);
        push_boolean(&mut out, false);
        assert_eq!(out, vec![0x20, 0x01, 0x20, 0x00]);
    }

    #[test]
    fn empty_array() {
        let arr = NeoArray::<NeoValue>::new();
        let bytes = serialise_array(&arr);
        assert_eq!(bytes, vec![TAG_ARRAY, 0x00]);
    }

    #[test]
    fn array_with_int_and_bool() {
        let mut arr = NeoArray::new();
        arr.push(NeoValue::Integer(NeoInteger::new(7i32)));
        arr.push(NeoValue::Boolean(NeoBoolean::new(true)));
        let bytes = serialise_array(&arr);
        // TAG_ARRAY, count=2, INT(7)={tag,1,7}, BOOL={tag,1}
        assert_eq!(
            bytes,
            vec![TAG_ARRAY, 0x02, TAG_INTEGER, 0x01, 0x07, TAG_BOOLEAN, 0x01]
        );
    }

    #[test]
    fn notification_event_plus_state() {
        let mut arr = NeoArray::new();
        arr.push(NeoValue::Integer(NeoInteger::new(99i32)));
        let event = NeoString::from_str("Transfer");
        let bytes = serialise_notification(&event, &arr);
        // outer array, count=2, "Transfer" as bytestring, state as nested array
        let mut expected = vec![TAG_ARRAY, 0x02];
        // "Transfer" bytestring
        expected.push(TAG_BYTESTRING);
        expected.push(8);
        expected.extend_from_slice(b"Transfer");
        // nested state array: [INT(99)]
        expected.push(TAG_ARRAY);
        expected.push(0x01);
        expected.push(TAG_INTEGER);
        expected.push(0x01);
        expected.push(0x63);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn bytestring_value() {
        let v = NeoValue::ByteString(NeoByteString::from_slice(&[1, 2, 3]));
        let bytes = serialise_value(&v);
        assert_eq!(bytes, vec![TAG_BYTESTRING, 0x03, 0x01, 0x02, 0x03]);
    }
}
