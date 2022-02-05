[![Compatible with Substrate v3.0.0](https://img.shields.io/badge/Substrate-v3.0.0-E6007A)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)

# IPVM Pallet: Intellectual Property Virtual Machine Pallet

This is a Substrate [Pallet](https://substrate.dev/docs/en/knowledgebase/runtime/pallets) that defines the cross-chain state machine that natively execute both EVM-bytecode or WASM binaries, depending on the format of the function source. The purpose for such an environment to exist is to provide a distributed IP directory across multiple protocols.