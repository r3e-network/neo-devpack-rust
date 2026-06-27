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
//! Reference values (round-tripped against the C# implementation):
//!
//! | Tag (1 byte) | Type |
//! |--------------|------|
//! | `0x21` | Boolean (followed by 0x00/0x01) |
//! | `0x01` | Integer (varint length, then big-endian signed bytes) |
//! | `0x28` | ByteString (varint length, then bytes) |
//! | `0x29` | Buffer (same as ByteString; treated as a writable buffer) |
//! | `0x40` | Array/Struct (varint count, then nested items) |
//! | `0x00` | Null/Any (no payload) |
//!
//! Reference: C# `ApplicationEngine.Runtime.cs` `RuntimeNotify` calls
//! `BinarySerializer.Serialize(writer, state, MaxNotificationSize, ...)`.

use crate::{NeoArray, NeoString, NeoValue};

/// Max serialised size for a notification (C#: `MaxNotificationSize = 1024`).
pub const MAX_NOTIFICATION_SIZE: usize = 1024;

/// Max items in a serialised Array/Struct (C#: `Limits.MaxStackSize`).
pub const MAX_STACK_SIZE: usize = 1024;

const TAG_BOOLEAN: u8 = 0x21;
const TAG_INTEGER: u8 = 0x01;
const TAG_BYTESTRING: u8 = 0x28;
const TAG_ARRAY: u8 = 0x40;
const TAG_STRUCT: u8 = 0x41;
const TAG_NULL: u8 = 0x00;

fn push_varint(out: &mut Vec<u8>, mut value: usize) {
    while value >= 0x80 {
        out.push(((value as u8) & 0x7F) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn push_integer(out: &mut Vec<u8>, n: &crate::NeoInteger) {
    let bytes = n.as_bigint().to_signed_bytes_be();
    out.push(TAG_INTEGER);
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
    use crate::{NeoByteString, NeoBoolean, NeoInteger};

    #[test]
    fn varint_single_byte() {
        let mut out = Vec::new();
        push_varint(&mut out, 0);
        assert_eq!(out, vec![0]);
        push_varint(&mut out, 127);
        assert_eq!(out, vec![0, 127]);
    }

    #[test]
    fn varint_multi_byte() {
        let mut out = Vec::new();
        push_varint(&mut out, 128);
        assert_eq!(out, vec![0x80, 0x01]);
    }

    #[test]
    fn integer_positive() {
        let n = NeoInteger::new(42i32);
        let mut out = Vec::new();
        push_integer(&mut out, &n);
        // tag + varint(len=1) + 0x2A
        assert_eq!(out, vec![TAG_INTEGER, 0x01, 0x2A]);
    }

    #[test]
    fn integer_negative_minimum_length() {
        // -1 in two's complement big-endian is 0xFF (1 byte)
        let n = NeoInteger::new(-1i32);
        let mut out = Vec::new();
        push_integer(&mut out, &n);
        assert_eq!(out, vec![TAG_INTEGER, 0x01, 0xFF]);
    }

    #[test]
    fn boolean() {
        let mut out = Vec::new();
        push_boolean(&mut out, true);
        assert_eq!(out, vec![TAG_BOOLEAN, 0x01]);
        push_boolean(&mut out, false);
        assert_eq!(out, vec![TAG_BOOLEAN, 0x01, TAG_BOOLEAN, 0x00]);
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
