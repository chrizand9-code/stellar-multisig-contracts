# Stellar Multisig Contracts

On-chain Soroban smart contracts for M-of-N multisig accounts on Stellar. Built on the Soroban smart contract platform, these contracts integrate directly with Stellar's native account model, token standard (SEP-41), and the Soroban host environment.

## Overview

| Contract | Description |
|---|---|
| `multisig-account` | Custom account implementing `__check_auth` for M-of-N ed25519 signature verification |
| `account-factory` | Deploys and tracks multisig accounts on-chain using Soroban's deployer API |

## Stellar Integration

### Custom Account Protocol (`__check_auth`)

Soroban exposes a custom account interface that replaces Stellar's classic account signing. When any contract calls `require_auth()` on a multisig account address, the Soroban host invokes `__check_auth` on that contract instead of checking a Stellar keypair. This means the multisig account can authorize:

- **Token transfers** — any SAC (Stellar Asset Contract) or SEP-41 token transfer
- **Contract invocations** — calling any other Soroban contract
- **Signer management** — adding/removing signers and changing the threshold
- **Cross-contract calls** — any action that calls `require_auth()` on the multisig address

```
User submits transaction
        │
        ▼
Soroban Host receives require_auth(multisig_address)
        │
        ▼
Host calls multisig.__check_auth(payload, signatures, contexts)
        │
        ├── Verify signatures are sorted (no duplicates)
        ├── Verify each signer is registered
        ├── Verify each ed25519 signature against payload
        └── Count valid sigs >= threshold → ✅ authorized
```

### Ed25519 Signature Verification

Signatures use Stellar's native ed25519 keys (the same keys used for classic Stellar accounts). The `__check_auth` function calls `env.crypto().ed25519_verify()` — a host function provided by the Soroban runtime — to verify each signature against the 32-byte payload hash.

```rust
env.crypto().ed25519_verify(
    &sig.public_key,   // BytesN<32> — ed25519 public key
    &signature_payload.into(), // 32-byte hash of the auth payload
    &sig.signature,    // BytesN<64> — ed25519 signature
);
```

### Stellar Asset Contract (SAC) Compatibility

Because the multisig account implements the custom account interface, it is fully compatible with Stellar Asset Contracts. Any SAC transfer that calls `require_auth()` on the multisig address will trigger the M-of-N verification flow automatically — no special integration needed.

### Factory Deployment via Soroban Deployer

The `account-factory` contract uses Soroban's built-in deployer API to deploy new multisig instances deterministically:

```rust
let deployed_address = env
    .deployer()
    .with_current_contract(salt)   // deterministic address from factory + salt
    .deploy(account_wasm_hash);    // deploys the multisig WASM on-chain
```

The deployed address is deterministic — given the same factory address and salt, the multisig address is always predictable before deployment.

### On-Chain Events

Both contracts emit Soroban events that can be indexed by Horizon or any Stellar event streaming service:

| Contract | Event | Data |
|---|---|---|
| multisig-account | `Initialized` | `(threshold, signer_count)` |
| multisig-account | `SignerAdded` | `public_key` |
| multisig-account | `SignerRemoved` | `public_key` |
| multisig-account | `ThresholdChanged` | `new_threshold` |
| account-factory | `Initialized` | `(admin, wasm_hash)` |
| account-factory | `AccountCreated` | `(creator, address, threshold, signer_count)` |

## Features

- ✅ M-of-N signature threshold enforcement via `__check_auth`
- ✅ Native ed25519 verification using Soroban host crypto functions
- ✅ Compatible with all SAC tokens and SEP-41 tokens
- ✅ Deterministic account address derivation via factory + salt
- ✅ Self-custodial signer management (add/remove requires multisig approval)
- ✅ Dynamic threshold adjustment (requires multisig approval)
- ✅ Replay and duplicate signature protection
- ✅ On-chain events for Horizon indexing
- ✅ No panics — all errors via typed error enums

## Repository Structure

```
stellar-multisig-contracts/
├── contracts/
│   ├── multisig-account/     # Custom account with __check_auth
│   │   └── src/
│   │       ├── lib.rs        # Contract logic
│   │       └── test.rs       # Unit tests
│   └── account-factory/      # Factory for deploying accounts
│       └── src/
│           ├── lib.rs        # Contract logic
│           └── test.rs       # Unit tests
├── docs/
│   ├── multisig-account.md   # Deep dive into the account contract
│   └── account-factory.md    # Factory contract documentation
├── scripts/
│   └── deploy.sh             # Deployment automation
├── DEPLOYMENTS.md            # Testnet/mainnet addresses
├── CONTRIBUTING.md           # Development guide
└── README.md
```

