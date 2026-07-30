# Deployment Configuration

This document describes environment variables, secret management, and
configuration options for deploying the StellarAid platform.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `SOROBAN_RPC_URL` | Yes | — | Soroban RPC endpoint (e.g., `https://soroban-testnet.stellar.org`) |
| `HORIZON_URL` | Yes | — | Horizon API endpoint (e.g., `https://horizon-testnet.stellar.org`) |
| `NETWORK_PASSPHRASE` | Yes | — | Network passphrase: `Test SDF Network ; September 2015` for testnet |
| `CONTRACT_DIR` | No | `./contracts` | Path to compiled `.wasm` files |
| `ADMIN_SECRET_KEY` | Yes | — | Admin Stellar secret key (seed) |
| `LOG_LEVEL` | No | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

## Secrets

Sensitive values must never be committed to the repository.

- Store `ADMIN_SECRET_KEY` in a vault (e.g., Vault, AWS Secrets Manager, 1Password)
- Use environment-specific `.env` files that are git-ignored
- Rotate keys between testnet and mainnet deployments

## Network Configuration

The `scripts/deploy.sh` script supports:

- **testnet**: `--network testnet` (default)
- **mainnet**: `--network mainnet`

Additional flags:

```
--dry-run          Validate configuration without deploying
--wasm <path>      Path to the contract WASM file
--admin <address>  Override admin address
```

## Contract Initialization

After deployment, each contract must be initialized via its `initialize` function:

```bash
# Example: initialize campaign contract
soroban contract invoke \
  --id <CONTRACT_ID> \
  --rpc-url $SOROBAN_RPC_URL \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source $ADMIN_SECRET_KEY \
  -- \
  initialize \
  --admin <ADMIN_ADDRESS>
```

## Health Checks

The worker exposes:

- `GET /health` — JSON with uptime, donation count, error count, last activity
- `GET /ready` — 200 OK when the worker is ready to serve traffic

## CI/CD

The `.github/workflows/ci.yml` workflow:

1. Builds all Rust contracts (wasm32 target)
2. Runs Rust test suite
3. Runs TypeScript SDK tests
4. Runs clippy and rustfmt
