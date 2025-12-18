use core::slice;
use neo_devpack::prelude::*;

use crate::storage::*;
use crate::types::WalletConfig;
use crate::utils::addresses_equal;

pub fn load_config(ctx: &NeoStorageContext) -> Option<WalletConfig> {
    let threshold = read_i64(ctx, CONFIG_THRESHOLD_KEY)?;
    let owner_count = read_u16(ctx, CONFIG_OWNER_COUNT_KEY)? as usize;
    let mut owners = Vec::with_capacity(owner_count);
    for index in 0..owner_count {
        let key = config_owner_key(index as u16);
        let bytes = read_storage_bytes(ctx, &key)?;
        if bytes.len() != 20 {
            return None;
        }
        owners.push(NeoByteString::from_slice(&bytes));
    }
    Some(WalletConfig { owners, threshold })
}

pub fn store_config(ctx: &NeoStorageContext, cfg: &WalletConfig) -> NeoResult<()> {
    write_i64(ctx, CONFIG_THRESHOLD_KEY, cfg.threshold)?;
    write_u16(ctx, CONFIG_OWNER_COUNT_KEY, cfg.owners.len() as u16)?;
    for (index, owner) in cfg.owners.iter().enumerate() {
        let key = config_owner_key(index as u16);
        write_bytes(ctx, &key, owner.as_slice())?;
    }
    Ok(())
}

pub fn is_owner(cfg: &WalletConfig, owner: &NeoByteString) -> bool {
    cfg.owners
        .iter()
        .any(|existing| addresses_equal(existing, owner))
}

pub fn read_owners(ptr: i64, count: i64) -> Option<Vec<NeoByteString>> {
    if ptr == 0 || count <= 0 {
        return None;
    }
    let count = count as usize;
    let total = count.checked_mul(20)?;
    let bytes = unsafe { slice::from_raw_parts(ptr as *const u8, total) };
    let mut owners = Vec::with_capacity(count);
    for chunk in bytes.chunks_exact(20) {
        owners.push(NeoByteString::from_slice(chunk));
    }
    Some(owners)
}

pub fn encode_config_json(cfg: &WalletConfig) -> NeoByteString {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(32 + cfg.owners.len() * 42);
    out.push_str("{\"threshold\":");
    out.push_str(&cfg.threshold.to_string());
    out.push_str(",\"owners\":[");
    for (idx, owner) in cfg.owners.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str("\"0x");
        for byte in owner.as_slice() {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0F) as usize] as char);
        }
        out.push('\"');
    }
    out.push_str("]}");
    NeoByteString::from_slice(out.as_bytes())
}
