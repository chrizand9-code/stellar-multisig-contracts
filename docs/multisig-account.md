# Multisig Account Contract

The multisig account contract is a Soroban custom account implementation that requires M-of-N signatures to authorize any action.

## Overview

This contract acts as a smart account address on Stellar. Any operation that requires authorization from this address (token transfers, contract invocations, etc.) must pass through the `__check_auth` function with valid signatures from at least M of N registered signers.

## Contract Interface

### Initialization

```rust
pub fn init(
    env: Env,
    signers: Vec<BytesN<32>>,
    threshold: u32
) -> Result<(), MultisigError>
```

Initialize the multisig account with a set of signers and a threshold. Can only be called once.

**Parameters:**
- `signers`: Vector of Ed25519 public keys (32 bytes each)
- `threshold`: Minimum number of signatures required (M)

**Constraints:**
- `threshold` must be ≥ 1 and ≤ `signers.len()`
- No duplicate signers allowed
- Cannot be called twice (returns `AlreadyInitialized`)

**Example:**
```rust
let signers = vec![
    &env,
    pubkey_alice,
    pubkey_bob,
    pubkey_charlie,
];
let threshold = 2; // Require 2 out of 3

client.init(&signers, &threshold)?;
```

### Custom Account Authentication

```rust
pub fn __check_auth(
    env: Env,
    signature_payload: Hash<32>,
    signatures: Vec<SignerSig>,
    auth_contexts: Vec<Context>
) -> Result<(), MultisigError>
```

Called by the Soroban host when `require_auth()` is invoked on this contract address.

**Parameters:**
- `signature_payload`: Hash of the operation being authorized
- `signatures`: Vector of signatures, each containing a public key and signature
- `auth_contexts`: Additional authorization context (unused in basic implementation)

**Validation Process:**

1. **Load State**: Get registered signers and threshold from storage
2. **Sort Validation**: Ensure signatures are sorted by public key (ascending)
3. **Duplicate Check**: Reject if any two consecutive signatures have the same public key
4. **Signer Verification**: Each public key must be in the registered signer list
5. **Signature Verification**: Verify each signature using Ed25519
6. **Threshold Check**: Count valid signatures and ensure count ≥ threshold

**Signature Format:**
```rust
pub struct SignerSig {
    pub public_key: BytesN<32>,
    pub signature: BytesN<64>,
}
```

**Example:**
```rust
// Create a payload (this is done by the Soroban host in production)
let payload = Hash::from_array(&env, &[5u8; 32]);

// Sign with 2 of 3 signers
let signatures = vec![
    &env,
    SignerSig {
        public_key: pubkey_alice,
        signature: sign_with_alice(payload),
    },
    SignerSig {
        public_key: pubkey_bob,
        signature: sign_with_bob(payload),
    },
];

client.__check_auth(&payload, &signatures, &vec![&env])?;
```

### Signer Management

#### Add Signer

```rust
pub fn add_signer(env: Env, signer: BytesN<32>) -> Result<(), MultisigError>
```

Add a new signer to the account. **Requires multisig authorization** (calls `require_auth()` on the contract address).

**Parameters:**
- `signer`: Ed25519 public key to add

**Constraints:**
- Signer must not already exist (returns `DuplicateSigner`)

**Authorization Required**: ✅ Yes (M-of-N signatures)

#### Remove Signer

```rust
pub fn remove_signer(env: Env, signer: BytesN<32>) -> Result<(), MultisigError>
```

Remove an existing signer. **Requires multisig authorization**.

**Parameters:**
- `signer`: Ed25519 public key to remove

**Constraints:**
- Signer must exist (returns `UnknownSigner`)
- Removal must not violate threshold (i.e., threshold ≤ remaining signer count)

**Authorization Required**: ✅ Yes (M-of-N signatures)

#### Set Threshold

```rust
pub fn set_threshold(env: Env, threshold: u32) -> Result<(), MultisigError>
```

Change the signature threshold. **Requires multisig authorization**.

**Parameters:**
- `threshold`: New minimum signature count

**Constraints:**
- `threshold` must be ≥ 1 and ≤ current signer count

**Authorization Required**: ✅ Yes (M-of-N signatures)

### Query Functions

#### Get Signers

```rust
pub fn get_signers(env: Env) -> Result<Vec<BytesN<32>>, MultisigError>
```

Returns the list of registered signers (sorted).

#### Get Threshold

```rust
pub fn get_threshold(env: Env) -> Result<u32, MultisigError>
```

Returns the current signature threshold.

## Error Types

