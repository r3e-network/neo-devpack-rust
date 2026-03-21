use neo_devpack::prelude::*;

mod config;
mod events;
mod execution;
mod proposals;
mod storage;
mod types;
mod utils;

use config::{encode_config_json, is_owner, load_config, read_owners, store_config};
use execution::execute_proposal;
use proposals::{decode_arguments, load_proposal, next_proposal_id, store_proposal};
use storage::storage_context;
use types::{Proposal, WalletConfig};
use utils::{ensure_witness, read_address, read_string};

neo_manifest_overlay!(
    r#"{
    "name": "SampleMultisig"
}"#
);

// Events
#[neo_event]
pub struct ProposalCreatedEvt {
    pub proposal_id: NeoInteger,
    pub proposer: NeoByteString,
    pub target: NeoByteString,
    pub method: NeoString,
}

#[neo_event]
pub struct ProposalExecutedEvt {
    pub proposal_id: NeoInteger,
}

#[neo_contract]
pub struct SampleMultisigContract;

#[neo_contract]
impl SampleMultisigContract {
    pub fn new() -> Self {
        Self
    }

    /// Configure the multisig wallet with owners and threshold.
    #[neo_method]
    pub fn configure(owner_ptr: i64, owner_count: i64, threshold: i64) -> bool {
        if threshold <= 0 || owner_count <= 0 || threshold > owner_count {
            return false;
        }
        let owners = match read_owners(owner_ptr, owner_count) {
            Some(o) => o,
            None => return false,
        };
        if !ensure_witness(&owners[0]) {
            return false;
        }
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        let cfg = WalletConfig { owners, threshold };
        store_config(&ctx, &cfg).is_ok()
    }

    /// Create a new proposal.
    #[neo_method]
    pub fn propose(
        proposer_ptr: i64,
        proposer_len: i64,
        target_ptr: i64,
        target_len: i64,
        method_ptr: i64,
        method_len: i64,
        args_ptr: i64,
        args_len: i64,
    ) -> bool {
        let proposer = match read_address(proposer_ptr, proposer_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&proposer) {
            return false;
        }
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        let cfg = match load_config(&ctx) {
            Some(c) => c,
            None => return false,
        };
        if !is_owner(&cfg, &proposer) {
            return false;
        }
        let target = match read_address(target_ptr, target_len) {
            Some(a) => a,
            None => return false,
        };
        let method = match read_string(method_ptr, method_len) {
            Some(s) => s,
            None => return false,
        };
        let arguments = match decode_arguments(args_ptr, args_len) {
            Some(a) => a,
            None => return false,
        };
        let id = match next_proposal_id(&ctx) {
            Some(i) => i,
            None => return false,
        };
        let proposal = Proposal {
            proposer: proposer.clone(),
            target: target.clone(),
            method: method.clone(),
            arguments,
            approvals: vec![proposer.clone()],
            executed: false,
        };
        if store_proposal(&ctx, id, &proposal).is_err() {
            return false;
        }
        let _ = (ProposalCreatedEvt {
            proposal_id: NeoInteger::new(id),
            proposer,
            target,
            method: NeoString::from_str(&method),
        })
        .emit();
        true
    }

    /// Approve an existing proposal.
    #[neo_method]
    pub fn approve(proposal_id: i64, signer_ptr: i64, signer_len: i64) -> bool {
        let signer = match read_address(signer_ptr, signer_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&signer) {
            return false;
        }
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        let cfg = match load_config(&ctx) {
            Some(c) => c,
            None => return false,
        };
        if !is_owner(&cfg, &signer) {
            return false;
        }
        let mut proposal = match load_proposal(&ctx, proposal_id) {
            Some(p) => p,
            None => return false,
        };
        if proposal.executed {
            return false;
        }
        if proposal
            .approvals
            .iter()
            .any(|a| a.as_slice() == signer.as_slice())
        {
            return false;
        }
        proposal.approvals.push(signer);
        store_proposal(&ctx, proposal_id, &proposal).is_ok()
    }

    /// Execute a proposal once threshold approvals are reached.
    #[neo_method]
    pub fn execute(proposal_id: i64, executor_ptr: i64, executor_len: i64) -> bool {
        let executor = match read_address(executor_ptr, executor_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&executor) {
            return false;
        }
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        let cfg = match load_config(&ctx) {
            Some(c) => c,
            None => return false,
        };
        if !is_owner(&cfg, &executor) {
            return false;
        }
        let mut proposal = match load_proposal(&ctx, proposal_id) {
            Some(p) => p,
            None => return false,
        };
        if proposal.executed {
            return false;
        }
        if (proposal.approvals.len() as i64) < cfg.threshold {
            return false;
        }
        if execute_proposal(&proposal.target, &proposal.method, &proposal.arguments).is_err() {
            return false;
        }
        proposal.executed = true;
        if store_proposal(&ctx, proposal_id, &proposal).is_err() {
            return false;
        }
        let _ = (ProposalExecutedEvt {
            proposal_id: NeoInteger::new(proposal_id),
        })
        .emit();
        true
    }

    /// Return the current wallet configuration as a JSON byte string via notify.
    #[neo_method(name = "getConfig")]
    pub fn get_config() {
        let ctx = match storage_context() {
            Some(c) => c,
            None => return,
        };
        let cfg = match load_config(&ctx) {
            Some(c) => c,
            None => return,
        };
        let json = encode_config_json(&cfg);
        let label = NeoString::from_str("getConfig");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(json));
        let _ = NeoRuntime::notify(&label, &state);
    }
}

impl Default for SampleMultisigContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require NeoVM runtime stubs
    // and are exercised through integration tests against the compiled WASM.
}
