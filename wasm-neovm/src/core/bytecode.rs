// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Bytecode encoding utilities.
//!
//! Compact NeoVM `PUSHINT*` integer encoding shared by the translator's
//! numeric lowering and the fuzz harness.

/// Calculate the size of an integer when encoded as a NeoVM PUSHINT* opcode.
///
/// Neo N3 opcodes: PUSHM1=0x0F, PUSH0=0x10, PUSH1..PUSH16=0x11..0x20,
/// PUSHINT8=0x00, PUSHINT16=0x01, PUSHINT32=0x02, PUSHINT64=0x03.
pub fn encoded_int_size(value: i64) -> usize {
    match value {
        -1..=16 => 1,                     // PUSHM1 / PUSH0-PUSH16 (single-byte opcodes)
        -128..=-2 | 17..=127 => 2,        // PUSHINT8 (opcode + 1 byte)
        -32768..=-129 | 128..=32767 => 3, // PUSHINT16 (opcode + 2 bytes)
        -2147483648..=-32769 | 32768..=2147483647 => 5, // PUSHINT32 (opcode + 4 bytes)
        _ => 9,                           // PUSHINT64 (opcode + 8 bytes)
    }
}

/// Encode an integer as the most compact NeoVM PUSHINT* opcode.
///
/// Neo N3 opcodes: PUSHM1=0x0F, PUSH0=0x10, PUSH1..PUSH16=0x11..0x20,
/// PUSHINT8=0x00, PUSHINT16=0x01, PUSHINT32=0x02, PUSHINT64=0x03.
pub fn encode_int(value: i64) -> Vec<u8> {
    if value == -1 {
        // PUSHM1 = 0x0F
        vec![0x0F]
    } else if (0..=16).contains(&value) {
        // PUSH0=0x10, PUSH1=0x11, ..., PUSH16=0x20
        vec![0x10 + value as u8]
    } else if (-128..=127).contains(&value) {
        // PUSHINT8
        vec![0x00, value as i8 as u8]
    } else if (-32768..=32767).contains(&value) {
        // PUSHINT16
        let bytes = (value as i16).to_le_bytes();
        vec![0x01, bytes[0], bytes[1]]
    } else if (-2147483648..=2147483647).contains(&value) {
        // PUSHINT32
        let bytes = (value as i32).to_le_bytes();
        vec![0x02, bytes[0], bytes[1], bytes[2], bytes[3]]
    } else {
        // PUSHINT64
        let bytes = value.to_le_bytes();
        vec![
            0x03, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_int() {
        // PUSHM1 = 0x0F
        assert_eq!(encode_int(-1), vec![0x0F]);

        // PUSH0=0x10, PUSH1=0x11, ..., PUSH16=0x20
        assert_eq!(encode_int(0), vec![0x10]);
        assert_eq!(encode_int(1), vec![0x11]);
        assert_eq!(encode_int(16), vec![0x20]);

        // PUSHINT8 (opcode 0x00 + 1-byte signed value)
        assert_eq!(encode_int(17), vec![0x00, 0x11]);
        assert_eq!(encode_int(127), vec![0x00, 0x7F]);
        assert_eq!(encode_int(-2), vec![0x00, 0xFE]);
        assert_eq!(encode_int(-128), vec![0x00, 0x80]);

        // PUSHINT16 (opcode 0x01 + 2-byte LE signed value)
        assert_eq!(encode_int(128), vec![0x01, 0x80, 0x00]);
        assert_eq!(encode_int(32767), vec![0x01, 0xFF, 0x7F]);
        assert_eq!(encode_int(-129), vec![0x01, 0x7F, 0xFF]);
        assert_eq!(encode_int(-32768), vec![0x01, 0x00, 0x80]);
    }

    #[test]
    fn test_encoded_int_size() {
        // Single-byte opcodes: PUSHM1, PUSH0-PUSH16
        assert_eq!(encoded_int_size(-1), 1);
        assert_eq!(encoded_int_size(0), 1);
        assert_eq!(encoded_int_size(16), 1);

        // PUSHINT8: opcode + 1 byte
        assert_eq!(encoded_int_size(17), 2);
        assert_eq!(encoded_int_size(127), 2);
        assert_eq!(encoded_int_size(-2), 2);
        assert_eq!(encoded_int_size(-128), 2);

        // PUSHINT16: opcode + 2 bytes
        assert_eq!(encoded_int_size(128), 3);
        assert_eq!(encoded_int_size(-129), 3);

        // PUSHINT32: opcode + 4 bytes
        assert_eq!(encoded_int_size(32768), 5);
        assert_eq!(encoded_int_size(-32769), 5);

        // PUSHINT64: opcode + 8 bytes
        assert_eq!(encoded_int_size(i64::MAX), 9);
        assert_eq!(encoded_int_size(i64::MIN), 9);
    }
}
