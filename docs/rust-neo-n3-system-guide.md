# Rust Neo N3 System Guide

This guide documents the end-to-end system for authoring, compiling,
optimizing, translating, deploying, and validating Rust smart contracts for Neo
N3 with this repository.

Use it as the detailed companion to:

- [`rust-smart-contract-quickstart.md`](rust-smart-contract-quickstart.md) for the minimal first build.
- [`wasm-pipeline.md`](wasm-pipeline.md) for the translator design.
- [`neoexpress-integration.md`](neoexpress-integration.md) for Neo Express deployment.
- [`contracts/README.md`](../contracts/README.md) for the sample contract catalogue.

## 1. System Map

The production flow is Wasm-first:

```text
Rust contract crate
  -> cargo build --target wasm32-unknown-unknown --release
  -> optimized Wasm module
  -> wasm-neovm translator
  -> NeoVM script
  -> NEF + manifest
  -> Neo Express / Neo node deployment
```

The repository is split into these system areas:

| Area | Path | Responsibility |
| --- | --- | --- |
| Contract examples | `contracts/*` | Rust and cross-chain sample contracts compiled to Wasm. |
| DevPack facade | `rust-devpack/` | Macros, types, runtime, storage, and syscall wrappers used by contracts. |
| Runtime crates | `rust-devpack/neo-runtime`, `rust-devpack/neo-syscalls`, `rust-devpack/neo-types` | Low-level contract-facing APIs and host-mode simulation. |
| Translator | `wasm-neovm/` | Parses Wasm, lowers supported instructions/imports to NeoVM, emits NEF and manifest. |
| Cross-chain layers | `solana-compat/`, `move-neovm/` | Solana-compatible Rust facade and experimental Move lowering. |
| Build scripts | `scripts/build_contract.sh`, `Makefile` | Build Wasm, optimize it, translate to NEF/manifest, and run smoke checks. |
| Runtime validation | `scripts/neoxp_smoke.sh`, `integration-tests/` | Neo Express deploy/invoke validation. |
| Specs and docs | `docs/`, `spec/` | User guides, design notes, conformance matrix, and translator spec. |

## 2. Contract Authoring Model

Rust contracts are normal `cdylib` crates targeting `wasm32-unknown-unknown`.
Most examples depend on `neo-devpack` with default features disabled:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
neo-devpack = { path = "../../rust-devpack", default-features = false }

[features]
default = ["neo-devpack/std"]
```

The default feature is useful for host-mode tests. Production Wasm builds use
`--no-default-features`, which avoids pulling in JSON, serde, crypto helper
implementations, and host-only dependencies.

### Contract Macro Pattern

The preferred shape is:

```rust
use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "ExampleContract"
}"#
);

#[neo_event]
pub struct ValueStored {
    pub key: i64,
    pub value: i64,
}

#[neo_contract]
pub struct ExampleContract;

#[neo_contract]
impl ExampleContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn put(key: i64, value: i64) -> bool {
        RawStorage::put_i64_key(key, value);
        let _ = (ValueStored { key, value }).emit();
        true
    }

    #[neo_method(safe)]
    pub fn get(key: i64) -> i64 {
        RawStorage::get_i64_key_or_zero(key)
    }
}
```

The macro layer emits ABI exports and custom manifest metadata. The translator
then validates that generated manifest entries match real Wasm exports.

### ABI Guidance

For compact NEF output, prefer:

- `i64`, `i32`, and `bool` ABI parameters/results for sample contracts.
- `#[neo_method(safe)]` for read-only view methods.
- Primitive event fields such as `i64` and `bool`.
- Integer IDs for sample accounts/script hashes when the method is intended
  for deploy/invoke smoke tests.

Avoid on the Wasm contract path unless necessary:

- `Vec`, `String`, dynamic allocation, and formatting.
- `NeoInteger`/BigInt conversion for simple integer values.
- `serde`, `serde_json`, or JSON storage payloads.
- `panic!`, `unwrap`, indexing without bounds checks, and arithmetic that can
  overflow on public inputs.

Panics fault the VM and revert the transaction. Treat every public entry point
as an untrusted input boundary.

## 3. Storage Model

There are two storage APIs:

| API | Best for | Notes |
| --- | --- | --- |
| `NeoStorage` | Host tests and byte-string-typed APIs | Uses `NeoByteString` values and the host syscall simulation. |
| `RawStorage` | Production Wasm contracts | Heap-free facade using slices or direct `i64` keys. |

### Direct `i64` Storage

Hot sample contracts use direct `i64` keys:

