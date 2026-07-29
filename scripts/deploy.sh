#!/bin/bash
set -e

NETWORK=${1:-testnet}
ADMIN_SECRET=${2:-$STELLAR_PLATFORM_SECRET}
DRY_RUN=false

# Parse optional flags
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
  esac
done

if [ -z "$ADMIN_SECRET" ]; then
  echo "Usage: $0 [network] [admin_secret] [--dry-run]"
  echo "Or set STELLAR_PLATFORM_SECRET environment variable."
  exit 1
fi

if [ "$NETWORK" = "testnet" ]; then
  RPC_URL="https://soroban-testnet.stellar.org"
  PASSPHRASE="Test SDF Network ; September 2015"
elif [ "$NETWORK" = "mainnet" ]; then
  RPC_URL="https://soroban.stellar.org"
  PASSPHRASE="Public Global Stellar Network ; September 2015"
else
  echo "Unknown network: $NETWORK. Use testnet or mainnet."
  exit 1
fi

echo "Network:  $NETWORK"
echo "RPC URL:  $RPC_URL"
if [ "$DRY_RUN" = true ]; then
  echo "*** DRY RUN — no state changes will be made ***"
fi

soroban network add "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE" 2>/dev/null || true

if [ "$DRY_RUN" = true ]; then
  echo "Dry-run: configuration validated successfully."
  echo "To deploy for real, run without --dry-run."
  exit 0
fi

echo "Building contracts..."
cargo build --target wasm32-unknown-unknown --release

CONFIG_FILE="config/${NETWORK}_contracts.json"

declare -A CONTRACT_IDS

CONTRACTS=("campaign" "donation" "withdrawal" "escrow" "platform_config" "dispute_arbiter" "commission_agreement")
for contract in "${CONTRACTS[@]}"; do
  WASM="target/wasm32-unknown-unknown/release/${contract}.wasm"
  if [ ! -f "$WASM" ]; then
    echo "Warning: $WASM not found, skipping $contract"
    continue
  fi
  echo "Deploying $contract..."
  CONTRACT_ID=$(soroban contract deploy \
    --wasm "$WASM" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$PASSPHRASE" \
    --source "$ADMIN_SECRET")
  CONTRACT_IDS[$contract]=$CONTRACT_ID
  echo "$contract contract ID: $CONTRACT_ID"
done

echo ""
echo "Initializing contracts..."

ADDR=$(soroban keys address --network "$NETWORK" --source "$ADMIN_SECRET")

if [ -n "${CONTRACT_IDS[campaign]:-}" ]; then
  echo "Initializing Campaign contract..."
  soroban contract invoke \
    --id "${CONTRACT_IDS[campaign]}" \
    --network "$NETWORK" \
    --source "$ADMIN_SECRET" \
    -- \
    initialize \
    --admin "$ADDR"
fi

if [ -n "${CONTRACT_IDS[donation]:-}" ]; then
  echo "Initializing Donation contract..."
  soroban contract invoke \
    --id "${CONTRACT_IDS[donation]}" \
    --network "$NETWORK" \
    --source "$ADMIN_SECRET" \
    -- \
    initialize \
    --admin "$ADDR" \
    --campaign_contract "${CONTRACT_IDS[campaign]}"
fi

if [ -n "${CONTRACT_IDS[withdrawal]:-}" ]; then
  echo "Initializing Withdrawal contract..."
  soroban contract invoke \
    --id "${CONTRACT_IDS[withdrawal]}" \
    --network "$NETWORK" \
    --source "$ADMIN_SECRET" \
    -- \
    initialize \
    --admin "$ADDR" \
    --donation_contract "${CONTRACT_IDS[donation]}"
fi

if [ -n "${CONTRACT_IDS[platform_config]:-}" ]; then
  echo "Initializing PlatformConfig contract..."
  soroban contract invoke \
    --id "${CONTRACT_IDS[platform_config]}" \
    --network "$NETWORK" \
    --source "$ADMIN_SECRET" \
    -- \
    initialize \
    --admin "$ADDR" \
    --fee_bps 500 \
    --platform_wallet "$ADDR" \
    --usdc_token "$ADDR"
fi

echo ""
echo "Deployment to $NETWORK complete!"
echo "Contract IDs saved to $CONFIG_FILE"
