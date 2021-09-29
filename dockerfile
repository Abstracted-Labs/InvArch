FROM ubuntu as builder
RUN apt-get update --fix-missing
RUN apt-get install -y git && apt-get install -y curl
RUN git clone -b main https://github.com/InvArch/InvArch-node
RUN apt-get install -y build-essential && \
    apt-get install -y clang && \
    apt-get install -y jq && \
	curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    export PATH="$PATH:$HOME/.cargo/bin" && \
    cd InvArch-node && \
    rustup toolchain install $(cat rust-toolchain) && \
    rustup default stable && \
    rustup target add wasm32-unknown-unknown --toolchain $(cat rust-toolchain) && \
    cargo $(cat rust-toolchain) build --release

# /\-Build Stage | Final Stage-\/

FROM docker.io/library/ubuntu:20.04
COPY --from=builder /InvArch-node/target/release/invarch-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /invarch-node invarch-node && \
        mkdir -p /invarch-node/.local/share && \
        mkdir /data && \
        chown -R invarch-node:invarch-node /data && \
        ln -s /data /invarch-node/.local/share/invarch-node && \
        rm -rf /usr/bin /usr/sbin

USER invarch-node
EXPOSE 30333 9933 9944
VOLUME ["/data"]

EXPOSE 30333 9933 9944

ENTRYPOINT ["/usr/local/bin/invarch-node"]
