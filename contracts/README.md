# Sample Contracts

This directory provides ready-to-build Rust contracts that exercise the Wasm → NeoVM pipeline.
Each contract is a standalone crate compiling to a `cdylib` Wasm module.

| Contract | Description | Entry Points | Notes |
| --- | --- | --- | --- |
| `hello-world` | Minimal example returning a constant value. | `hello()` | Marks `hello` safe via `#[neo_safe]`. |
| `nep17-token` | State-backed NEP‑17 token with witness enforcement and transfer events. | `init(owner, amount)`, `totalSupply()`, `balanceOf(account)`, `transfer(from, to, amount)`, `onNEP17Payment(from, amount, data)` | Declares `totalSupply`/`balanceOf` safe via `#[neo_safe]`. |
| `constant-product` | Constant-product AMM with storage-backed reserves, swap fees, and swap events. | `init(initial_x, initial_y)`, `getReserves()`, `quote(amount_in)`, `swap(trader, amount_in)` | Declares query methods safe via `#[neo_safe]`. |
| `nep11-nft` | Minimal NEP‑11 NFT that supports minting, owner lookup, and transfers. | `mint(owner, token_id)`, `totalSupply()`, `balanceOf(owner)`, `ownerOf(token_id)`, `transfer(from, to, token_id)` | View methods use `#[neo_safe]`; emits NEP‑11 transfer events. |
| `multisig-wallet` | Configurable multisig wallet with JSON-based proposal workflow and contract calls. | `configure(payload_ptr, payload_len)`, `propose(payload_ptr, payload_len)`, `approve(payload_ptr, payload_len)`, `execute(payload_ptr, payload_len)`, `getConfig()` | View method `getConfig` is safe; approvals enforce threshold/quorum. |
| `escrow` | Arbiter-controlled NEP‑17 escrow with manifest-backed events. | `configure(...)`, `release(...)`, `refund(...)`, `onNEP17Payment(from, amount, data)`, `getState()` | `getState` marked safe; funds released via `NeoContractRuntime::call`. |
| `crowdfunding` | Campaign-style crowdfunding with NEP‑17 contributions and refunds. | `configure(...)`, `finalize(current_time)`, `claimRefund(...)`, `onNEP17Payment(from, amount, data)`, `getCampaign()`, `contributionOf(address)` | Safe views for campaign state/contributions; contributions tracked per address. |
| `governance-dao` | Stake-weighted governance with proposals, voting, execution, and staking. | `configure(...)`, `propose(...)`, `vote(...)`, `execute(proposal_id)`, `unstake(...)`, `onNEP17Payment(from, amount, data)`, `getConfig()`, `getProposal(id)`, `stakeOf(address)` | Safe accessors for config/proposals/stakes; proposal execution triggers contract calls. |
| `oracle-consumer` | Oracle request/response manager for off-chain data retrieval. | `configure(...)`, `request(url, filter, user_data)`, `onOracleResponse(request_id, code, data)`, `getResponse(id)`, `lastRequestId()` | Responses stored as JSON; safe getters for config and responses. |
| `nft-marketplace` | Custodial NEP‑11 marketplace settling sales with NEP‑17 payments. | `createListing(...)`, `cancelListing(...)`, `onNEP11Payment(from, token_id, amount, data)`, `onNEP17Payment(from, amount, data)`, `getListing(id)` | Listings persisted in storage; safe listing lookup for tooling. |

## Building

Run the Makefile target to compile all examples and translate them to NEF/manifest files:

```bash
make examples
```

Artifacts appear in the repository-level `build/` directory. To regenerate a specific contract,
invoke its target (e.g., `make nep17-token`).

## Deployment

Refer to `scripts/neoexpress_deploy.sh` and [`docs/neoexpress-integration.md`](../docs/neoexpress-integration.md)
for instructions on deploying the generated artifacts to a Neo Express instance.
