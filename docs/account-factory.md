# Account Factory Contract

The account factory contract simplifies the deployment of multisig accounts by providing a single-transaction creation interface.

## Overview

Instead of deploying a multisig account contract manually and initializing it separately, users can call the factory's `create_account` function to deploy and initialize a new multisig account in one step.

## Contract Interface

### Initialization

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    account_wasm_hash: BytesN<32>
) -> Result<(), FactoryError>
```

Initialize the factory with an admin address and the WASM hash of the multisig account contract to deploy.

**Parameters:**
- `admin`: Administrator address for the factory
- `account_wasm_hash`: Hash of the deployed multisig account WASM

**Constraints:**
- Can only be called once (returns `AlreadyInitialized`)

**Example:**
```rust
let admin = Address::generate(&env);
let wasm_hash = env.deployer().upload_contract_wasm(multisig_wasm);

client.initialize(&admin, &wasm_hash)?;
```

### Account Creation

```rust
pub fn create_account(
    env: Env,
    creator: Address,
    salt: BytesN<32>,
    signers: Vec<BytesN<32>>,
    threshold: u32
) -> Result<Address, FactoryError>
```

Deploy and initialize a new multisig account.

**Parameters:**
- `creator`: Address of the account creator (must sign the transaction)
- `salt`: Random salt for deterministic address generation
- `signers`: Vector of Ed25519 public keys for the multisig
- `threshold`: Minimum signatures required (M-of-N)

**Returns:**
- `Address`: The deployed multisig account's contract address

**Constraints:**
- Creator must authorize the transaction (`creator.require_auth()`)
- `threshold` must be ≥ 1 and ≤ `signers.len()`

**Process:**
1. Validate threshold
2. Deploy multisig account contract using salt (deterministic address)
3. Initialize the deployed account with signers and threshold
4. Track the account for the creator
5. Increment total account count
6. Emit `AccountCreated` event

**Example:**
```rust
let creator = Address::generate(&env);
let salt = BytesN::random(&env);

let signers = vec![&env, pubkey_alice, pubkey_bob, pubkey_charlie];
let threshold = 2;

let multisig_address = client.create_account(
    &creator,
    &salt,
    &signers,
    &threshold
)?;

// multisig_address is now a fully initialized 2-of-3 multisig account
```

### Query Functions

#### Get Accounts by Creator

```rust
pub fn get_accounts_by_creator(env: Env, creator: Address) -> Vec<Address>
```

Returns all multisig accounts created by a specific address.

**Parameters:**
- `creator`: Address to query

**Returns:**
- `Vec<Address>`: List of multisig account addresses (empty if none)

**Example:**
```rust
let creator = Address::generate(&env);
let accounts = client.get_accounts_by_creator(&creator);

for account in accounts.iter() {
    println!("Multisig account: {:?}", account);
}
```

#### Get Account Count

```rust
pub fn account_count(env: Env) -> u64
```

Returns the total number of accounts created by this factory.

**Example:**
```rust
let count = client.account_count();
println!("Total accounts created: {}", count);
```

#### Get Admin

```rust
pub fn get_admin(env: Env) -> Result<Address, FactoryError>
```

Returns the factory administrator address.

#### Get Account WASM Hash

```rust
pub fn get_account_wasm_hash(env: Env) -> Result<BytesN<32>, FactoryError>
```

Returns the WASM hash of the multisig account contract being deployed.

## Error Types

```rust
pub enum FactoryError {
    AlreadyInitialized = 1,  // initialize() called twice
    NotInitialized = 2,      // Factory not yet initialized
    InvalidThreshold = 3,    // Threshold is 0 or > signer count
}
```

## Events

```rust
pub enum FactoryEvent {
    Initialized,       // Factory initialized
    AccountCreated,    // New multisig account deployed
}
```

**Event Data:**

- `Initialized`: `(admin: Address, account_wasm_hash: BytesN<32>)`
- `AccountCreated`: `(creator: Address, account_address: Address, threshold: u32, signer_count: u32)`

## Deterministic Addresses

The factory uses Soroban's contract deployer with a salt to generate deterministic addresses:

```rust
let deployed_address = env
    .deployer()
    .with_current_contract(salt)
    .deploy(account_wasm_hash);
