# OCIF Staking Pallet

## Overview

The OCIF Staking Pallet is a pallet designed to facilitate staking towards INV-Cores within a blockchain network. This pallet introduces a staking mechanism that allows two distinct sets of entities, namely Cores and Stakers, to participate in the distribution of tokens from a predefined pot. The allocation of rewards is determined based on the amount staked by each entity and the total stake towards each Core.

### Cores

Cores represent virtual accounts identified by unique IDs, which are responsible for registering themselves within the staking ecosystem. The primary role of Cores is to attract Stakers to lock tokens in their favor. The rewards allocated to Cores are proportional to the total amount staked towards them by Stakers. However, for a Core to be eligible for rewards, it must have a total stake above a predefined threshold, thereby becoming `active`.

### Stakers

Stakers are individual accounts that engage in locking tokens in favor of a Core. Unlike Cores, Stakers receive a fraction of the rewards based on their own stake.

## Runtime Configuration Parameters

- `BlocksPerEra`: Defines the duration of an era in terms of block numbers.
- `RegisterDeposit`: Specifies the deposit amount required for Core registration.
- `MaxStakersPerCore`: Limits the maximum number of Stakers that can simultaneously stake towards a single Core.
- `MinimumStakingAmount`: Sets the minimum amount required for a Staker to participate in staking.
- `UnbondingPeriod`: Determines the period, in blocks, required for unbonding staked tokens.
- `RewardRatio`: Establishes the distribution ratio of rewards between Cores and Stakers.
- `StakeThresholdForActiveCore`: Sets the stake threshold required for a Core to become `active`.

## Dispatchable Functions

- `register_core`: Allows Cores to register themselves in the system.
- `unregister_core`: Enables Cores to unregister from the system, initiating the unbonding period for Stakers.
- `change_core_metadata`: Facilitates changes to the metadata associated with a Core.
- `stake`: Allows Stakers to lock tokens in favor of a Core.
- `unstake`: Permits Stakers to unlock tokens previously staked on a Core, starting the unbonding period.
- `withdraw_unstaked`: Enables Stakers to withdraw tokens that have completed the unbonding period.
- `staker_claim_rewards`: Allows Stakers to claim available rewards.
- `core_claim_rewards`: Enables Cores to claim available rewards.
- `halt_unhalt_pallet`: Grants Root permissions to halt or resume the staking operations within the pallet.

## Events

The pallet emits events such as `Staked`, `Unstaked`, `CoreRegistered`, `CoreUnregistered`, and others to signal various operations and state changes within the staking ecosystem.

## Errors

Errors such as `StakingNothing`, `InsufficientBalance`, `MaxStakersReached`, and others are defined to handle exceptional scenarios encountered during pallet operations.

## Example Runtime Implementation

For an example runtime implementation that integrates this pallet, refer to [src/testing/mock.rs](./src/testing/mock.rs).
