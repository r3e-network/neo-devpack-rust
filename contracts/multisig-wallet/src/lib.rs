use core::slice;
use neo_devpack::prelude::*;

const CONFIG_THRESHOLD_KEY: &[u8] = b"cfg:threshold";
const CONFIG_OWNER_COUNT_KEY: &[u8] = b"cfg:owners";
const CONFIG_OWNER_PREFIX: &[u8] = b"cfg:owner:";
const PROPOSAL_COUNTER_KEY: &[u8] = b"proposal:counter";
const PROPOSAL_PREFIX: &[u8] = b"proposal:";
const PROPOSER_SUFFIX: &[u8] = b":proposer";
const TARGET_SUFFIX: &[u8] = b":target";
const METHOD_SUFFIX: &[u8] = b":method";
const ARG_SUFFIX: &[u8] = b":args";
const APPROVAL_COUNT_SUFFIX: &[u8] = b":approvals";
const APPROVAL_PREFIX: &[u8] = b":approval:";
const EXECUTED_SUFFIX: &[u8] = b":executed";

#[derive(Clone, PartialEq, Eq)]
struct WalletConfig {
    owners: Vec<NeoByteString>,
    threshold: i64,
}

#[derive(Clone)]
struct Proposal {
    proposer: NeoByteString,
    target: NeoByteString,
    method: String,
    arguments: Vec<CallArgument>,
    approvals: Vec<NeoByteString>,
    executed: bool,
}

#[derive(Clone)]
enum CallArgument {
    Integer(i64),
    Boolean(bool),
    ByteString(Vec<u8>),
    String(String),
}

neo_manifest_overlay!(
    r#"{
    "name": "SampleMultisig",
    "features": { "storage": true }
}"#
);

#[neo_event]
pub struct ProposalCreated {
    pub proposal_id: NeoInteger,
    pub proposer: NeoByteString,
    pub target: NeoByteString,
    pub method: NeoString,
}

