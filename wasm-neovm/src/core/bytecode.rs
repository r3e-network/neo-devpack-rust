// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Bytecode manipulation utilities
//!
//! This module provides utilities for working with NeoVM bytecode.

use crate::types::BytecodeOffset;

/// A builder for constructing NeoVM bytecode with patching support
#[derive(Debug, Clone, Default)]
pub struct BytecodeBuilder {
    bytecode: Vec<u8>,
    patches: Vec<(BytecodeOffset, Vec<u8>)>,
}

impl BytecodeBuilder {
    /// Create a new bytecode builder with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bytecode: Vec::with_capacity(capacity),
            patches: Vec::new(),
        }
    }

    /// Emit a single byte
    pub fn emit(&mut self, byte: u8) -> &mut Self {
        self.bytecode.push(byte);
        self
    }

    /// Emit multiple bytes
    pub fn emit_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.bytecode.extend_from_slice(bytes);
        self
    }

    /// Emit a u16 in little-endian format
    pub fn emit_u16(&mut self, value: u16) -> &mut Self {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Emit a u32 in little-endian format
    pub fn emit_u32(&mut self, value: u32) -> &mut Self {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Emit a u64 in little-endian format
    pub fn emit_u64(&mut self, value: u64) -> &mut Self {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Emit a placeholder of the given size, returning its offset
    pub fn emit_placeholder(&mut self, size: usize) -> BytecodeOffset {
        let offset = BytecodeOffset::new(self.bytecode.len());
        self.bytecode.resize(self.bytecode.len() + size, 0);
        offset
    }

    /// Schedule a patch at the given offset
    pub fn schedule_patch(&mut self, offset: BytecodeOffset, data: Vec<u8>) -> &mut Self {
        self.patches.push((offset, data));
        self
    }

    /// Patch a u16 at the given offset
    pub fn patch_u16(&mut self, offset: BytecodeOffset, value: u16) -> &mut Self {
        let start = offset.value();
        if start + 2 <= self.bytecode.len() {
            let bytes = value.to_le_bytes();
            self.bytecode[start] = bytes[0];
            self.bytecode[start + 1] = bytes[1];
        }
        self
    }

    /// Patch a u32 at the given offset
    pub fn patch_u32(&mut self, offset: BytecodeOffset, value: u32) -> &mut Self {
        let start = offset.value();
        if start + 4 <= self.bytecode.len() {
            let bytes = value.to_le_bytes();
            self.bytecode[start..start + 4].copy_from_slice(&bytes);
        }
        self
    }

    /// Apply all scheduled patches
    pub fn apply_patches(&mut self) -> &mut Self {
        for (offset, data) in self.patches.drain(..) {
            let start = offset.value();
            if start + data.len() <= self.bytecode.len() {
                self.bytecode[start..start + data.len()].copy_from_slice(&data);
            }
        }
        self
    }

    /// Get the current offset
    pub fn offset(&self) -> BytecodeOffset {
        BytecodeOffset::new(self.bytecode.len())
    }

    /// Get the current bytecode length
    pub fn len(&self) -> usize {
        self.bytecode.len()
    }

    /// Check if the bytecode is empty
    pub fn is_empty(&self) -> bool {
        self.bytecode.is_empty()
    }

    /// Build the final bytecode, applying patches
    pub fn build(mut self) -> Vec<u8> {
        self.apply_patches();
        self.bytecode
    }

    /// Get a reference to the bytecode (without applying patches)
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytecode
    }

    /// Clear the builder
    pub fn clear(&mut self) {
        self.bytecode.clear();
        self.patches.clear();
    }
}

/// A view into bytecode at a specific offset
#[derive(Debug, Clone, Copy)]
pub struct BytecodeView<'a> {
    bytecode: &'a [u8],
    offset: BytecodeOffset,
}

impl<'a> BytecodeView<'a> {
    /// Create a new bytecode view
    pub fn new(bytecode: &'a [u8], offset: BytecodeOffset) -> Self {
        Self { bytecode, offset }
    }

    /// Get the byte at the current offset
    pub fn peek(&self) -> Option<u8> {
        self.bytecode.get(self.offset.value()).copied()
    }

    /// Get bytes at the current offset
    pub fn peek_bytes(&self, count: usize) -> Option<&[u8]> {
        let start = self.offset.value();
        self.bytecode.get(start..start + count)
    }

    /// Read a u16 in little-endian format
    pub fn read_u16(&self) -> Option<u16> {
        let bytes = self.peek_bytes(2)?;
        Some(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a u32 in little-endian format
    pub fn read_u32(&self) -> Option<u32> {
        let bytes = self.peek_bytes(4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Advance the offset by the given amount
    pub fn advance(&mut self, amount: usize) -> &mut Self {
        self.offset = BytecodeOffset::new(self.offset.value() + amount);
        self
    }

    /// Get the current offset
    pub fn offset(&self) -> BytecodeOffset {
        self.offset
    }

    /// Check if at end of bytecode
    pub fn is_at_end(&self) -> bool {
        self.offset.value() >= self.bytecode.len()
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> &[u8] {
        &self.bytecode[self.offset.value()..]
    }
}

/// Iterator over bytecode chunks
pub struct BytecodeChunks<'a> {
    bytecode: &'a [u8],
    offset: usize,
    chunk_size: usize,
}

impl<'a> BytecodeChunks<'a> {
    /// Create a new chunk iterator
    pub fn new(bytecode: &'a [u8], chunk_size: usize) -> Self {
        Self {
            bytecode,
            offset: 0,
            chunk_size,
        }
    }
}

impl<'a> Iterator for BytecodeChunks<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.bytecode.len() {
            return None;
        }

        let end = (self.offset + self.chunk_size).min(self.bytecode.len());
        let chunk = &self.bytecode[self.offset..end];
        self.offset = end;
        Some(chunk)
    }
}

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
    fn test_bytecode_builder() {
        let mut builder = BytecodeBuilder::with_capacity(32);
        builder
            .emit(0x01)
            .emit_bytes(&[0x02, 0x03])
            .emit_u16(0x1234);

        assert_eq!(builder.as_bytes(), &[0x01, 0x02, 0x03, 0x34, 0x12]);
    }

    #[test]
    fn test_bytecode_builder_patch() {
        let mut builder = BytecodeBuilder::default();
        let placeholder = builder.emit_placeholder(2);
        builder.emit(0xFF);

        builder.patch_u16(placeholder, 0x1234);
        assert_eq!(builder.build(), &[0x34, 0x12, 0xFF]);
    }

    #[test]
    fn test_bytecode_view() {
        let bytecode = vec![0x01, 0x02, 0x34, 0x12, 0xFF];
        let mut view = BytecodeView::new(&bytecode, BytecodeOffset::new(0));

        assert_eq!(view.peek(), Some(0x01));
        assert_eq!(view.read_u16(), Some(0x0201));

        view.advance(2);
        assert_eq!(view.read_u16(), Some(0x1234));
    }

    #[test]
    fn test_bytecode_chunks() {
        let bytecode = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let chunks: Vec<_> = BytecodeChunks::new(&bytecode, 3).collect();

        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], &[1, 2, 3]);
        assert_eq!(chunks[1], &[4, 5, 6]);
        assert_eq!(chunks[2], &[7, 8, 9]);
        assert_eq!(chunks[3], &[10]);
    }

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
