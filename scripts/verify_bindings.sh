#!/usr/bin/env bash
set -euo pipefail

# verify_bindings.sh — Verify that TypeScript/JS SDK bindings match the
# latest compiled contract WASM specs.
#
# Usage:
#   ./scripts/verify_bindings.sh [contract_dir] [binding_dir]
#
# Defaults:
#   contract_dir = ./contracts
#   binding_dir  = ./sdk/bindings

CONTRACT_DIR="${1:-./contracts}"
BINDING_DIR="${2:-./sdk/bindings}"
EXIT_CODE=0

if ! command -v soroban &> /dev/null; then
    echo "Error: 'soroban' CLI not found."
    exit 1
fi

echo "Verifying bindings against compiled contracts..."
echo "  Contracts: $CONTRACT_DIR"
echo "  Bindings:  $BINDING_DIR"

for contract_wasm in "$CONTRACT_DIR"/*/target/wasm32-unknown-unknown/release/*.wasm; do
    [ -f "$contract_wasm" ] || continue
    contract_name="$(basename "$contract_wasm" .wasm)"

    spec_file="$BINDING_DIR/${contract_name}.json"
    if [ ! -f "$spec_file" ]; then
        echo "  MISSING: $spec_file (no binding spec found)"
        EXIT_CODE=1
        continue
    fi

    # Compare the generated spec against the checked-in binding
    tmp_spec=$(mktemp)
    soroban contract spec --wasm "$contract_wasm" --output "$tmp_spec" 2>/dev/null || true

    if diff -q "$tmp_spec" "$spec_file" >/dev/null 2>&1; then
        echo "  OK: $contract_name"
    else
        echo "  MISMATCH: $contract_name (bindings are stale)"
        EXIT_CODE=1
    fi
    rm -f "$tmp_spec"
done

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "All bindings are up to date."
else
    echo "Some bindings need updating. Re-run generate_abi.sh and commit the results."
fi

exit $EXIT_CODE