#[neo_event]
pub struct ProposalExecuted {
    pub proposal_id: NeoInteger,
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn getConfig() -> NeoByteString {
    storage_context()
        .and_then(|ctx| load_config(&ctx))
        .map(|cfg| encode_config_json(&cfg))
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[no_mangle]
pub extern "C" fn configure(owners_ptr: i64, owner_count: i64, threshold: i64) -> i64 {
    if owner_count <= 0 || threshold <= 0 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    if load_config(&ctx).is_some() {
        return 0;
    }
    let Some(owners) = read_owners(owners_ptr, owner_count) else {
        return 0;
    };
    if owners.is_empty() || threshold > owners.len() as i64 {
        return 0;
    }
    let config = WalletConfig { owners, threshold };
    store_config(&ctx, &config).map(|_| 1).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn propose(
    signer_ptr: i64,
    signer_len: i64,
    target_ptr: i64,
    target_len: i64,
    method_ptr: i64,
    method_len: i64,
    args_ptr: i64,
    args_len: i64,
) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(config) = load_config(&ctx) else {
        return 0;
    };
    let Some(signer) = read_address(signer_ptr, signer_len) else {
        return 0;
    };
    if !is_owner(&config, &signer) || !ensure_witness(&signer) {
        return 0;
    }
    let Some(target) = read_address(target_ptr, target_len) else {
        return 0;
    };
    let Some(method) = read_string(method_ptr, method_len) else {
        return 0;
    };
    let Some(arguments) = decode_arguments(args_ptr, args_len) else {
        return 0;
    };
    let proposal_id = match next_proposal_id(&ctx) {
        Some(id) => id,
        None => return 0,
    };
    let approvals = vec![signer.clone()];
    let proposal = Proposal {
        proposer: signer.clone(),
        target: target.clone(),
        method: method.clone(),
        arguments,
        approvals,
        executed: false,
    };
    if store_proposal(&ctx, proposal_id, &proposal).is_err() {
        return 0;
    }
    ProposalCreated {
        proposer: signer,
        proposal_id: NeoInteger::new(proposal_id),
        target,
        method: NeoString::from_str(&method),
    }
    .emit()
    .ok();
    proposal_id
}

#[no_mangle]
pub extern "C" fn approve(signer_ptr: i64, signer_len: i64, proposal_id: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(mut proposal) = load_proposal(&ctx, proposal_id) else {
        return 0;
    };
    let Some(config) = load_config(&ctx) else {
        return 0;
    };
    if proposal.executed {
        return 0;
    }
    let Some(signer) = read_address(signer_ptr, signer_len) else {
        return 0;
    };
    if !is_owner(&config, &signer) || !ensure_witness(&signer) {
        return 0;
    }
    if proposal
        .approvals
        .iter()
        .any(|existing| addresses_equal(existing, &signer))
    {
        return 0;
    }
    proposal.approvals.push(signer);
    store_proposal(&ctx, proposal_id, &proposal)
        .map(|_| 1)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn execute(signer_ptr: i64, signer_len: i64, proposal_id: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(mut proposal) = load_proposal(&ctx, proposal_id) else {
        return 0;
    };
    let Some(config) = load_config(&ctx) else {
        return 0;
    };
    if proposal.executed {
        return 0;
    }
    let Some(signer) = read_address(signer_ptr, signer_len) else {
        return 0;
    };
    if !is_owner(&config, &signer) || !ensure_witness(&signer) {
        return 0;
    }
    if (proposal.approvals.len() as i64) < config.threshold {
        return 0;
    }
    let args = match build_argument_array(&proposal.arguments) {
        Some(array) => array,
        None => return 0,
    };
    if NeoContractRuntime::call(
        &proposal.target,
        &NeoString::from_str(&proposal.method),
        &args,
    )
    .is_err()
    {
        return 0;
    }
    proposal.executed = true;
    if store_proposal(&ctx, proposal_id, &proposal).is_err() {
        return 0;
    }
    remove_proposal_entries(&ctx, proposal_id).ok();
    ProposalExecuted {
        proposal_id: NeoInteger::new(proposal_id),
    }
    .emit()
    .ok();
    1
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn load_config(ctx: &NeoStorageContext) -> Option<WalletConfig> {
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

fn store_config(ctx: &NeoStorageContext, cfg: &WalletConfig) -> NeoResult<()> {
    write_i64(ctx, CONFIG_THRESHOLD_KEY, cfg.threshold)?;
    write_u16(ctx, CONFIG_OWNER_COUNT_KEY, cfg.owners.len() as u16)?;
    for (index, owner) in cfg.owners.iter().enumerate() {
        let key = config_owner_key(index as u16);
        write_bytes(ctx, &key, owner.as_slice())?;
    }
    Ok(())
}

fn is_owner(cfg: &WalletConfig, owner: &NeoByteString) -> bool {
    cfg.owners
        .iter()
        .any(|existing| addresses_equal(existing, owner))
}

fn next_proposal_id(ctx: &NeoStorageContext) -> Option<i64> {
    let current = read_i64(ctx, PROPOSAL_COUNTER_KEY).unwrap_or(0);
    let next = current.checked_add(1)?;
    write_i64(ctx, PROPOSAL_COUNTER_KEY, next).ok()?;
    Some(next)
}

fn load_proposal(ctx: &NeoStorageContext, id: i64) -> Option<Proposal> {
    let proposer = read_proposal_address(ctx, id, PROPOSER_SUFFIX)?;
    let target = read_proposal_address(ctx, id, TARGET_SUFFIX)?;
    let method = read_proposal_string(ctx, id, METHOD_SUFFIX)?;
    let arguments = read_proposal_arguments(ctx, id)?;
    let approvals = read_proposal_approvals(ctx, id)?;
    let executed = read_proposal_bool(ctx, id, EXECUTED_SUFFIX)?;
    Some(Proposal {
        proposer,
        target,
        method,
        arguments,
        approvals,
        executed,
    })
}

fn store_proposal(ctx: &NeoStorageContext, id: i64, proposal: &Proposal) -> NeoResult<()> {
    write_bytes(
        ctx,
        &proposal_field_key(id, PROPOSER_SUFFIX),
        proposal.proposer.as_slice(),
    )?;
    write_bytes(
        ctx,
        &proposal_field_key(id, TARGET_SUFFIX),
        proposal.target.as_slice(),
    )?;
    write_string(
        ctx,
        &proposal_field_key(id, METHOD_SUFFIX),
        &proposal.method,
    )?;
    write_bytes(
        ctx,
        &proposal_field_key(id, ARG_SUFFIX),
        &encode_arguments(&proposal.arguments),
    )?;
    write_u16(
        ctx,
        &proposal_field_key(id, APPROVAL_COUNT_SUFFIX),
        proposal.approvals.len() as u16,
    )?;
    for (idx, approval) in proposal.approvals.iter().enumerate() {
        write_bytes(
            ctx,
            &proposal_approval_key(id, idx as u16),
            approval.as_slice(),
        )?;
    }
    write_bool(
        ctx,
        &proposal_field_key(id, EXECUTED_SUFFIX),
        proposal.executed,
    )?;
    Ok(())
}

fn remove_proposal_entries(ctx: &NeoStorageContext, id: i64) -> NeoResult<()> {
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, PROPOSER_SUFFIX)),
    );
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, TARGET_SUFFIX)),
    );
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, METHOD_SUFFIX)),
    );
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, ARG_SUFFIX)),
    );
    let count = read_u16(ctx, &proposal_field_key(id, APPROVAL_COUNT_SUFFIX)).unwrap_or(0);
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, APPROVAL_COUNT_SUFFIX)),
    );
    for idx in 0..count {
        let _ = NeoStorage::delete(
            ctx,
            &NeoByteString::from_slice(&proposal_approval_key(id, idx)),
        );
    }
    let _ = NeoStorage::delete(
        ctx,
        &NeoByteString::from_slice(&proposal_field_key(id, EXECUTED_SUFFIX)),
    );
    Ok(())
}

