#!/bin/bash
set -e

NETWORK=${1:-testnet}
ADMIN_SECRET=${2:-$STELLAR_PLATFORM_SECRET}

# ── Preflight checks (#523) ──────────────────────────────────────────────────

preflight_check() {
  local var_name="$1"
  local var_value="$2"
  if [ -z "$var_value" ]; then
    echo "ERROR: Required variable $var_name is not set."
    echo ""
    echo "Usage: $0 [network] [admin_secret]"
    echo "Or set STELLAR_PLATFORM_SECRET environment variable."
    exit 1
  fi
  echo "  ✓ $var_name is set"
}

echo "Running deployment preflight checks..."
preflight_check "NETWORK" "$NETWORK"
preflight_check "ADMIN_SECRET" "$ADMIN_SECRET"

# Verify required CLI tools
for cmd in soroban cargo; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "ERROR: Required command '$cmd' not found in PATH."
    exit 1
  fi
  echo "  ✓ $cmd is available"
done

# Verify wasm target is installed
if ! rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
  echo "ERROR: wasm32-unknown-unknown target not installed."
  echo "Run: rustup target add wasm32-unknown-unknown"
  exit 1
fi
echo "  ✓ wasm32-unknown-unknown target is installed"

# Validate network selection
case "$NETWORK" in
  testnet|mainnet)
    echo "  ✓ NETWORK=$NETWORK is valid"
    ;;
  localhost)
    echo "  ✓ NETWORK=$NETWORK (local development)"
    RPC_URL="http://localhost:8000/soroban/rpc"
    PASSPHRASE="Standalone Network ; February 2017"
    ;;
  *)
    echo "ERROR: Unknown network '$NETWORK'. Use: testnet, mainnet, or localhost."
    echo "Run with --dry-run to validate configuration without deploying."
    exit 1
    ;;
esac

# Dry-run mode: just validate and exit
if [ "${2:-}" = "--dry-run" ] || [ "${DRY_RUN:-}" = "true" ]; then
  echo ""
  echo "Dry-run: all preflight checks passed. Set DRY_RUN=false to deploy."
  exit 0
fi

# ── Network configuration ────────────────────────────────────────────────────

RPC_URL=${RPC_URL:-}
PASSPHRASE=${PASSPHRASE:-}

if [ "$NETWORK" = "testnet" ]; then
  RPC_URL="https://soroban-testnet.stellar.org"
  PASSPHRASE="Test SDF Network ; September 2015"
elif [ "$NETWORK" = "mainnet" ]; then
  RPC_URL="https://soroban.stellar.org"
  PASSPHRASE="Public Global Stellar Network ; September 2015"
fi

echo ""
echo "Configuring Soroban network: $NETWORK"
soroban network add "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE" 2>/dev/null || true

echo ""
echo "Building contracts..."
cargo build --target wasm32-unknown-unknown --release

CONFIG_FILE="config/${NETWORK}_contracts.json"

declare -A CONTRACT_IDS

# Deploy in dependency order: campaign -> donation -> withdrawal
CONTRACTS=("campaign" "donation" "withdrawal")
for contract in "${CONTRACTS[@]}"; do
  WASM="target/wasm32-unknown-unknown/release/${contract}.wasm"
  if [ ! -f "$WASM" ]; then
    echo "ERROR: WASM file not found: $WASM"
    exit 1
  fi
  echo ""
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

# 1. Initialize Campaign
echo "Initializing Campaign contract..."
soroban contract invoke \
  --id "${CONTRACT_IDS[campaign]}" \
  --network "$NETWORK" \
  --source "$ADMIN_SECRET" \
  -- \
  initialize \
  --admin "$(soroban keys address --network "$NETWORK" --source "$ADMIN_SECRET")"

# 2. Initialize Donation (depends on campaign contract)
echo "Initializing Donation contract..."
soroban contract invoke \
  --id "${CONTRACT_IDS[donation]}" \
  --network "$NETWORK" \
  --source "$ADMIN_SECRET" \
  -- \
  initialize \
  --admin "$(soroban keys address --network "$NETWORK" --source "$ADMIN_SECRET")" \
  --campaign_contract "${CONTRACT_IDS[campaign]}"

# 3. Initialize Withdrawal (depends on donation contract)
echo "Initializing Withdrawal contract..."
soroban contract invoke \
  --id "${CONTRACT_IDS[withdrawal]}" \
  --network "$NETWORK" \
  --source "$ADMIN_SECRET" \
  -- \
  initialize \
  --admin "$(soroban keys address --network "$NETWORK" --source "$ADMIN_SECRET")" \
  --donation_contract "${CONTRACT_IDS[donation]}"

echo ""
echo "Verifying deployment..."

for contract in "${CONTRACTS[@]}"; do
  echo "Verifying $contract..."
  case $contract in
    campaign)
      soroban contract invoke \
        --id "${CONTRACT_IDS[campaign]}" \
        --network "$NETWORK" \
        --source "$ADMIN_SECRET" \
        -- \
        get_campaign \
        --campaign_id 1 2>/dev/null || echo "  (no campaigns yet - expected)"
      ;;
    donation)
      TOTAL=$(soroban contract invoke \
        --id "${CONTRACT_IDS[donation]}" \
        --network "$NETWORK" \
        --source "$ADMIN_SECRET" \
        -- \
        get_total_raised \
        --campaign_id 1 2>/dev/null || echo "0")
      echo "  Total raised for campaign 1: $TOTAL"
      ;;
    withdrawal)
      COUNT=$(soroban contract invoke \
        --id "${CONTRACT_IDS[withdrawal]}" \
        --network "$NETWORK" \
        --source "$ADMIN_SECRET" \
        -- \
        get_withdrawals_by_campaign \
        --campaign_id 1 2>/dev/null || echo "[]")
      echo "  Withdrawals for campaign 1: $COUNT"
      ;;
  esac
done

echo ""
echo "Saving contract IDs to $CONFIG_FILE..."
cat > "$CONFIG_FILE" << EOF
{
  "network": "$NETWORK",
  "rpc_url": "$RPC_URL",
  "network_passphrase": "$PASSPHRASE",
  "contracts": {
    "campaign": { "id": "${CONTRACT_IDS[campaign]}" },
    "donation": { "id": "${CONTRACT_IDS[donation]}" },
    "withdrawal": { "id": "${CONTRACT_IDS[withdrawal]}" }
  }
}
EOF

echo ""
echo "Deployment to $NETWORK complete!"
echo "Contract IDs saved to $CONFIG_FILE"
