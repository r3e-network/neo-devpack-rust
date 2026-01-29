use neo_devpack::prelude::*;

mod config;
mod events;
mod execution;
mod proposals;
mod storage;
mod types;
mod utils;

use config::{encode_config_json, is_owner, load_config, read_owners, store_config};
use events::{ProposalCreated, ProposalExecuted};
use execution::execute_proposal;
use proposals::{decode_arguments, load_proposal, next_proposal_id, remove_proposal_entries, store_proposal};
use storage::storage_context;
use types::{Proposal, WalletConfig};
#[cfg(test)]
use types::CallArgument;
use utils::{addresses_equal, ensure_witness, read_address, read_string};

neo_manifest_overlay!(
    r#"{
    "name": "SampleMultisig",
    "features": { "storage": true }
}"#
);

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
    if execute_proposal(&proposal.target, &proposal.method, &proposal.arguments).is_err() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use types::encode_arguments;

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn address(byte: u8) -> Vec<u8> {
        vec![byte; 20]
    }

    fn reset_state() {
        use storage::*;
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
