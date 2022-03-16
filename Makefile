check:
	cargo check

build:
	cargo build --release

test:
	cargo test

run:
	./target/release/invarch-collator --dev

generate-keys:
	./target/release/invarch-collator key generate --scheme Sr25519 --password-interactive

# generate-derive-keys:
# 	./target/release/invarch-collator key inspect --password-interactive --scheme Ed25519 0xd44687c2ae9c9767027fc2beaf1e7f952bd1f5f1d579430de564245ca2f6ddb8

genesis-state:
	./target/release/invarch-collator export-genesis-state > node/testnet/genesis-state

genesis-wasm:
	./target/release/invarch-collator export-genesis-wasm > node/testnet/genesis-wasm

purge-first-node:
	./target/release/invarch-collator purge-chain --base-path /tmp/node01 --chain local -y

start-collator1:
	./target/release/invarch-collator \
	--collator \
	--alice \
	--force-authoring \
	--tmp \
	--port 40335 \
	--ws-port 9946 \
	-- \
	--execution wasm \
	--chain <relative path local rococo json file> \
	--port 30335 \

start-collator2:
	./target/release/invarch-collator \
	--collator \
	--bob \
	--force-authoring \
	--tmp \
	--port 40336 \
	--ws-port 9947 \
	-- \
	--execution wasm \
	--chain <relative path local rococo json file> \
	--port 30336 \

start-parachain-full-node:
	./target/release/invarch-collator \
	--tmp \
	--port 40337 \
	--ws-port 9948 \
	-- \
	--execution wasm \
	--chain <relative path local rococo json file> \
	--port 30337 \