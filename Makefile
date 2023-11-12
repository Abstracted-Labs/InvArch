build-tinkernet:
	cd tinkernet && cargo build --release

build-invarch:
	cd invarch && cargo build --release

bindir = zombienet/binaries
dir_target = $(bindir)-$(wildcard $(bindir))
dir_present = $(bindir)-$(bindir)
dir_absent = $(bindir)-

polkadot_target = $(bindir)/polkadot-$(wildcard $(bindir)/polkadot)
polkadot_present = $(bindir)/polkadot-$(bindir)/polkadot
polkadot_absent = $(bindir)/polkadot-

basilisk_target = $(bindir)/basilisk-$(wildcard $(bindir)/basilisk)
basilisk_present = $(bindir)/basilisk-$(bindir)/basilisk
basilisk_absent = $(bindir)/basilisk-

$(dir_present):
$(polkadot_present):
$(basilisk_present):

$(dir_absent): | zombienet-create-binaries-dir

$(polkadot_absent): | zombienet-download-polkadot

$(basilisk_absent): | zombienet-download-basilisk

zombienet-create-binaries-dir:
	mkdir zombienet/binaries

zombienet-download-polkadot: | $(dir_target)
	wget -O zombienet/binaries/polkadot "https://github.com/paritytech/polkadot/releases/latest/download/polkadot"
	chmod +x zombienet/binaries/polkadot

zombienet-download-basilisk: | $(dir_target)
	wget -O zombienet/binaries/basilisk "https://github.com/galacticcouncil/Basilisk-node/releases/download/v10.3.0/basilisk"
	chmod +x zombienet/binaries/basilisk

zombienet-run-tinkernet+basilisk: | $(polkadot_target) $(basilisk_target)
	zombienet spawn zombienet/rococo-and-tinkernet+basilisk.toml

zombienet-run-tinkernet+tinkernet: | $(polkadot_target)
	zombienet spawn zombienet/rococo-and-tinkernet+tinkernet.toml

zombienet-run-kusama+tinkernet:
	zombienet spawn zombienet/kusama-and-tinkernet.toml

run-tinkernet-solo-alice:
	cd tinkernet && ./target/release/tinkernet-collator --chain solo-dev --alice --tmp --listen-addr /ip4/0.0.0.0/tcp/53102/ws --rpc-cors=all --discover-local --collator --node-key c12b6d18942f5ee8528c8e2baf4e147b5c5c18710926ea492d09cbd9f6c9f82a

run-tinkernet-solo-bob:
	cd tinkernet && ./target/release/tinkernet-collator --chain solo-dev --bob --tmp --listen-addr /ip4/0.0.0.0/tcp/54102/ws --rpc-cors=all --discover-local --collator --bootnodes /ip4/127.0.0.1/tcp/53102/ws/p2p/12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2

run-tinkernet-solo: ; printf "run-tinkernet-solo-alice\nrun-tinkernet-solo-bob" | parallel -u make

run-invarch-solo-alice:
	cd invarch && ./target/release/invarch-collator --chain solo-dev --alice --tmp --listen-addr /ip4/0.0.0.0/tcp/53102/ws --rpc-cors=all --discover-local --collator --node-key c12b6d18942f5ee8528c8e2baf4e147b5c5c18710926ea492d09cbd9f6c9f82a

run-invarch-solo-bob:
	cd invarch && ./target/release/invarch-collator --chain solo-dev --bob --tmp --listen-addr /ip4/0.0.0.0/tcp/54102/ws --rpc-cors=all --discover-local --collator --bootnodes /ip4/127.0.0.1/tcp/53102/ws/p2p/12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2

run-invarch-solo: ; printf "run-invarch-solo-alice\nrun-invarch-solo-bob" | parallel -u make
