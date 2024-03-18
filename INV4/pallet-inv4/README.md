# INV4 Pallet

## Introduction

The INV4 pallet is designed to manage advanced virtual multisigs, internally referred to as cores. It provides the functionality to create cores, mint and burn the core's voting tokens, and manage multisig proposals. This pallet is a comprehensive solution for decentralized decision-making processes, allowing for flexible and secure management of multisig operations.

## Features

- **Core Creation**: Establish new cores with customizable parameters, including metadata, voting thresholds, and token freeze state.
- **Token Management**: Mint and burn the core's voting tokens to manage the voting power within the core.
- **Multisig Proposals**: Create, vote on, and cancel multisig proposals. Proposals automatically execute if they meet the execution threshold requirements.
- **Vote Management**: Members can vote on proposals, withdraw their votes, and influence the outcome of decisions.
- **Parameter Adjustment**: Core parameters, such as voting thresholds and token freeze state, can be dynamically adjusted by core origins.

## Functionality Overview

### Core Management

- `create_core`: Initialize a new core with specific parameters and distribute initial voting tokens to the creator.
- `set_parameters`: Modify core parameters, including voting thresholds, metadata, and token freeze state.

### Token Operations

- `token_mint`: Mint the core's voting tokens to a specified target, increasing their voting power within the core.
- `token_burn`: Burn the core's voting tokens from a specified target, decreasing their voting power.

### Multisig Operations

- `operate_multisig`: Submit a new multisig proposal. If the proposal meets execution thresholds, it is automatically executed.
- `vote_multisig`: Cast a vote on an existing multisig proposal. Proposals execute automatically if they meet threshold requirements after the vote.
- `withdraw_vote_multisig`: Withdraw a previously cast vote from a multisig proposal.
- `cancel_multisig_proposal`: Cancel an existing multisig proposal. This action can only be performed by a core origin.

### Utility Functions

- `CoreAccountDerivation`: Derive consistent core AccountIds across parachains for seamless interaction.
- `INV4Lookup`: Custom account lookup implementation for converting CoreIds to AccountIds.
- `FeeAsset`: Define the asset used by the multisig for paying transaction fees.
- `MultisigFeeHandler`: Manage fee payments for multisig operations, supporting both native and non-native assets.

## Usage

To utilize the INV4 pallet, users must first create a core and receive initial voting tokens. Cores can propose actions, vote on proposals, and execute decisions based on the collective voting power of their members. The pallet's flexible design supports a wide range of multisig use cases, from simple governance decisions to complex, conditional executions.
