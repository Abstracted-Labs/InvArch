check:
	cargo check

build:
	cargo build --release

test:
	cargo test

run:
	./target/release/invarch-collator --dev --tmp

purge-alice:
	./target/release/invarch-node purge-chain --base-path /tmp/alice --chain local


run-alice:
	./target/release/invarch-node \
	--base-path /tmp/alice \
	--chain local \
	--alice \
	--port 30333 \
	--ws-port 9945 \
	--rpc-port 9933 \
	--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
	--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
	--validator

purge-bob:
	./target/release/invarch-node purge-chain --base-path /tmp/bob --chain local -y

run-bob:
	./target/release/invarch-node \
	--base-path /tmp/bob \
	--chain local \
	--bob \
	--port 30334 \
	--ws-port 9946 \
	--rpc-port 9934 \
	--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
	--validator \
	--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp

generate-keys:
	./target/release/invarch-collator key generate --scheme Sr25519 --password-interactive

generate-derive-keys:
	./target/release/invarch-collator key inspect --password-interactive --scheme Ed25519 0xd44687c2ae9c9767027fc2beaf1e7f952bd1f5f1d579430de564245ca2f6ddb8


build-spec-local-rococo-plain:
	./target/release/invarch-collator build-spec --disable-default-bootnode > node/testnet/rococo-local-parachain-plain.json

build-spec-local-rococo-raw:
	./target/release/invarch-collator build-spec --chain node/testnet/rococo-local-parachain-plain.json --raw --disable-default-bootnode > node/testnet/rococo-local-parachain-2000-raw.json

generate-wasm:
	./target/release/invarch-collator export-genesis-wasm --chain node/testnet/rococo-local-parachain-2000-raw.json > node/testnet/para-2000-wasm

generate-genesis:
	./target/release/invarch-collator export-genesis-state --chain node/testnet/rococo-local-parachain-2000-raw.json > node/testnet/para-2000-genesis

purge-first-node:
	./target/release/invarch-collator purge-chain --base-path /tmp/node01 --chain local -y

start-collator1:
	./target/release/invarch-collator \
	--alice \
	--collator \
	--force-authoring \
	--chain node/testnet/rococo-local-parachain-2000-raw.json \
	--base-path /tmp/parachain/alice \
	--port 40333 \
	--ws-port 8844 \
	-- \
	--execution wasm \
	--chain /home/kresna/invarch/polkadot/node/invarch/testnet/tinker/tinker-local-spec-raw.json \
	--port 30343 \
	--ws-port 9977
