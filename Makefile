# InheritVault — common tasks

.PHONY: test build fmt fmt-check clean deploy-testnet

test:
	cargo test --workspace

build:
	stellar contract build

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

clean:
	cargo clean

# Usage: make deploy-testnet SOURCE=me
deploy-testnet: build
	stellar contract deploy \
	  --wasm target/wasm32v1-none/release/inherit_vault.wasm \
	  --source $(SOURCE) --network testnet