```rust
pub enum MultisigError {
    NotInitialized = 1,          // Contract not yet initialized
    AlreadyInitialized = 2,      // init() called twice
    InvalidThreshold = 3,        // Threshold is 0 or > signer count
    UnknownSigner = 4,           // Signature from unregistered key
    DuplicateSigner = 5,         // Same signer appears twice
    BadSignature = 6,            // Signature verification failed
    ThresholdNotMet = 7,         // Not enough valid signatures
    SignaturesNotSorted = 8,     // Signatures not in ascending order
}
```

## Events

The contract emits events for important state changes:

```rust
pub enum MultisigEvent {
    Initialized,       // Contract initialized
    SignerAdded,       // New signer added
    SignerRemoved,     // Signer removed
    ThresholdChanged,  // Threshold updated
}
```

**Event Data:**

- `Initialized`: `(threshold: u32, signer_count: u32)`
- `SignerAdded`: `(signer: BytesN<32>)`
- `SignerRemoved`: `(signer: BytesN<32>)`
- `ThresholdChanged`: `(new_threshold: u32)`

## Security Considerations

### Signature Ordering and Deduplication

Signatures MUST be sorted in ascending order by public key. This prevents:

1. **Duplicate Counting**: Submitting the same signature twice to inflate the count
2. **Replay Attacks**: Reusing signatures in different contexts

The `__check_auth` function rejects any signature list that is not strictly sorted:

```rust
if sig.public_key <= prev_key {
    return Err(MultisigError::SignaturesNotSorted);
}
```

### Self-Authorization

All admin functions (`add_signer`, `remove_signer`, `set_threshold`) require authorization from the contract's own address. This means:

1. To add a signer, you need M-of-N current signers to approve
2. To remove a signer, you need M-of-N current signers to approve
3. To change the threshold, you need M-of-N current signers to approve

This creates a self-governing system where the multisig controls its own configuration.

### Threshold Constraints

The contract enforces that:

- Threshold is always ≥ 1 (at least one signature required)
- Threshold is always ≤ signer count (can't require more signatures than signers exist)
- These constraints are checked at initialization, when adding/removing signers, and when changing threshold

### No Panics

The contract never panics. All error conditions return `Result<T, MultisigError>`.

## Usage Example

### Create a 2-of-3 Multisig

```rust
use soroban_sdk::{vec, BytesN, Env};

let env = Env::default();
let contract_id = env.register_contract(None, MultisigAccount);
let client = MultisigAccountClient::new(&env, &contract_id);

// 3 signers
let alice = BytesN::from_array(&env, &[1u8; 32]);
let bob = BytesN::from_array(&env, &[2u8; 32]);
let charlie = BytesN::from_array(&env, &[3u8; 32]);

let signers = vec![&env, alice, bob, charlie];
client.init(&signers, &2)?; // 2-of-3 threshold
```

### Authorize a Transfer

```rust
// In production, this is handled by the Soroban host
// When a token transfer is initiated:
// token.transfer(&multisig_address, &recipient, &amount)

// The host calls __check_auth on the multisig_address with:
// 1. A hash of the transfer operation
// 2. Signatures from the signers

let payload = /* hash of transfer operation */;
let signatures = vec![
    &env,
    SignerSig {
        public_key: alice,
        signature: /* alice's signature */,
    },
    SignerSig {
        public_key: bob,
        signature: /* bob's signature */,
    },
];

// This is called internally by the host
client.__check_auth(&payload, &signatures, &vec![&env])?;
// If successful, the transfer proceeds
```

### Add a New Signer

```rust
// This requires M-of-N authorization from the contract itself
let dave = BytesN::from_array(&env, &[4u8; 32]);

// The client would need to provide M signatures to authorize this
client.add_signer(&dave)?;

// Now the account is 2-of-4
```

## Integration with Sister Repositories

- **Web Interface** ([stellar-multisig-web](https://github.com/<your-org>/stellar-multisig-web)): Provides a UI for creating accounts, managing signers, and signing transactions
- **API & Docs** ([stellar-multisig-api-docs](https://github.com/<your-org>/stellar-multisig-api-docs)): Backend services for transaction coordination and documentation

## Testing

The contract includes comprehensive tests covering:

- ✅ Initialization with various thresholds
- ✅ Threshold enforcement (met and not met)
- ✅ Duplicate signer detection at init and in signatures
- ✅ Unknown signer rejection
- ✅ Signature verification
- ✅ Authorization requirements for admin functions
- ✅ Threshold validation when removing signers
- ✅ Edge cases (threshold = 0, threshold > N, etc.)

Run tests:
```bash
cargo test -p multisig-account
```

## Deployment

See [DEPLOYMENTS.md](../DEPLOYMENTS.md) for testnet and mainnet contract addresses.

## License

MIT
