# Contributing to Stellar Multisig Contracts

Thank you for your interest in contributing to Stellar Multisig Contracts! This guide will help you understand the architecture, set up your development environment, and make meaningful contributions.

## Table of Contents

1. [Development Setup](#development-setup)
2. [Architecture Overview](#architecture-overview)
3. [Understanding __check_auth](#understanding-__check_auth)
4. [Testing Guidelines](#testing-guidelines)
5. [Signature Ordering and Deduplication](#signature-ordering-and-deduplication)
6. [Extending Signature Types](#extending-signature-types)
7. [Deployment](#deployment)

## Development Setup

### Prerequisites

- **Rust**: 1.74 or higher
- **Soroban CLI**: Latest version
- **Stellar Account**: With testnet XLM

### Install Soroban CLI

```bash
cargo install --locked soroban-cli --features opt
```

### Install Target

```bash
rustup target add wasm32-unknown-unknown
```

### Clone and Build

```bash
git clone https://github.com/<your-org>/stellar-multisig-contracts.git
cd stellar-multisig-contracts
cargo build --release --target wasm32-unknown-unknown
```

### Run Tests

```bash
cargo test
```

### Run Specific Test

```bash
cargo test test_check_auth_duplicate_signer_rejected
```

## Architecture Overview

This project consists of two main contracts:

### 1. Multisig Account Contract (`contracts/multisig-account`)

A Soroban custom account that implements the `__check_auth` function to enforce M-of-N signature requirements. This contract acts as an address that can authorize actions on behalf of its signers.

**Key Components:**

- **Signers**: A sorted list of Ed25519 public keys authorized to sign
- **Threshold**: Minimum number of valid signatures required (M)
- **__check_auth**: Custom authentication function called by Soroban host

**Storage:**

- `Signers` → `Vec<BytesN<32>>`: Registered public keys (sorted)
- `Threshold` → `u32`: Required signature count
- `Initialized` → `bool`: Initialization flag

### 2. Account Factory Contract (`contracts/account-factory`)

Deploys new multisig account instances with a single transaction.

**Key Components:**

- **Admin**: Factory administrator address
- **Account WASM Hash**: Hash of the multisig account contract to deploy
- **Creator Tracking**: Maps creators to their deployed accounts

## Understanding __check_auth

The `__check_auth` function is the heart of the multisig account. It's called by the Soroban host whenever `require_auth()` is invoked on this contract's address.

### How It Works

1. **Signature Payload**: A hash of the operation being authorized (e.g., transfer 100 USDC)
2. **Signatures**: A vector of `SignerSig` structs, each containing a public key and signature
3. **Auth Contexts**: Additional context about the authorization (not used in basic implementation)

### Plain-Language Walkthrough

```rust
pub fn __check_auth(
    env: Env,
    signature_payload: Hash<32>,  // What is being signed
    signatures: Vec<SignerSig>,   // Who signed it
    _auth_contexts: Vec<Context>, // Additional context
) -> Result<(), MultisigError>
```

**Step-by-Step Process:**

1. **Load Configuration**: Retrieve the registered signers and threshold from storage
2. **Validate Sorting**: Iterate through signatures and ensure each public key is strictly greater than the previous one
3. **Check Registration**: For each signature, verify the public key is in the registered signer list
4. **Verify Cryptography**: Use `env.crypto().ed25519_verify()` to validate each signature
5. **Count Valid Signatures**: Increment counter for each valid signature
6. **Enforce Threshold**: Ensure valid count ≥ threshold

**Why This Matters:**

- If a signature is invalid or from an unknown key → reject immediately
- If signatures are out of order or duplicated → reject (prevents double-counting)
- If valid signatures < threshold → reject
- Otherwise → accept the authorization

## Testing Guidelines

### Test-Driven Development

We follow **test-first development**. Write tests before implementation.

### Required Test Coverage

Every contract must have tests for:

1. ✅ Happy path scenarios
2. ✅ Threshold enforcement (met and not met)
3. ✅ Duplicate signature detection
4. ✅ Unknown signer rejection
5. ✅ Invalid signature rejection
6. ✅ Authorization requirements for admin functions
7. ✅ Initialization constraints
8. ✅ Edge cases (threshold = 0, threshold > N, etc.)

### Critical Test: Duplicate Signer Prevention

The test `test_check_auth_duplicate_signer_rejected` is **mission-critical**. It ensures that submitting the same signer's signature twice does NOT count as two approvals.

**Why It's Important:**

If an attacker could submit the same signature twice, they could meet a 2-of-3 threshold with only 1 actual signer. This would completely break the multisig security model.

**How It's Prevented:**

Signatures must be sorted by public key in ascending order. The `__check_auth` function checks that each signature's public key is strictly greater than the previous one. If they're equal (duplicate) or out of order, the function returns `SignaturesNotSorted`.

```rust
// Check for proper sorting and no duplicates
if let Some(ref prev) = prev_key {
    if sig.public_key <= *prev {
        return Err(MultisigError::SignaturesNotSorted);
    }
}
```

## Signature Ordering and Deduplication

### Why Sorting Matters

Soroban custom accounts receive signatures as a vector. Without enforcing order, a malicious caller could:

1. Submit the same signature multiple times
2. Count it as multiple approvals
3. Meet threshold with fewer unique signers than required

### How We Enforce It

**At Initialization:**
```rust
let mut sorted_signers = signers.clone();
sorted_signers.sort();
// Check for duplicates after sorting
```

**In __check_auth:**
```rust
let mut prev_key: Option<BytesN<32>> = None;
for sig in signatures.iter() {
    if let Some(ref prev) = prev_key {
        if sig.public_key <= *prev {
            return Err(MultisigError::SignaturesNotSorted);
        }
    }
    prev_key = Some(sig.public_key.clone());
    // ... verify signature ...
}
```

**Key Properties:**

- Signatures must be sorted in ascending order by public key
- Equal keys are rejected (prevents duplicates)
- Out-of-order keys are rejected (enforces client-side sorting)
- This ensures each unique signer is counted at most once

## Extending Signature Types

Currently, the contract supports **Ed25519** signatures. To add a new signature type (e.g., secp256r1, secp256k1):

### Step 1: Extend the SignerSig Enum

```rust
#[contracttype]
pub enum SignerSig {
    Ed25519 {
        public_key: BytesN<32>,
        signature: BytesN<64>,
    },
    Secp256r1 {
        public_key: BytesN<33>,  // Compressed format
        signature: BytesN<64>,
    },
}
```

### Step 2: Update Signer Storage

Store signer type alongside the key:

```rust
#[contracttype]
pub struct Signer {
    pub key: Bytes,
    pub sig_type: SignerType,
}
```

### Step 3: Extend __check_auth Verification

```rust
match sig {
    SignerSig::Ed25519 { public_key, signature } => {
        env.crypto().ed25519_verify(public_key, payload, signature);
    }
    SignerSig::Secp256r1 { public_key, signature } => {
        env.crypto().secp256r1_verify(public_key, payload, signature);
    }
}
```

### Step 4: Maintain Threshold Logic Integrity

**CRITICAL**: Ensure that regardless of signature type, each signer is counted only once. The sorting and deduplication logic must work across signature types.

**Recommendation**: Sort by a normalized identifier (e.g., hash of public key + type) to ensure consistent ordering across different key types.

## Deployment

### Build for Release

```bash
cargo build --release --target wasm32-unknown-unknown
```

Optimized WASM files will be in:
```
target/wasm32-unknown-unknown/release/multisig_account.wasm
target/wasm32-unknown-unknown/release/account_factory.wasm
```

### Deploy to Testnet

```bash
./scripts/deploy.sh
```

### Manual Deployment

```bash
# Deploy multisig account contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/multisig_account.wasm \
  --network testnet \
  --source <YOUR_SECRET_KEY>

# Deploy factory contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/account_factory.wasm \
  --network testnet \
  --source <YOUR_SECRET_KEY>
```

### Initialize Factory

```bash
soroban contract invoke \
  --id <FACTORY_CONTRACT_ID> \
  --network testnet \
  --source <YOUR_SECRET_KEY> \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --account-wasm-hash <MULTISIG_WASM_HASH>
```

## Code Quality Standards

### No Panics

Use `Result<T, E>` and error enums. Never use `panic!()`, `unwrap()`, or `expect()` in contract code.

**Bad:**
```rust
let threshold = env.storage().get(&DataKey::Threshold).unwrap();
```

**Good:**
```rust
let threshold: u32 = env.storage()
    .instance()
    .get(&DataKey::Threshold)
    .ok_or(MultisigError::NotInitialized)?;
```

### Event Emission

Emit events for all state changes:

```rust
env.events().publish((MultisigEvent::SignerAdded,), signer);
```

### Authorization Checks

Admin functions must require contract authorization:

```rust
pub fn add_signer(env: Env, signer: BytesN<32>) -> Result<(), MultisigError> {
    env.current_contract_address().require_auth();
    // ... rest of function
}
```

## Commit Guidelines

- **Minimum 35 commits**: Break work into logical, atomic changes
- **Descriptive messages**: Explain what and why, not just what
- **Format**: `<type>: <description>` (e.g., `feat: add duplicate signer detection`)

### Commit Types

- `feat`: New feature
- `fix`: Bug fix
- `test`: Add or update tests
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `chore`: Maintenance tasks

## Sister Repositories

- **Web Interface**: [stellar-multisig-web](https://github.com/<your-org>/stellar-multisig-web)
- **API & Docs**: [stellar-multisig-api-docs](https://github.com/<your-org>/stellar-multisig-api-docs)

## Getting Help

- Open an issue for bugs or feature requests
- Join our Discord for real-time discussions
- Review existing PRs for examples

## License

MIT - see LICENSE file for details