```rust
const KEY_STRIDE: i64 = 16;
const FIELD_OWNER: i64 = 1;

fn owner_key(id: i64) -> i64 {
    id * KEY_STRIDE + FIELD_OWNER
}

fn valid_id(id: i64) -> bool {
    id > 0 && id <= i64::MAX / KEY_STRIDE
}
```

This avoids materializing byte-string keys in Wasm linear memory, which reduces
NEF size and avoids allocator-heavy code paths.

Always guard multiplication-based keys with an upper bound such as
`id <= i64::MAX / KEY_STRIDE`.

### Byte Keys Without Heap Allocation

For composite keys, use `RawKeyBuilder`:

```rust
const PREFIX: &[u8] = b"vote:";
const KEY_LEN: usize = 5 + 8 + 1 + 8;

fn vote_key(proposal_id: i64, voter_id: i64) -> RawKeyBuilder<KEY_LEN> {
    let mut key = RawKeyBuilder::new();
    key.push_bytes(PREFIX);
    key.push_i64_le(proposal_id);
    key.push_byte(b':');
    key.push_i64_le(voter_id);
    key
}
```

The builder is fixed-capacity and rejects overflow without mutating the
existing key bytes.

### Missing Values and Empty Bytes

Neo N3 commonly surfaces missing storage values as empty byte strings. For that
reason:

- `RawStorage::get_into` can return `Found(0)` for both empty and absent values.
- `RawStorageGet::Missing` is only for runtimes that explicitly report null.
- Do not use a zero-length byte value as your only existence marker.

For direct `i64` keys, `RawStorage::has_i64_key` treats any non-empty storage
value as present. A stored integer zero is encoded as one byte (`0x00`) so it is
distinguishable from a missing direct key.

When business logic needs existence, prefer an explicit status/existence field:

```rust
const FIELD_EXISTS: i64 = 3;

RawStorage::put_i64_key(record_key(id, FIELD_EXISTS), 1);

if RawStorage::get_i64_key_or_zero(record_key(id, FIELD_EXISTS)) == 0 {
    return false;
}
```

## 4. Runtime Helpers

Use the direct helpers when a contract only needs primitive values:

| Helper | Why it exists |
| --- | --- |
| `NeoRuntime::check_witness_i64(account)` | Avoids linear-memory byte construction for sample account IDs. |
| `NeoRuntime::get_calling_script_hash_i64()` | Lets contracts compare compact script-hash IDs without heap allocation. |
| `NeoRuntime::get_time_i64()` | Reads `System.Runtime.GetTime` without pulling BigInt conversion into small Wasm contracts. |
| `RawStorage::put_i64_key/get_i64_key_or_zero/delete_i64_key` | Direct storage syscall path for integer keys/values. |

Use the richer `NeoRuntime`/`NeoStorage` APIs when the method truly needs
Neo byte strings, arrays, maps, or host-side JSON/testing support.

## 5. Build Pipeline

The most direct build path is:

```bash
scripts/build_contract.sh contracts/hello-world HelloWorld
```

The helper does four things:

1. Builds the contract with `cargo build --target wasm32-unknown-unknown --release`.
2. Defaults to `--no-default-features` via `NEO_CARGO_NO_DEFAULT_FEATURES=1`.
3. Applies size and compatibility Rust flags:

   ```text
   -C opt-level=z
   -C strip=symbols
   -C panic=abort
   -C target-feature=-simd128,-reference-types,-multivalue,-tail-call
   ```

4. Optionally runs `wasm-opt -Oz --enable-bulk-memory --strip-debug --strip-producers`.
5. Runs `wasm-neovm` to emit `.nef` and `.manifest.json`.

Useful environment switches:

| Variable | Default | Effect |
| --- | --- | --- |
| `NEO_CARGO_NO_DEFAULT_FEATURES` | `1` | Set to `0` to build with contract default features. |
| `NEO_CARGO_FEATURES` | empty | Extra Cargo features for the contract build. |
| `NEO_WASM_RUSTFLAGS` | size/compat flags | Override all contract Rust flags. |
| `NEO_WASM_OPT` | `1` | Set to `0` to skip `wasm-opt`. |
| `NEO_WASM_OPT_FLAGS` | `-Oz --enable-bulk-memory --strip-debug --strip-producers` | Override wasm-opt flags. |
| `SOURCE_CHAIN` | empty | Adds `--source-chain` for cross-chain samples when not supplied manually. |

The Makefile wraps the same idea for all bundled examples:

```bash
make examples
make crowdfunding
make governance-dao
make smoke-neoxp
```

Generated artifacts live under `build/` or contract-local `target/` directories.
They are ignored by Git.

## 6. Translator Import Surface

The translator recognizes several import modules:

