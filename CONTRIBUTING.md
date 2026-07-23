# Contributing to Lumora Contracts

## Dev Environment Setup

1. Install Rust: https://rustup.rs
2. Add Wasm target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. Install Soroban CLI:
   ```bash
   cargo install --locked soroban-cli --features opt
   ```
4. Copy env file:
   ```bash
   cp .env.example .env
   ```

## Running Tests

```bash
cargo test
```

## PR Guidelines

- Every PR must include unit tests for new functionality
- All PRs must pass `cargo clippy -- -D warnings`
- Use conventional commit messages: `feat:`, `fix:`, `chore:`, `docs:`, `test:`
- One issue per PR — never combine multiple issues

## Security Checklist

- Never commit private keys or secret values
- Always use `require_auth` for privileged operations
- Follow the CEI pattern (Checks-Effects-Interactions)
- Validate all inputs before state changes

## Resources

- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Soroban SDK](https://docs.rs/soroban-sdk)
