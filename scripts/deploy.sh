#!/bin/bash

# Stellar Multisig Contracts - Deployment Script
# This script deploys the multisig account and factory contracts to Stellar testnet

set -e

echo "🚀 Stellar Multisig Contracts Deployment"
echo "========================================"
echo ""

# Check if soroban CLI is installed
if ! command -v soroban &> /dev/null; then
    echo "❌ Error: soroban CLI is not installed"
    echo "Install it with: cargo install --locked soroban-cli --features opt"
    exit 1
fi

# Configuration
NETWORK="${NETWORK:-testnet}"
SOURCE_ACCOUNT="${SOURCE_ACCOUNT:-}"

if [ -z "$SOURCE_ACCOUNT" ]; then
    echo "❌ Error: SOURCE_ACCOUNT environment variable not set"
    echo "Usage: SOURCE_ACCOUNT=<your-secret-key> ./scripts/deploy.sh"
    exit 1
fi

echo "📦 Network: $NETWORK"
echo ""

# Step 1: Build contracts
echo "🔨 Building contracts..."
cargo build --release --target wasm32-unknown-unknown

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo ""

# Step 2: Deploy multisig account contract
echo "📤 Deploying multisig account contract..."
MULTISIG_WASM="target/wasm32-unknown-unknown/release/multisig_account.wasm"

if [ ! -f "$MULTISIG_WASM" ]; then
    echo "❌ Error: $MULTISIG_WASM not found"
    exit 1
fi

MULTISIG_HASH=$(soroban contract install \
    --wasm "$MULTISIG_WASM" \
    --network "$NETWORK" \
    --source "$SOURCE_ACCOUNT" 2>&1 | tail -n 1)

if [ -z "$MULTISIG_HASH" ]; then
    echo "❌ Failed to upload multisig account WASM"
    exit 1
fi

echo "✅ Multisig account WASM uploaded"
echo "   Hash: $MULTISIG_HASH"
echo ""

# Step 3: Deploy factory contract
echo "📤 Deploying factory contract..."
FACTORY_WASM="target/wasm32-unknown-unknown/release/account_factory.wasm"

if [ ! -f "$FACTORY_WASM" ]; then
    echo "❌ Error: $FACTORY_WASM not found"
    exit 1
fi

FACTORY_ID=$(soroban contract deploy \
    --wasm "$FACTORY_WASM" \
    --network "$NETWORK" \
    --source "$SOURCE_ACCOUNT" 2>&1 | tail -n 1)

if [ -z "$FACTORY_ID" ]; then
    echo "❌ Failed to deploy factory contract"
    exit 1
fi

echo "✅ Factory contract deployed"
echo "   Contract ID: $FACTORY_ID"
echo ""

# Step 4: Initialize factory
echo "🔧 Initializing factory..."

# Use the source account as admin (you can change this)
ADMIN_ADDRESS=$(soroban keys address "$SOURCE_ACCOUNT" 2>/dev/null || echo "$SOURCE_ACCOUNT")

soroban contract invoke \
    --id "$FACTORY_ID" \
    --network "$NETWORK" \
    --source "$SOURCE_ACCOUNT" \
    -- initialize \
    --admin "$ADMIN_ADDRESS" \
    --account_wasm_hash "$MULTISIG_HASH"

if [ $? -ne 0 ]; then
    echo "❌ Failed to initialize factory"
    exit 1
fi

echo "✅ Factory initialized"
echo ""

# Summary
echo "🎉 Deployment Complete!"
echo "======================="
echo ""
echo "Multisig Account WASM Hash: $MULTISIG_HASH"
echo "Factory Contract ID:        $FACTORY_ID"
echo "Factory Admin:              $ADMIN_ADDRESS"
echo "Network:                    $NETWORK"
echo ""
echo "📝 Next Steps:"
echo "  1. Update DEPLOYMENTS.md with these addresses"
echo "  2. Verify contracts on Stellar Expert"
echo "  3. Test account creation:"
echo ""
echo "     soroban contract invoke \\"
echo "       --id $FACTORY_ID \\"
echo "       --network $NETWORK \\"
echo "       --source \$SOURCE_ACCOUNT \\"
echo "       -- create_account \\"
echo "       --creator \$YOUR_ADDRESS \\"
echo "       --salt \$(xxd -l 32 -p /dev/urandom | tr -d '\n') \\"
echo "       --signers '[\$PUBKEY1, \$PUBKEY2, \$PUBKEY3]' \\"
echo "       --threshold 2"
echo ""
echo "🔗 Explorer:"
echo "   https://stellar.expert/explorer/$NETWORK/contract/$FACTORY_ID"
echo ""
