#!/usr/bin/env bash
set -euo pipefail

# generate_abi.sh — Generate Soroban ABI (contract spec) JSON for all contracts.
#
# Usage:
#   ./scripts/generate_abi.sh [contract_dir] [output_dir]
#
# Defaults:
#   contract_dir = ./contracts
#   output_dir   = ./abis

CONTRACT_DIR="${1:-./contracts}"
OUTPUT_DIR="${2:-./abis}"

if ! command -v soroban &> /dev/null; then
    echo "Error: 'soroban' CLI not found. Install it with:"
    echo "  cargo install soroban-cli"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

echo "Generating ABIs from: $CONTRACT_DIR"
echo "Output directory: $OUTPUT_DIR"

for dir in "$CONTRACT_DIR"/*/; do
    contract_name="$(basename "$dir")"
    wasm_path="$dir/target/wasm32-unknown-unknown/release/${contract_name//-/_}.wasm"

    if [ -f "$wasm_path" ]; then
        echo "  Generating ABI for: $contract_name"
        soroban contract spec \
            --wasm "$wasm_path" \
            --output "$OUTPUT_DIR/${contract_name}.json" 2>/dev/null || {
            echo "    Warning: ABI generation failed for $contract_name (may need build first)"
        }
    else
        echo "  Skipping $contract_name (no wasm found at $wasm_path)"
    fi
done

echo "Done. ABI files written to $OUTPUT_DIR/"
