use super::MOVE_MAGIC;

/// Validate Move bytecode without full parsing
pub fn validate_move_bytecode(bytes: &[u8]) -> bool {
    // Check magic bytes
    if bytes.len() < 4 {
        return false;
    }
    // Move module magic: 0xa1, 0x1c, 0xeb, 0x0b
    bytes[0..4] == MOVE_MAGIC
}
