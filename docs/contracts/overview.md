# Contract Examples

This repository ships with a curated set of Rust smart-contract examples that demonstrate common dApp patterns on Neo N3. Every contract includes storage-backed state, safe method annotations via `#[neo_safe]`, events, and Makefile targets for building/translating to NEF + manifest.

| Contract | Path | Highlights |
| --- | --- | --- |
| Hello World | `contracts/hello-world` | Minimal constant-return export. |
| NEP-17 Token | `contracts/nep17-token` | Storage-backed balances, witness enforcement, transfer events. |
| Constant-product AMM | `contracts/constant-product` | Swap/quote functions, reserves in storage, fee logic. |
| NEP-11 NFT | `contracts/nep11-nft` | Minting, ownership tracking, balance queries, transfer events. |
| Multisig Wallet | `contracts/multisig-wallet` | Configurable owners/threshold, JSON proposal approvals, on-chain execution. |
| Escrow | `contracts/escrow` | Arbiter-driven NEP‑17 escrow with release/refund controls. |
| Crowdfunding | `contracts/crowdfunding` | Campaign funding with NEP‑17 deposits, success/failure handling, refunds. |
| Governance DAO | `contracts/governance-dao` | Stake-weighted proposals, voting, execution, and staking/unstaking. |
| Oracle Consumer | `contracts/oracle-consumer` | Issues oracle requests and records responses for downstream consumption. |
| NFT Marketplace | `contracts/nft-marketplace` | Custodial NEP‑11 listings settled with NEP‑17 payments. |
| C Hello World | `contracts/c-hello` | Minimal C contract built via clang helper, illustrates Wasm translation without Rust. |
Each contract crate compiles to Wasm (`cargo build --target wasm32-unknown-unknown`) and can be translated via `make <name>`. The examples cover common dApp archetypes including wallets, DeFi (AMM, crowdfunding, staking governance), NFT trading, oracle integration, and programmatic escrow, providing end-to-end templates for production-grade Neo deployments.

For deployment guidance, see [`docs/neoexpress-integration.md`](../neoexpress-integration.md) and the helper script `scripts/neoexpress_deploy.sh`.
