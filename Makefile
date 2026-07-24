build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings

deploy-testnet:
	./scripts/deploy_testnet.sh

clean:
	cargo clean

.PHONY: build test fmt lint deploy-testnet clean