fn read_proposal_address(ctx: &NeoStorageContext, id: i64, suffix: &[u8]) -> Option<NeoByteString> {
    let bytes = read_storage_bytes(ctx, &proposal_field_key(id, suffix))?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

fn read_proposal_string(ctx: &NeoStorageContext, id: i64, suffix: &[u8]) -> Option<String> {
    read_storage_string(ctx, &proposal_field_key(id, suffix))
}

fn read_proposal_bool(ctx: &NeoStorageContext, id: i64, suffix: &[u8]) -> Option<bool> {
    read_bool(ctx, &proposal_field_key(id, suffix))
}

fn read_proposal_arguments(ctx: &NeoStorageContext, id: i64) -> Option<Vec<CallArgument>> {
    let bytes = read_storage_bytes(ctx, &proposal_field_key(id, ARG_SUFFIX)).unwrap_or_default();
    decode_arguments_from_bytes(&bytes)
}

fn read_proposal_approvals(ctx: &NeoStorageContext, id: i64) -> Option<Vec<NeoByteString>> {
    let count = read_u16(ctx, &proposal_field_key(id, APPROVAL_COUNT_SUFFIX)).unwrap_or(0);
    let mut approvals = Vec::with_capacity(count as usize);
    for idx in 0..count {
        let bytes = read_storage_bytes(ctx, &proposal_approval_key(id, idx))?;
        if bytes.len() != 20 {
            return None;
        }
        approvals.push(NeoByteString::from_slice(&bytes));
    }
    Some(approvals)
}

fn read_owners(ptr: i64, count: i64) -> Option<Vec<NeoByteString>> {
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

const ARG_INTEGER: u8 = 0;
const ARG_BOOL: u8 = 1;
const ARG_BYTES: u8 = 2;
const ARG_STRING: u8 = 3;

fn decode_arguments(ptr: i64, len: i64) -> Option<Vec<CallArgument>> {
    if len <= 0 {
        return Some(Vec::new());
    }
    let bytes = read_bytes(ptr, len)?;
    decode_arguments_from_bytes(&bytes)
}

fn decode_arguments_from_bytes(bytes: &[u8]) -> Option<Vec<CallArgument>> {
    if bytes.is_empty() {
        return Some(Vec::new());
    }
    let mut cursor = 0usize;
    let count = bytes[cursor] as usize;
    cursor += 1;
    let mut parsed = Vec::with_capacity(count);
    for _ in 0..count {
        if cursor >= bytes.len() {
            return None;
        }
        let kind = bytes[cursor];
        cursor += 1;
        if cursor + 2 > bytes.len() {
            return None;
        }
        let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        if cursor + length > bytes.len() {
            return None;
        }
        let segment = &bytes[cursor..cursor + length];
        cursor += length;
        let arg = match kind {
            ARG_INTEGER => {
                if length != 8 {
                    return None;
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(segment);
                CallArgument::Integer(i64::from_le_bytes(buf))
            }
            ARG_BOOL => {
                if length != 1 {
                    return None;
                }
                CallArgument::Boolean(segment[0] != 0)
            }
            ARG_BYTES => CallArgument::ByteString(segment.to_vec()),
            ARG_STRING => match core::str::from_utf8(segment) {
                Ok(value) => CallArgument::String(value.to_string()),
                Err(_) => return None,
            },
            _ => return None,
        };
        parsed.push(arg);
    }
    Some(parsed)
}

fn encode_arguments(arguments: &[CallArgument]) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.push(arguments.len() as u8);
    for argument in arguments {
        match argument {
            CallArgument::Integer(value) => {
                buffer.push(ARG_INTEGER);
                buffer.extend_from_slice(&(8u16).to_le_bytes());
                buffer.extend_from_slice(&value.to_le_bytes());
            }
            CallArgument::Boolean(value) => {
                buffer.push(ARG_BOOL);
                buffer.extend_from_slice(&(1u16).to_le_bytes());
                buffer.push(*value as u8);
            }
            CallArgument::ByteString(bytes) => {
                buffer.push(ARG_BYTES);
                buffer.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
                buffer.extend_from_slice(bytes);
            }
            CallArgument::String(value) => {
                buffer.push(ARG_STRING);
                buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
                buffer.extend_from_slice(value.as_bytes());
            }
        }
    }
    buffer
}

fn build_argument_array(arguments: &[CallArgument]) -> Option<NeoArray<NeoValue>> {
    let mut values = Vec::with_capacity(arguments.len());
    for argument in arguments {
        let value = match argument {
            CallArgument::Integer(v) => NeoValue::from(NeoInteger::new(*v)),
            CallArgument::Boolean(v) => NeoValue::from(NeoBoolean::new(*v)),
            CallArgument::String(v) => NeoValue::from(NeoString::from_str(v)),
            CallArgument::ByteString(bytes) => NeoValue::from(NeoByteString::from_slice(bytes)),
        };
        values.push(value);
    }
    Some(NeoArray::from_vec(values))
}

fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    let bytes = read_bytes(ptr, len)?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

fn read_string(ptr: i64, len: i64) -> Option<String> {
    let bytes = read_bytes(ptr, len)?;
    String::from_utf8(bytes).ok()
}

fn read_bytes(ptr: i64, len: i64) -> Option<Vec<u8>> {
    if ptr == 0 || len <= 0 {
        return None;
    }
    let slice = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
    Some(slice.to_vec())
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn addresses_equal(left: &NeoByteString, right: &NeoByteString) -> bool {
    left.as_slice() == right.as_slice()
}

fn write_bytes(ctx: &NeoStorageContext, key: &[u8], bytes: &[u8]) -> NeoResult<()> {
    let key_bytes = NeoByteString::from_slice(key);
    let value = NeoByteString::from_slice(bytes);
    NeoStorage::put(ctx, &key_bytes, &value)
}

fn read_storage_bytes(ctx: &NeoStorageContext, key: &[u8]) -> Option<Vec<u8>> {
    let key_bytes = NeoByteString::from_slice(key);
    let bytes = NeoStorage::get(ctx, &key_bytes).ok()?;
    if bytes.is_empty() {
        return None;
    }
    Some(bytes.as_slice().to_vec())
}

fn write_i64(ctx: &NeoStorageContext, key: &[u8], value: i64) -> NeoResult<()> {
    write_bytes(ctx, key, &value.to_le_bytes())
}

fn read_i64(ctx: &NeoStorageContext, key: &[u8]) -> Option<i64> {
    let bytes = read_storage_bytes(ctx, key)?;
    if bytes.len() != 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes);
    Some(i64::from_le_bytes(buf))
}

fn write_u16(ctx: &NeoStorageContext, key: &[u8], value: u16) -> NeoResult<()> {
    write_bytes(ctx, key, &value.to_le_bytes())
}

fn read_u16(ctx: &NeoStorageContext, key: &[u8]) -> Option<u16> {
    let bytes = read_storage_bytes(ctx, key)?;
    if bytes.len() != 2 {
        return None;
    }
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn write_bool(ctx: &NeoStorageContext, key: &[u8], value: bool) -> NeoResult<()> {
    write_bytes(ctx, key, &[value as u8])
}

fn read_bool(ctx: &NeoStorageContext, key: &[u8]) -> Option<bool> {
    let bytes = read_storage_bytes(ctx, key)?;
    if bytes.len() != 1 {
        return None;
    }
    Some(bytes[0] != 0)
}

fn write_string(ctx: &NeoStorageContext, key: &[u8], value: &str) -> NeoResult<()> {
    let mut buffer = Vec::with_capacity(2 + value.len());
    buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
    buffer.extend_from_slice(value.as_bytes());
    write_bytes(ctx, key, &buffer)
}

fn read_storage_string(ctx: &NeoStorageContext, key: &[u8]) -> Option<String> {
    let bytes = read_storage_bytes(ctx, key)?;
    if bytes.len() < 2 {
        return None;
    }
    let len = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
    if bytes.len() - 2 != len {
        return None;
    }
    String::from_utf8(bytes[2..].to_vec()).ok()
}

fn config_owner_key(index: u16) -> Vec<u8> {
    let mut key = CONFIG_OWNER_PREFIX.to_vec();
    key.extend_from_slice(&index.to_le_bytes());
    key
}

fn proposal_field_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(suffix);
    key
}

fn proposal_approval_key(id: i64, index: u16) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(APPROVAL_PREFIX);
    key.extend_from_slice(&index.to_le_bytes());
    key
}

fn encode_config_json(cfg: &WalletConfig) -> NeoByteString {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn address(byte: u8) -> Vec<u8> {
        vec![byte; 20]
    }

    fn reset_state() {
        let ctx = storage_context().unwrap();
        let _ = NeoStorage::delete(&ctx, &NeoByteString::from_slice(CONFIG_THRESHOLD_KEY));
        let _ = NeoStorage::delete(&ctx, &NeoByteString::from_slice(CONFIG_OWNER_COUNT_KEY));
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(CONFIG_OWNER_PREFIX)) {
            let mut iterator = iter;
            while iterator.has_next() {
                if let Some(entry) = iterator.next() {
                    if let Some(key) = entry
                        .as_struct()
                        .and_then(|st| st.get_field("key"))
                        .and_then(NeoValue::as_byte_string)
                    {
                        let _ = NeoStorage::delete(&ctx, &key);
                    }
                }
            }
        }
        let _ = NeoStorage::delete(&ctx, &NeoByteString::from_slice(PROPOSAL_COUNTER_KEY));
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(PROPOSAL_PREFIX)) {
            let mut iterator = iter;
            while iterator.has_next() {
                if let Some(entry) = iterator.next() {
                    if let Some(key) = entry
                        .as_struct()
                        .and_then(|st| st.get_field("key"))
                        .and_then(NeoValue::as_byte_string)
                    {
                        let _ = NeoStorage::delete(&ctx, &key);
                    }
                }
            }
        }
    }

    fn configure_sample(threshold: i64) -> WalletConfig {
        reset_state();
        let owners = [address(0x11), address(0x22), address(0x33)];
        let mut buffer = Vec::new();
        for owner in &owners {
            buffer.extend_from_slice(owner);
        }
        assert_eq!(
            configure(buffer.as_ptr() as i64, owners.len() as i64, threshold),
            1
        );
        load_config(&storage_context().unwrap()).expect("config")
    }

    fn encode_args(args: &[CallArgument]) -> Vec<u8> {
        encode_arguments(args)
    }

    #[test]
    fn configure_persists_wallet() {
        let _guard = test_lock().lock().unwrap();
        let cfg = configure_sample(2);
        assert_eq!(cfg.threshold, 2);
        assert_eq!(cfg.owners.len(), 3);
    }

    #[test]
    fn propose_requires_owner() {
        let _guard = test_lock().lock().unwrap();
        let cfg = configure_sample(2);
        let signer = cfg.owners[1].clone();
        let target = address(0x00);
        let method = b"transfer".to_vec();
        let args_buf = encode_args(&[
            CallArgument::ByteString(address(0xAA)),
            CallArgument::Integer(1000),
        ]);
        let proposal_id = propose(
            signer.as_slice().as_ptr() as i64,
            signer.len() as i64,
            target.as_ptr() as i64,
            target.len() as i64,
            method.as_ptr() as i64,
            method.len() as i64,
            args_buf.as_ptr() as i64,
            args_buf.len() as i64,
        );
        assert!(proposal_id > 0);

        let random = address(0x44);
        let bad = propose(
            random.as_ptr() as i64,
            random.len() as i64,
            target.as_ptr() as i64,
            target.len() as i64,
            method.as_ptr() as i64,
            method.len() as i64,
            args_buf.as_ptr() as i64,
            args_buf.len() as i64,
        );
        assert_eq!(bad, 0);
    }

    #[test]
    fn approvals_gate_execution() {
        let _guard = test_lock().lock().unwrap();
        let cfg = configure_sample(2);
        let owner_a = cfg.owners[0].clone();
        let owner_b = cfg.owners[1].clone();
        let target = address(0x00);
        let method = b"transfer".to_vec();
        let args_buf = encode_args(&[
            CallArgument::ByteString(address(0xDD)),
            CallArgument::Integer(500),
        ]);
        let proposal_id = propose(
            owner_a.as_slice().as_ptr() as i64,
            owner_a.len() as i64,
            target.as_ptr() as i64,
            target.len() as i64,
            method.as_ptr() as i64,
            method.len() as i64,
            args_buf.as_ptr() as i64,
            args_buf.len() as i64,
        );
        assert!(proposal_id > 0);
        assert_eq!(
            approve(
                owner_b.as_slice().as_ptr() as i64,
                owner_b.len() as i64,
                proposal_id
            ),
            1
        );
        assert_eq!(
            execute(
                owner_b.as_slice().as_ptr() as i64,
                owner_b.len() as i64,
                proposal_id
            ),
            1
        );
        let ctx = storage_context().unwrap();
        assert!(load_proposal(&ctx, proposal_id).is_none());
    }
}