| Module | Purpose |
| --- | --- |
| `neo` | DevPack-friendly runtime/storage imports such as `runtime_get_time`, `raw_storage_put_i64`, and `runtime_check_witness_i64`. |
| `syscall` | Canonical Neo syscall descriptors such as `System.Runtime.GetTime`. |
| `opcode` | Direct NeoVM opcode emission for low-level tests and advanced cases. |
| `env` | Bounded memory shims such as `memcpy`, `memmove`, and `memset`. |

Unsupported imports fail translation with explicit diagnostics. This is
intentional: silent fallback would be worse than a hard build failure.

## 7. Size Optimization Playbook

The largest NEF reductions come from avoiding code that the translator must
lower into runtime helpers.

### Prefer These

- Build with `--no-default-features`.
- Use `RawStorage` and direct `i64` keys for hot storage paths.
- Use `RawKeyBuilder` for composite byte keys.
- Use primitive event fields.
- Use `NeoRuntime::get_time_i64` and script-hash `i64` helpers when possible.
- Use explicit arithmetic guards before multiplication/addition.
- Keep public methods simple and split helper logic only when it reduces
  repeated code.

### Avoid These in Wasm Contracts

- Formatting, `Debug`, `Display`, or panic-heavy code paths.
- JSON serialization inside the on-chain contract.
- Heap allocation for keys or tiny state values.
- BigInt conversions for native timestamp/account/sample values.
- Generic abstractions that monomorphize many copies of the same logic.
- `checked_mul`/`checked_add` blindly in very small contracts when a manual
  bound check is smaller and equally clear.

Example size-friendly multiplication:

```rust
if amount > i64::MAX / FEE_BPS {
    return 0;
}
let fee = (amount * FEE_BPS) / BPS_DENOMINATOR;
```

### Diagnose a Large NEF

Run:

```bash
cargo tree --manifest-path contracts/<name>/Cargo.toml \
  --target wasm32-unknown-unknown \
  --no-default-features \
  -e normal
```

Look for unexpected runtime dependencies such as JSON, serde, crypto hashing,
formatting, host-only synchronization crates, or allocator-heavy code paths.

Then compare raw and optimized Wasm:

```bash
NEO_WASM_OPT=0 scripts/build_contract.sh contracts/<name> ContractName
scripts/build_contract.sh contracts/<name> ContractName
```

If the optimized build is much larger than expected, inspect recent changes for:

- New `Vec`/`String` usage.
- New `NeoInteger` conversions.
- New panic paths.
- `checked_*` helpers in tiny arithmetic-only contracts.
- Accidental default feature usage.

## 8. Current Smoke-Test NEF Sizes

The following sizes were produced by the Neo Express smoke path on
2026-06-18 after rebuilding all sample contracts:

| Contract | NEF bytes |
| --- | ---: |
| HelloWorld | 81 |
| NEP17 | 318 |
| NEP11 | 267 |
| AMM | 488 |
| UniswapV2 | 736 |
| StakingRewards | 2219 |
| TimelockVault | 2099 |
| FlashLoanPool | 581 |
| MultisigWallet | 1239 |
| Escrow | 2278 |
| Crowdfunding | 5211 |
| GovernanceDAO | 6306 |
| OracleConsumer | 2538 |
| NFTMarketplace | 2213 |
| solana_hello | 997 |
| MoveCoin | 201 |
| StorageSmoke | 1783 |

These are smoke-test reference points, not hard ABI guarantees. Correctness
guards may intentionally increase size.

## 9. Validation Matrix

Use layered validation. Each layer catches a different class of problem.

| Layer | Command | Catches |
| --- | --- | --- |
| Formatting | `cargo fmt --all` | Style drift. |
| Core clippy | `cargo clippy --manifest-path rust-devpack/Cargo.toml --all-targets -- -D warnings` | DevPack regressions. |
| Translator clippy | `cargo clippy --manifest-path wasm-neovm/Cargo.toml --all-targets -- -D warnings` | Translator regressions. |
| Wasm contract clippy | `cargo clippy --manifest-path contracts/<name>/Cargo.toml --target wasm32-unknown-unknown --lib --release --no-default-features -- -D warnings` | Production Wasm no-default issues. |
| Host contract clippy | `cargo clippy --manifest-path contracts/<name>/Cargo.toml --all-targets -- -D warnings` | Host tests and simulation warnings. |
| Unit tests | `make test` | Workspace and contract unit coverage. |
| Runtime smoke | `DOTNET_ROOT=... NEOXP_BIN=... scripts/neoxp_smoke.sh` | NEF build, deploy, invoke, and persistent storage behavior. |
| Diff hygiene | `git diff --check` | Whitespace and patch hygiene. |
| Git artifact check | `git ls-files 'build/**' 'target/**' '*.nef' '*.wasm'` | Accidental generated-file tracking. |

The strongest local validation before publishing a change is:

