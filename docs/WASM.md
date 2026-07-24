# Wasm Compilation

Soroban smart contracts compile to WebAssembly (Wasm). This repo uses `rust-toolchain.toml` to pin the Rust toolchain.

## Why wasm32-unknown-unknown?

Soroban contracts run on the Stellar network inside a Wasm VM. The `wasm32-unknown-unknown` target produces a Wasm binary with no OS dependencies, which is required by the Soroban runtime.

## Building

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled `.wasm` files appear in `target/wasm32-unknown-unknown/release/`.
