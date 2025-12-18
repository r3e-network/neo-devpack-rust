use anyhow::{bail, Context, Result};
use std::io::{Cursor, Read};

/// Bytecode reader helper
#[allow(dead_code)]
pub(super) struct BytecodeReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> BytecodeReader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(bytes),
        }
    }

    pub(super) fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub(super) fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.cursor.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub(super) fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub(super) fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub(super) fn read_u128(&mut self) -> Result<u128> {
        let mut buf = [0u8; 16];
        self.cursor.read_exact(&mut buf)?;
        Ok(u128::from_le_bytes(buf))
    }

    pub(super) fn read_uleb128(&mut self) -> Result<u64> {
        let mut result: u64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7f) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 64 {
                bail!("ULEB128 overflow");
            }
        }
        Ok(result)
    }

    pub(super) fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub(super) fn read_string(&mut self) -> Result<String> {
        let len = self.read_uleb128()? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes).context("Invalid UTF-8 in string")
    }

    pub(super) fn bytes(&self) -> &'a [u8] {
        self.cursor.get_ref()
    }

    pub(super) fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub(super) fn remaining(&self) -> usize {
        let pos = self.cursor.position() as usize;
        let len = self.cursor.get_ref().len();
        len.saturating_sub(pos)
    }
}