```

**Key Properties:**

- Same salt + same factory = same address
- Different salt = different address
- Users can pre-compute addresses before deployment
- Enables CREATE2-style address prediction

**Example:**
```rust
// Pre-compute the address before deploying
let predicted_address = env
    .deployer()
    .with_current_contract(salt)
    .deployed_address(account_wasm_hash);

// Deploy
let actual_address = client.create_account(
    &creator,
    &salt,
    &signers,
    &threshold
)?;

assert_eq!(predicted_address, actual_address);
```

## Usage Patterns

### Single-Transaction Multisig Creation

The primary benefit of the factory is creating a fully functional multisig account in one transaction:

```rust
// Without factory: 3 transactions
// 1. Deploy multisig contract
// 2. Upload WASM
// 3. Initialize

// With factory: 1 transaction
let multisig = factory_client.create_account(
    &creator,
    &salt,
    &signers,
    &threshold
)?;
```

### Tracking User Accounts

The factory maintains a mapping of creators to their accounts:

```rust
// Creator A makes 2 accounts
let account1 = factory_client.create_account(&creator_a, &salt1, &signers1, &2)?;
let account2 = factory_client.create_account(&creator_a, &salt2, &signers2, &3)?;

// Query all accounts by creator A
let accounts = factory_client.get_accounts_by_creator(&creator_a);
assert_eq!(accounts.len(), 2);
```

### Account Discovery

Applications can use the factory to discover all multisig accounts for a user without scanning the entire ledger.

## Integration with Web Interface

The [stellar-multisig-web](https://github.com/<your-org>/stellar-multisig-web) repository uses this factory to:

1. **Create Multisigs**: Single-click account creation
2. **List User Accounts**: Show all multisigs a user has created
3. **Track Deployments**: Monitor account creation events

Example workflow:
```
User → Web UI → Factory.create_account() → New Multisig Address → Store in UI
```

## Security Considerations

### Creator Authorization

The factory requires the creator to authorize account creation:

```rust
creator.require_auth();
```

This prevents unauthorized deployment of accounts on behalf of others.

### Threshold Validation

The factory validates threshold constraints before deployment:

```rust
if threshold == 0 || threshold > signers.len() {
    return Err(FactoryError::InvalidThreshold);
}
```

This ensures all created accounts are properly configured.

### No Admin Privileges on Created Accounts

The factory admin has NO special privileges on created multisig accounts. Once deployed, accounts are entirely self-governed by their signers.

## Testing

The factory contract includes tests for:

- ✅ Initialization (success and duplicate prevention)
- ✅ Invalid threshold rejection
- ✅ Account creation and tracking
- ✅ Creator account queries
- ✅ Account count incrementing
- ✅ Uninitialized factory error handling

Run tests:
```bash
cargo test -p account-factory
```

## Deployment

### Deploy Factory

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/account_factory.wasm \
  --network testnet \
  --source <YOUR_SECRET_KEY>
```

### Upload Multisig WASM

```bash
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/multisig_account.wasm \
  --network testnet \
  --source <YOUR_SECRET_KEY>
```

This returns a WASM hash.

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

### Create an Account

```bash
soroban contract invoke \
  --id <FACTORY_CONTRACT_ID> \
  --network testnet \
  --source <CREATOR_SECRET_KEY> \
  -- create_account \
  --creator <CREATOR_ADDRESS> \
  --salt <32_BYTE_SALT> \
  --signers '[<PUBKEY1>, <PUBKEY2>, <PUBKEY3>]' \
  --threshold 2
```

## Sister Repositories

- **Web Interface**: [stellar-multisig-web](https://github.com/<your-org>/stellar-multisig-web)
- **API & Docs**: [stellar-multisig-api-docs](https://github.com/<your-org>/stellar-multisig-api-docs)

## License

MIT
