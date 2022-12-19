# pallet-ocif-staking

## OCIF Staking pallet
A pallet for for allowing INV-Cores to be staked towards.


### Overview

This pallet provides functionality to allow 2 sets of entities to participate in distribution of tokens
available in a predefined pot account.
The tokens provided to the pot account are to be handled by the Runtime,
either directly or with the assistance of another pallet.

The 2 entity sets will be referred to in code as Cores and Stakers:

#### Cores
Cores are virtual accounts that have an ID used to derive their own account address,
their task in the process is to register themselves and have Stakers lock tokens in favor of a specifc Core.
Cores receive their fraction of the pot rewards based on the total amount staked towards them by Stakers,
however, a Core must have total stake above the defined threshold (making it `active`), otherwise they won't be entitled to rewards.

#### Stakers
Stakers are any account existing on the chain, their task is to lock tokens in favor of a Core.
Unlike Cores, Stakers get their fraction of the rewards based on their own stake and regardless of
the `active` state of the Core they staked towards.


### Relevant runtime configs

* `BlocksPerEra` - Defines how many blocks constitute an era.
* `RegisterDeposit` - Defines the deposit amount for a Core to register in the system.
* `MaxStakersPerCore` - Defines the maximum amount of Stakers allowed to stake simultaneously towards the same Core.
* `MinimumStakingAmount` - Defines the minimum amount a Staker has to stake to participate.
* `UnbondingPeriod` - Defines the period, in blocks, that it takes to unbond a stake.
* `RewardRatio` - Defines the ratio of balance from the pot to distribute to Cores and Stakers, respectively.
* `StakeThresholdForActiveCore` - Defines the threshold of stake a Core needs to surpass to become active.

**Example Runtime implementation can be found in [src/testing/mock.rs](./src/testing/mock.rs)**

### Dispatchable Functions

* `register_core` - Registers a Core in the system.
* `unregister_core` - Unregisters a Core from the system, starting the unbonding period for the Stakers.
* `change_core_metadata` - Changes the metadata tied to a Core.
* `stake` - Stakes tokens towards a Core.
* `untake` - Unstakes tokens from a core and starts the unbonding period for those tokens.
* `withdraw_unstaked` - Withdraws tokens that have already been through the unbonding period.
* `staker_claim_rewards` - Claims rewards available for a Staker.
* `core_claim_rewards` - Claims rewards available for a Core.
* `halt_unhalt_pallet` - Allows Root to trigger a halt of the system, eras will stop counting and rewards won't be distributed.

[`Call`]: ./enum.Call.html
[`Config`]: ./trait.Config.html

License: GPLv3
