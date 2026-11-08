# Deployment Addresses

This document tracks all deployed contract instances across Stellar networks.

## Stellar Testnet

### Multisig Account Contract

- **Contract ID**: `TBD`
- **WASM Hash**: `TBD`
- **Deployed**: `TBD`
- **Version**: `0.1.0`
- **Network**: Stellar Testnet
- **Explorer**: `https://stellar.expert/explorer/testnet/contract/TBD`

### Account Factory Contract

- **Contract ID**: `TBD`
- **Admin**: `TBD`
- **Account WASM Hash**: `TBD`
- **Deployed**: `TBD`
- **Version**: `0.1.0`
- **Network**: Stellar Testnet
- **Explorer**: `https://stellar.expert/explorer/testnet/contract/TBD`

## Stellar Mainnet

### Multisig Account Contract

- **Contract ID**: `Not yet deployed`
- **WASM Hash**: `Not yet deployed`
- **Deployed**: `N/A`
- **Version**: `N/A`
- **Network**: Stellar Mainnet
- **Explorer**: `N/A`

### Account Factory Contract

- **Contract ID**: `Not yet deployed`
- **Admin**: `N/A`
- **Account WASM Hash**: `N/A`
- **Deployed**: `N/A`
- **Version**: `N/A`
- **Network**: Stellar Mainnet
- **Explorer**: `N/A`

## Deployment History

### Version 0.1.0 (Initial Release)

**Testnet:**
- `TBD`: Multisig account contract deployed
- `TBD`: Factory contract deployed and initialized

**Features:**
- M-of-N signature verification
- Ed25519 signature support
- Signer management (add/remove)
- Threshold adjustment
- Factory-based deployment

**Known Issues:**
- None

---

## How to Deploy

See the deployment script:

```bash
./scripts/deploy.sh
```

Or follow the manual deployment process in [CONTRIBUTING.md](./CONTRIBUTING.md#deployment).

## Verification

### Verify Contract Code

```bash
# Get deployed WASM hash
soroban contract info \
  --id <CONTRACT_ID> \
  --network testnet

# Compare with local build
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/multisig_account.wasm \
  --network testnet
```

### Verify Factory Configuration

```bash
# Check factory admin
soroban contract invoke \
  --id <FACTORY_ID> \
  --network testnet \
  -- get_admin

# Check account WASM hash
soroban contract invoke \
  --id <FACTORY_ID> \
  --network testnet \
  -- get_account_wasm_hash
```

## Emergency Contacts

- **Maintainer**: `<your-contact>`
- **Security Issues**: `security@<your-domain>`
- **Discord**: `<your-discord>`

## Upgrade Plan

Soroban contracts are immutable. Upgrades require deploying new contracts and migrating state. See [CONTRIBUTING.md](./CONTRIBUTING.md) for upgrade procedures.

---

Last Updated: `TBD`
