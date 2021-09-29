FROM rust:1.52-slim as builder
RUN apt-get update --fix-missing
RUN apt-get install -y git && apt-get install -y curl
RUN git clone https://github.com/InvArch/InvArch-node
RUN cd InvArch-node && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    	
        export PATH="$PATH:$HOME/.cargo/bin" && \
        rustup toolchain install $(cat rust-toolchain) && \
        rustup target add wasm32-unknown-unknown --toolchain $(cat rust-toolchain) && \
        CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --release

# /\-Build Stage | Final Stage-\/

FROM docker.io/library/ubuntu:20.04
COPY --from=builder /InvArch-node/target/release/invarch-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /invarch-node invarch-node && \
        mkdir -p /invarch-node/.local/share && \
        mkdir /data && \
        chown -R invarch-node:polkadex-node /data && \
        ln -s /data /invarch-node/.local/share/invarch-node && \
        rm -rf /usr/bin /usr/sbin

USER invarch-node
EXPOSE 30333 9933 9944
VOLUME ["/data"]

EXPOSE 30333 9933 9944

ENTRYPOINT ["/usr/local/bin/invarch-node"]
