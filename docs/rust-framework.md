# Neo N3 Rust Smart Contract Framework Design

## Design Goals
- Ergonomic developer experience inspired by Anchor (Solana), ink! (Polkadot), and Move frameworks.
- Type-safe bindings for Neo N3 syscalls and native contracts.
- Deterministic execution, explicit storage access, manifest auto-generation.
- Integrated testing, simulation, and deployment tooling.

## Core Crates
1. `neo-sdk` (no_std)
   - Stack conversion traits (`FromStack`, `ToStack`).
   - Syscall wrappers (`runtime`, `storage`, `contract`, `ledger`, `oracle`, `policy`, `crypto`, `utility` modules).
   - Panic handler -> `ABORTMSG`.
   - Logging/events macros (`emit!`).
2. `neo-macros`
   - `#[contract]` attribute: marks entry struct, generates dispatcher.
   - `#[event]` and `#[syscall]` derive macros.
   - `#[storage]` attribute for typed storage access.
3. `neo-abi`
   - Manifest generator from Rust metadata.
   - Permission builder.
4. `neo-cargo` (binary)
   - `cargo neo-build`, `cargo neo-test`, `cargo neo-deploy` subcommands.

## Contract Authoring Pattern
```rust
use neo_sdk::{contract, storage, event};

#[contract]
pub struct Counter;

impl Counter {
    #[init]
    pub fn deploy(owner: Address) {
        storage::put(b"owner", &owner);
        storage::put(b"count", &0u32);
    }

    #[method(payable)]
    pub fn increment(ctx: Context, delta: u32) -> u32 {
        ctx.assert_sender_is(storage::get::<Address>(b"owner"));
        let mut count: u32 = storage::get(b"count");
        count += delta;
        storage::put(b"count", &count);
        Incremented::emit(delta, count);
        count
    }
}

#[event]
pub struct Incremented { delta: u32, new_value: u32 }
```
- Attribute macros expand to dispatcher that decodes arguments from stack, handles permissions, writes manifest entries.
- `Context` provides access to transaction info via syscalls.

## Storage API
- Modeled after Move resource semantics and Anchor accounts.
- `storage::Map<T>` typed wrapper; ensures serialization using Borsh-like format.
- `#[storage(collection = "map", key_type = "Address", value_type = "Balance"]` to auto-generate typed stores.
- Provide `State<T>` for singletons, `Ledger<T>` for NEP-17 tokens.

## Syscall Coverage
- Auto-generated modules from registry with typed signatures:
  ```rust
  pub fn runtime_get_time() -> Timestamp;
  pub fn contract_call(hash: ContractHash, method: &str, args: &[Value]) -> Result<Value>;
  ```
- Use traits to abstract environment (for testing, provide mock engine implementing same trait).

## Manifest & Permission DSL
- Macro collects metadata: events, methods, permissions, trust requirements.
- Provide builder API akin to Ink! `#[ink::chain_extension]` and Anchor IDL generation.

## Testing Framework
- `neo-test` crate providing:
  - In-memory NeoVM interpreter (Rust port or FFI to C++ implementation).
  - `TestRuntime` trait for customizing block/timestamp, storage snapshot.
  - Property testing harness using `proptest`.
  - Fixtures for common contracts (NEP-17, NFT) for integration tests.

## Deployment Workflow
1. `cargo neo-build` -> `.nef`, `.manifest.json`, metadata.
2. `cargo neo-test` -> run scenario tests using interpreter.
3. `cargo neo-deploy --network testnet --wallet path.json` -> uses RPC to deploy.
4. Optional `neo fmt-manifest` to inspect ABI.

## Inspiration Mapping
- **Anchor (Solana)**: macros for entrypoints, account validation -> `Context` validation and storage wrappers.
- **ink! (Polkadot)**: `#[ink(storage)]`, `#[ink(message)]` -> `#[contract]`, `#[method]` macros controlling manifest generation.
- **Move**: resource-oriented storage -> typed storage wrappers ensuring borrow rules; add lints to prevent unbounded storage growth.

## Extensibility Roadmap
- Provide `neo-migrate` for schema upgrades (similar to Anchor `#[state]` updates).
- Integrate deterministic JSON serialization for oracle interactions.
- Add `neo-bench` to profile gas usage using instrumentation pass.

