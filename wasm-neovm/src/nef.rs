use std::fs::File;
use std::io::Write;
use std::path::Path;

const NEF_MAGIC: &[u8; 4] = b"NEF3";
const NEF_VERSION: u8 = 0x01;

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = if crc & 1 == 1 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    crc ^ 0xFFFF_FFFF
}

/// Write a NEF artefact combining the provided script and manifest payloads.
pub fn write_nef<P: AsRef<Path>>(
    script: &[u8],
    manifest_json: &str,
    output_path: P,
) -> anyhow::Result<()> {
    if script.is_empty() {
        anyhow::bail!("script payload is empty");
    }

    let mut buffer = Vec::new();
    buffer.extend_from_slice(NEF_MAGIC);
    buffer.push(NEF_VERSION);
    buffer.extend_from_slice(&0u32.to_le_bytes());
    buffer.extend_from_slice(&(script.len() as u32).to_le_bytes());
    buffer.extend_from_slice(script);

    let manifest_bytes = manifest_json.as_bytes();
    buffer.extend_from_slice(&(manifest_bytes.len() as u32).to_le_bytes());
    buffer.extend_from_slice(manifest_bytes);

    let checksum = crc32(&buffer);
    buffer.extend_from_slice(&checksum.to_le_bytes());

    let mut file = File::create(output_path)?;
    file.write_all(&buffer)?;
    Ok(())
}