```bash
cargo fmt --all
make test
for manifest in contracts/*/Cargo.toml; do
  cargo clippy --manifest-path "$manifest" \
    --target wasm32-unknown-unknown \
    --lib --release --no-default-features -- -D warnings
done
DOTNET_ROOT=/path/to/dotnet NEOXP_BIN=/path/to/neoxp scripts/neoxp_smoke.sh
git diff --check
```

## 10. Neo Express Smoke Scope

`scripts/neoxp_smoke.sh`:

1. Rebuilds all sample contracts.
2. Prints NEF sizes.
3. Creates a temporary Neo Express chain.
4. Deploys every generated NEF.
5. Invokes runtime-safe methods.
6. Commits and reads back a real storage value through `StorageSmoke`.

The smoke suite validates critical runtime facts:

- Direct call argument order is correct (`directFirst`, `directSecond`).
- Byte-key storage round-trips through persistent Neo Express chain state.
- Direct `i64` storage can distinguish a missing key from a stored zero.
- Cross-chain examples deploy and selected methods invoke.

Some witness-heavy/stateful sample paths are deploy-validated in Neo Express and
covered by Rust host tests until richer Neo Express fixtures provision account
witnesses and inter-contract token flows.

## 11. Adding a New Rust Contract

Checklist:

1. Create `contracts/<name>/Cargo.toml` with `crate-type = ["cdylib"]`.
2. Depend on `neo-devpack` with `default-features = false`.
3. Add `#[neo_contract]`, `#[neo_method]`, safe view methods, and a manifest
   overlay name.
4. Prefer primitive ABI parameters and return values.
5. Use `RawStorage` for storage-heavy code.
6. Guard public arithmetic and key multiplication.
7. Add host tests with `#[test]`.
8. Add a Makefile target and smoke script entry if it belongs in the bundled suite.
9. Build with:

   ```bash
   scripts/build_contract.sh contracts/<name> ContractName
   ```

10. Validate with both host and wasm clippy.
11. Deploy/invoke through Neo Express if the method is runtime-safe.

## 12. Troubleshooting

### NEF Suddenly Gets Large

Check for default features:

```bash
cargo tree --manifest-path contracts/<name>/Cargo.toml \
  --target wasm32-unknown-unknown \
  --no-default-features \
  -e features
```

Then inspect recent code for heap allocation, JSON, BigInt, formatting, or
panic paths.

### Host `--no-default-features` Fails but Wasm Works

Host simulation may need target-specific support crates. The production check
is the wasm target no-default build:

```bash
cargo clippy --manifest-path contracts/<name>/Cargo.toml \
  --target wasm32-unknown-unknown \
  --lib --release --no-default-features -- -D warnings
```

If a host-only dependency is needed, put it under a target-specific dependency
section instead of enabling it for wasm.

### Manifest Does Not Match Exports

The translator rejects overlays that refer to missing methods or mutate
translated signatures/offsets. Check:

- `neo_manifest_overlay!` method names.
- `#[neo_method(name = "...")]` aliases.
- External `manifest.overlay.json` files.
- Whether the method was optimized away or not exported by the macro.

### Missing Storage Looks Like Zero

For integer storage, absence usually reads as `0`. Use explicit status fields
when zero is a valid business value.

### Contract FAULTs on Neo Express

Look for:

- Panic paths (`unwrap`, indexing, overflow).
- Unsupported Wasm features.
- Wrong argument order or type encoding in the invoke command.
- Missing witness/account flags for state-changing methods.
- Contract logic that assumes storage existence from a zero value.

### Generated Files Appear in Git

Expected generated paths are ignored:

```text
build/
target/
**/target/
*.nef
*.manifest.json
```

The only tracked manifest JSON exception is the test baseline:

```text
contracts/hello-world/expected.manifest.json
```

Run:

```bash
git ls-files 'build/**' 'target/**' 'contracts/**/target/**' '*.nef' '*.wasm'
```

This should not list generated build products.

## 13. Current Boundaries

The Rust -> Wasm -> NeoVM path supports the bundled examples and the integer
contract surface documented in the translator compatibility matrix. Important
boundaries remain:

- Floating point and SIMD are unsupported.
- Multiple memories are unsupported.
- Reference types beyond the supported `funcref` table model are unsupported.
- Some sample contracts are smoke-oriented templates, not full production
  financial protocols.
- Rich inter-contract fixtures are still better covered by targeted host tests
  unless a Neo Express scenario provisions all required witnesses and contract
  state.

For low-level translator details, keep [`wasm-pipeline.md`](wasm-pipeline.md)
and [`spec/wasm-neovm-spec.tex`](../spec/wasm-neovm-spec.tex) as the source of
truth.
