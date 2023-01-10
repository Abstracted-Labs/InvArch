use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::traits::Currency;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Zero},
    RuntimeDebug,
};
use sp_std::{ops::Add, prelude::*};

pub use crate::pallet::*;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const MAX_ASSUMED_VEC_LEN: u32 = 10;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CoreMetadata<Name, Description, Image> {
    pub name: Name,
    pub description: Description,
    pub image: Image,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CoreInfo<AccountId, Metadata> {
    pub account: AccountId,
    pub metadata: Metadata,
}

#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RewardInfo<Balance: HasCompact + MaxEncodedLen> {
    #[codec(compact)]
    pub(crate) stakers: Balance,
    #[codec(compact)]
    pub(crate) core: Balance,
}

#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EraInfo<Balance: HasCompact + MaxEncodedLen> {
    pub(crate) rewards: RewardInfo<Balance>,
    #[codec(compact)]
    pub(crate) staked: Balance,
    #[codec(compact)]
    pub(crate) active_stake: Balance,
    #[codec(compact)]
    pub(crate) locked: Balance,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CoreStakeInfo<Balance: HasCompact + MaxEncodedLen> {
    #[codec(compact)]
    pub(crate) total: Balance,
    #[codec(compact)]
    pub(crate) number_of_stakers: u32,
    pub(crate) reward_claimed: bool,
    pub(crate) active: bool,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub(crate) struct EraStake<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> {
    #[codec(compact)]
    pub(crate) staked: Balance,
    #[codec(compact)]
    pub(crate) era: Era,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct StakerInfo<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> {
    pub(crate) stakes: Vec<EraStake<Balance>>,
}

impl<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> MaxEncodedLen for StakerInfo<Balance> {
    fn max_encoded_len() -> usize {
        codec::Compact(MAX_ASSUMED_VEC_LEN)
            .encoded_size()
            .saturating_add(
                (MAX_ASSUMED_VEC_LEN as usize)
                    .saturating_mul(EraStake::<Balance>::max_encoded_len()),
            )
    }
}

impl<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> StakerInfo<Balance> {
    pub(crate) fn is_empty(&self) -> bool {
        self.stakes.is_empty()
    }

    pub(crate) fn len(&self) -> u32 {
        self.stakes.len() as u32
    }

    pub(crate) fn stake(&mut self, current_era: Era, value: Balance) -> Result<(), &str> {
        if let Some(era_stake) = self.stakes.last_mut() {
            if era_stake.era > current_era {
                return Err("Unexpected era");
            }

            let new_stake_value = era_stake.staked.saturating_add(value);

            if current_era == era_stake.era {
                *era_stake = EraStake {
                    staked: new_stake_value,
                    era: current_era,
                }
            } else {
                self.stakes.push(EraStake {
                    staked: new_stake_value,
                    era: current_era,
                })
            }
        } else {
            self.stakes.push(EraStake {
                staked: value,
                era: current_era,
            });
        }

        Ok(())
    }

    pub(crate) fn unstake(&mut self, current_era: Era, value: Balance) -> Result<(), &str> {
        if let Some(era_stake) = self.stakes.last_mut() {
            if era_stake.era > current_era {
                return Err("Unexpected era");
            }

            let new_stake_value = era_stake.staked.saturating_sub(value);
            if current_era == era_stake.era {
                *era_stake = EraStake {
                    staked: new_stake_value,
                    era: current_era,
                }
            } else {
                self.stakes.push(EraStake {
                    staked: new_stake_value,
                    era: current_era,
                })
            }

            if !self.stakes.is_empty() && self.stakes[0].staked.is_zero() {
                self.stakes.remove(0);
            }
        }

        Ok(())
    }

    pub(crate) fn claim(&mut self) -> (Era, Balance) {
        if let Some(era_stake) = self.stakes.first() {
            let era_stake = *era_stake;

            if self.stakes.len() == 1 || self.stakes[1].era > era_stake.era + 1 {
                self.stakes[0] = EraStake {
                    staked: era_stake.staked,
                    era: era_stake.era.saturating_add(1),
                }
            } else {
                self.stakes.remove(0);
            }

            if !self.stakes.is_empty() && self.stakes[0].staked.is_zero() {
                self.stakes.remove(0);
            }

            (era_stake.era, era_stake.staked)
        } else {
            (0, Zero::zero())
        }
    }

    pub(crate) fn latest_staked_value(&self) -> Balance {
        self.stakes.last().map_or(Zero::zero(), |x| x.staked)
    }
}

#[derive(
    Clone, PartialEq, Eq, Copy, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub(crate) struct UnlockingChunk<Balance: MaxEncodedLen> {
    #[codec(compact)]
    pub(crate) amount: Balance,
    #[codec(compact)]
    pub(crate) unlock_era: Era,
}

impl<Balance> UnlockingChunk<Balance>
where
    Balance: Add<Output = Balance> + Copy + MaxEncodedLen,
{
    pub(crate) fn add_amount(&mut self, amount: Balance) {
        self.amount = self.amount + amount
    }
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub(crate) struct UnbondingInfo<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> {
    pub(crate) unlocking_chunks: Vec<UnlockingChunk<Balance>>,
}

impl<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> MaxEncodedLen
    for UnbondingInfo<Balance>
{
    fn max_encoded_len() -> usize {
        codec::Compact(MAX_ASSUMED_VEC_LEN)
            .encoded_size()
            .saturating_add(
                (MAX_ASSUMED_VEC_LEN as usize)
                    .saturating_mul(UnlockingChunk::<Balance>::max_encoded_len()),
            )
    }
}

impl<Balance> UnbondingInfo<Balance>
where
    Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen,
{
    pub(crate) fn len(&self) -> u32 {
        self.unlocking_chunks.len() as u32
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.unlocking_chunks.is_empty()
    }

    pub(crate) fn sum(&self) -> Balance {
        self.unlocking_chunks
            .iter()
            .map(|chunk| chunk.amount)
            .reduce(|c1, c2| c1 + c2)
            .unwrap_or_default()
    }

    pub(crate) fn add(&mut self, chunk: UnlockingChunk<Balance>) {
        match self
            .unlocking_chunks
            .binary_search_by(|x| x.unlock_era.cmp(&chunk.unlock_era))
        {
            Ok(pos) => self.unlocking_chunks[pos].add_amount(chunk.amount),
            Err(pos) => self.unlocking_chunks.insert(pos, chunk),
        }
    }

    pub(crate) fn partition(self, era: Era) -> (Self, Self) {
        let (matching_chunks, other_chunks): (
            Vec<UnlockingChunk<Balance>>,
            Vec<UnlockingChunk<Balance>>,
        ) = self
            .unlocking_chunks
            .iter()
            .partition(|chunk| chunk.unlock_era <= era);

        (
            Self {
                unlocking_chunks: matching_chunks,
            },
            Self {
                unlocking_chunks: other_chunks,
            },
        )
    }
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AccountLedger<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> {
    #[codec(compact)]
    pub(crate) locked: Balance,
    pub(crate) unbonding_info: UnbondingInfo<Balance>,
}

impl<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> AccountLedger<Balance> {
    pub(crate) fn is_empty(&self) -> bool {
        self.locked.is_zero() && self.unbonding_info.is_empty()
    }
}