## Quick Start

### Prerequisites

- Rust 1.74+
- Soroban CLI: `cargo install soroban-cli`
- Stellar account funded on testnet: `soroban keys generate --network testnet`

### Build

```bash
cargo build --release --target wasm32-unknown-unknown
```

### Test

```bash
cargo test
```

### Deploy to Testnet

```bash
# 1. Upload multisig-account WASM
soroban contract upload \
  --network testnet \
  --source <your-key> \
  --wasm target/wasm32-unknown-unknown/release/multisig_account.wasm

# 2. Deploy and initialize the factory
soroban contract deploy \
  --network testnet \
  --source <your-key> \
  --wasm target/wasm32-unknown-unknown/release/account_factory.wasm

soroban contract invoke \
  --network testnet \
  --source <your-key> \
  --id <factory-address> \
  -- initialize \
  --admin <your-address> \
  --account_wasm_hash <wasm-hash-from-step-1>

# 3. Create a multisig account (2-of-3)
soroban contract invoke \
  --network testnet \
  --source <your-key> \
  --id <factory-address> \
  -- create_account \
  --creator <your-address> \
  --salt 0000000000000000000000000000000000000000000000000000000000000001 \
  --signers '["<pubkey1>","<pubkey2>","<pubkey3>"]' \
  --threshold 2
```

See [DEPLOYMENTS.md](./DEPLOYMENTS.md) for deployed contract addresses, or run the full automation:

```bash
./scripts/deploy.sh
```

## Contract API

### multisig-account

| Function | Auth Required | Description |
|---|---|---|
| `init(signers, threshold)` | None (one-time) | Initialize with signer list and threshold |
| `add_signer(signer)` | Multisig | Add a new signer public key |
| `remove_signer(signer)` | Multisig | Remove a signer (threshold must remain valid) |
| `set_threshold(threshold)` | Multisig | Update the M threshold |
| `get_signers()` | None | Return all registered signer public keys |
| `get_threshold()` | None | Return current threshold |
| `__check_auth(payload, sigs, contexts)` | Host-invoked | Verify M-of-N ed25519 signatures |

### account-factory

| Function | Auth Required | Description |
|---|---|---|
| `initialize(admin, wasm_hash)` | None (one-time) | Set admin and multisig WASM hash |
| `create_account(creator, salt, signers, threshold)` | Creator | Deploy and initialize a new multisig |
| `get_accounts_by_creator(creator)` | None | List all accounts deployed by an address |
| `account_count()` | None | Total accounts deployed via this factory |
| `get_admin()` | None | Return factory admin address |
| `get_account_wasm_hash()` | None | Return the registered multisig WASM hash |

## Error Codes

### MultisigError

| Code | Name | Description |
|---|---|---|
| 1 | `NotInitialized` | Contract not yet initialized |
| 2 | `AlreadyInitialized` | `init` called more than once |
| 3 | `InvalidThreshold` | Threshold is 0 or exceeds signer count |
| 4 | `UnknownSigner` | Signature from unregistered public key |
| 5 | `DuplicateSigner` | Duplicate public key in signer list |
| 6 | `BadSignature` | ed25519 verification failed |
| 7 | `ThresholdNotMet` | Valid signature count below threshold |
| 8 | `SignaturesNotSorted` | Signatures not in ascending key order |

### FactoryError

| Code | Name | Description |
|---|---|---|
| 1 | `AlreadyInitialized` | Factory already initialized |
| 2 | `NotInitialized` | Factory not yet initialized |
| 3 | `InvalidThreshold` | Threshold is 0 or exceeds signer count |

## Security

- All admin functions require the multisig contract itself as the auth source — no single key can modify signers or threshold
- Signatures must be submitted in strictly ascending public key order, preventing replay and duplicate counting
- Threshold is validated on every state change — it can never be left in an invalid state
- No `unwrap()` or `panic!` — all error paths return typed errors

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines, testing procedures, and architecture details.

## License

MIT
