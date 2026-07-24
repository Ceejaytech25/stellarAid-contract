# Deployment Guide

## Prerequisites

- Rust + `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Soroban CLI: `cargo install --locked soroban-cli`
- Copy `.env.example` to `.env` and fill in your values

## Network Setup

```bash
soroban network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

soroban network add mainnet \
  --rpc-url https://soroban.stellar.org \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

## Deploy

```bash
./scripts/deploy.sh testnet
./scripts/deploy.sh mainnet
```

## Invoke Example

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- hello
```
