// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Encoding utilities for NeoVM types
//!
//! This module provides encoding/decoding for types used in the
//! NeoVM ecosystem.

pub mod error;
pub mod primitives;
pub mod reader;
pub mod writer;

pub use error::{EncodingError, EncodingResult};
pub use primitives::{
    decode_bool, decode_bytes, decode_string, decode_varint, encode_bool, encode_bytes,
    encode_string, encode_varint,
};
pub use reader::ByteReader;
pub use writer::ByteWriter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encoding() {
        // Small values
        assert_eq!(encode_varint(0), vec![0]);
        assert_eq!(encode_varint(252), vec![252]);

        // 16-bit values
        assert_eq!(encode_varint(253), vec![0xFD, 0xFD, 0x00]);
        assert_eq!(encode_varint(1000), vec![0xFD, 0xE8, 0x03]);

        // 32-bit values
        assert_eq!(encode_varint(65536), vec![0xFE, 0x00, 0x00, 0x01, 0x00]);

        // 64-bit values
        assert_eq!(
            encode_varint(u64::MAX),
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        );
    }

    #[test]
    fn test_varint_roundtrip() {
        let test_values = [0u64, 100, 253, 1000, 65536, 100000, u64::MAX];

        for value in test_values {
            let encoded = encode_varint(value);
            let (decoded, _) = decode_varint(&encoded).unwrap();
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_string_encoding() {
        let test_strings = ["", "hello", "Hello, 世界!"];

        for s in test_strings {
            let encoded = encode_string(s);
            let (decoded, _) = decode_string(&encoded).unwrap();
            assert_eq!(s, decoded);
        }
    }

    #[test]
    fn test_byte_writer() {
        let mut writer = ByteWriter::with_capacity(32);
        writer
            .write_u8(0x01)
            .write_u16_le(0x1234)
            .write_u32_le(0x567890AB)
            .write_string("test");

        let bytes = writer.finish();
        assert!(!bytes.is_empty());

        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_u8().unwrap(), 0x01);
        assert_eq!(reader.read_u16_le().unwrap(), 0x1234);
        assert_eq!(reader.read_u32_le().unwrap(), 0x567890AB);
        assert_eq!(reader.read_string().unwrap(), "test");
    }

    #[test]
    fn test_byte_reader_seek() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = ByteReader::new(&data);

        assert_eq!(reader.read_u8().unwrap(), 0x01);
        reader.seek(3).unwrap();
        assert_eq!(reader.read_u8().unwrap(), 0x04);
    }
}
