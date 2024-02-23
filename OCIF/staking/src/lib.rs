//! # OCIF Staking pallet
//! A pallet for for allowing INV-Cores to be staked towards.
//!
//!
//! ## Overview
//!
//! This pallet provides functionality to allow 2 sets of entities to participate in distribution of tokens
//! available in a predefined pot account.
//! The tokens provided to the pot account are to be handled by the Runtime,
//! either directly or with the assistance of another pallet.
//!
//! The 2 entity sets will be referred to in code as Cores and Stakers:
//!
//! ### Cores
//! Cores are virtual accounts that have an ID used to derive their own account address,
//! their task in the process is to register themselves and have Stakers lock tokens in favor of a specifc Core.
//! Cores receive their fraction of the pot rewards based on the total amount staked towards them by Stakers,
//! however, a Core must have total stake above the defined threshold (making it `active`), otherwise they won't be entitled to rewards.
//!
//! ### Stakers
//! Stakers are any account existing on the chain, their task is to lock tokens in favor of a Core.
//! Unlike Cores, Stakers get their fraction of the rewards based on their own stake and regardless of
//! the `active` state of the Core they staked towards.
//!
//!
//! ## Relevant runtime configs
//!
//! * `BlocksPerEra` - Defines how many blocks constitute an era.
//! * `RegisterDeposit` - Defines the deposit amount for a Core to register in the system.
//! * `MaxStakersPerCore` - Defines the maximum amount of Stakers allowed to stake simultaneously towards the same Core.
//! * `MinimumStakingAmount` - Defines the minimum amount a Staker has to stake to participate.
//! * `UnbondingPeriod` - Defines the period, in blocks, that it takes to unbond a stake.
//! * `RewardRatio` - Defines the ratio of balance from the pot to distribute to Cores and Stakers, respectively.
//! * `StakeThresholdForActiveCore` - Defines the threshold of stake a Core needs to surpass to become active.
//!
//! **Example Runtime implementation can be found in [src/testing/mock.rs](./src/testing/mock.rs)**
//!
//! ## Dispatchable Functions
//!
//! * `register_core` - Registers a Core in the system.
//! * `unregister_core` - Unregisters a Core from the system, starting the unbonding period for the Stakers.
//! * `change_core_metadata` - Changes the metadata tied to a Core.
//! * `stake` - Stakes tokens towards a Core.
//! * `untake` - Unstakes tokens from a core and starts the unbonding period for those tokens.
//! * `withdraw_unstaked` - Withdraws tokens that have already been through the unbonding period.
//! * `staker_claim_rewards` - Claims rewards available for a Staker.
//! * `core_claim_rewards` - Claims rewards available for a Core.
//! * `halt_unhalt_pallet` - Allows Root to trigger a halt of the system, eras will stop counting and rewards won't be distributed.
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::{Pays, PostDispatchInfo},
    ensure,
    pallet_prelude::*,
    traits::{
        Currency, ExistenceRequirement, Get, Imbalance, LockIdentifier, LockableCurrency,
        ReservableCurrency, WithdrawReasons,
    },
    weights::Weight,
    PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::{
    traits::{AccountIdConversion, Saturating, Zero},
    Perbill,
};
use sp_std::{
    convert::{From, TryInto},
    vec::Vec,
};

pub mod primitives;
use core::ops::Div;
use primitives::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod testing;
pub mod weights;

pub use weights::WeightInfo;

/// Staking lock dentifier.
const LOCK_ID: LockIdentifier = *b"ocif-stk";

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use pallet_inv4::{
        origin::{ensure_multisig, INV4Origin},
        CoreAccountDerivation,
    };

    use super::*;

    /// The balance type of this pallet.
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// The opaque token type for an imbalance. This is returned by unbalanced operations and must be dealt with.
    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    /// The core metadata type of this pallet.
    pub type CoreMetadataOf<T> = CoreMetadata<
        BoundedVec<u8, <T as Config>::MaxNameLength>,
        BoundedVec<u8, <T as Config>::MaxDescriptionLength>,
        BoundedVec<u8, <T as Config>::MaxImageUrlLength>,
    >;

    /// The core information type, containing a core's AccountId and CoreMetadataOf.
    pub type CoreInfoOf<T> = CoreInfo<<T as frame_system::Config>::AccountId, CoreMetadataOf<T>>;

    /// Counter for the number of eras that have passed.
    pub type Era = u32;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_inv4::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The staking balance.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
            + ReservableCurrency<Self::AccountId>;

        // type CoreId: Parameter
        //     + Member
        //     + AtLeast32BitUnsigned
        //     + Default
        //     + Copy
        //     + Display
        //     + MaxEncodedLen
        //     + Clone
        //     + From<<Self as pallet_inv4::Config>::CoreId>;

        /// Number of blocks per era.
        #[pallet::constant]
        type BlocksPerEra: Get<BlockNumberFor<Self>>;

        /// Deposit amount that will be reserved as part of new core registration.
        #[pallet::constant]
        type RegisterDeposit: Get<BalanceOf<Self>>;

        /// Maximum number of unique stakers per core.
        #[pallet::constant]
        type MaxStakersPerCore: Get<u32>;

        /// Minimum amount user must have staked on a core.
        /// User can stake less if they already have the minimum staking amount staked on that particular core.
        #[pallet::constant]
        type MinimumStakingAmount: Get<BalanceOf<Self>>;

        /// Account Identifier from which the internal Pot is generated.
        #[pallet::constant]
        type PotId: Get<PalletId>;

        /// The minimum amount required to keep an account open.
        #[pallet::constant]
        type ExistentialDeposit: Get<BalanceOf<Self>>;

        /// Max number of unlocking chunks per account Id <-> core Id pairing.
        /// If value is zero, unlocking becomes impossible.
        #[pallet::constant]
        type MaxUnlocking: Get<u32>;

        /// Number of blocks that need to pass until unstaked value can be withdrawn.
        /// When set to `0`, it's equal to having no unbonding period.
        #[pallet::constant]
        type UnbondingPeriod: Get<u32>;

        /// Max number of unique `EraStake` values that can exist for a `(staker, core)` pairing.
        /// When stakers claims rewards, they will either keep the number of `EraStake` values the same or they will reduce them by one.
        /// Stakers cannot add an additional `EraStake` value by calling `bond&stake` or `unbond&unstake` if they've reached the max number of values.
        ///
        /// This ensures that history doesn't grow indefinitely - if there are too many chunks, stakers should first claim their former rewards
        /// before adding additional `EraStake` values.
        #[pallet::constant]
        type MaxEraStakeValues: Get<u32>;

        /// Reward ratio of the pot to be distributed between the core and stakers, respectively.
        #[pallet::constant]
        type RewardRatio: Get<(u32, u32)>;

        /// Threshold of staked tokens necessary for the core to become active.
        #[pallet::constant]
        type StakeThresholdForActiveCore: Get<BalanceOf<Self>>;

        /// Maximum lenght of a core's name.
        #[pallet::constant]
        type MaxNameLength: Get<u32>;

        /// Maximum lenght of a core's description.
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;

        /// Maximum lenght of a core's image url.
        #[pallet::constant]
        type MaxImageUrlLength: Get<u32>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// General information about the staker.
    #[pallet::storage]
    #[pallet::getter(fn ledger)]
    pub type Ledger<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AccountLedger<BalanceOf<T>>, ValueQuery>;

    /// The current era index.
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, Era, ValueQuery>;

    /// Accumulator for block rewards during an era. It is reset at every new era
    #[pallet::storage]
    #[pallet::getter(fn reward_accumulator)]
    pub type RewardAccumulator<T> = StorageValue<_, RewardInfo<BalanceOf<T>>, ValueQuery>;

    /// Stores the block number of when the next era starts.
    #[pallet::storage]
    #[pallet::getter(fn next_era_starting_block)]
    pub type NextEraStartingBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// Simple map where CoreId points to the respective core information.
    #[pallet::storage]
    #[pallet::getter(fn core_info)]
    pub(crate) type RegisteredCore<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CoreId, CoreInfoOf<T>>;

    /// General information about an era.
    #[pallet::storage]
    #[pallet::getter(fn general_era_info)]
    pub type GeneralEraInfo<T: Config> = StorageMap<_, Twox64Concat, Era, EraInfo<BalanceOf<T>>>;

    /// Staking information about a core in a particular era.
    #[pallet::storage]
    #[pallet::getter(fn core_stake_info)]
    pub type CoreEraStake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CoreId,
        Twox64Concat,
        Era,
        CoreStakeInfo<BalanceOf<T>>,
    >;

    /// Info about stakers stakes on particular core.
    #[pallet::storage]
    #[pallet::getter(fn staker_info)]
    pub type GeneralStakerInfo<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CoreId,
        Blake2_128Concat,
        T::AccountId,
        StakerInfo<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Denotes whether pallet is halted(disabled).
    #[pallet::storage]
    #[pallet::getter(fn is_halted)]
    pub type Halted<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Account has staked funds on a core.
        Staked {
            staker: T::AccountId,
            core: T::CoreId,
            amount: BalanceOf<T>,
        },

        /// Account has unstaked funds on a core.
        Unstaked {
            staker: T::AccountId,
            core: T::CoreId,
            amount: BalanceOf<T>,
        },

        /// Account has withdrawn unbonded funds.
        Withdrawn {
            staker: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// New core added for staking.
        CoreRegistered { core: T::CoreId },

        /// Core removed.
        CoreUnregistered { core: T::CoreId },

        /// Beginning of a new era.
        NewEra { era: u32 },

        /// Staker claimed staking rewards.
        StakerClaimed {
            staker: T::AccountId,
            core: T::CoreId,
            era: u32,
            amount: BalanceOf<T>,
        },

        /// Core claimed staking rewards.
        CoreClaimed {
            core: T::CoreId,
            destination_account: T::AccountId,
            era: u32,
            amount: BalanceOf<T>,
        },

        /// Halted status changed.
        HaltChanged { is_halted: bool },

        MetadataChanged {
            core: T::CoreId,
            old_metadata: CoreMetadata<Vec<u8>, Vec<u8>, Vec<u8>>,
            new_metadata: CoreMetadata<Vec<u8>, Vec<u8>, Vec<u8>>,
        },

        /// Staker moved his stake.
        StakeMoved {
            staker: T::AccountId,
            from_core: T::CoreId,
            to_core: T::CoreId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Staking nothing.
        StakingNothing,
        /// Staker tried to stake less than the minimum staking amount.
        InsufficientBalance,
        /// Maximum number of stakers reached.
        MaxStakersReached,
        /// Core not found.
        CoreNotFound,
        /// No stake available for withdrawal.
        NoStakeAvailable,
        /// Core is not unregistered.
        NotUnregisteredCore,
        /// Unclaimed rewards available.
        UnclaimedRewardsAvailable,
        /// Unstaking nothing.
        UnstakingNothing,
        /// Nothing available for withdrawal.
        NothingToWithdraw,
        /// Core already registered.
        CoreAlreadyRegistered,
        /// Unknown rewards for era.
        UnknownEraReward,
        /// Unexpected stake info for era.
        UnexpectedStakeInfoEra,
        /// Too many unlocking chunks.
        TooManyUnlockingChunks,
        /// Reward already claimed.
        RewardAlreadyClaimed,
        /// Incorrect era.
        IncorrectEra,
        /// Too many era stake values.
        TooManyEraStakeValues,
        /// Not a staker.
        NotAStaker,
        /// No permission.
        NoPermission,
        /// Name exceeds maximum length.
        MaxNameExceeded,
        /// Description exceeds maximum length.
        MaxDescriptionExceeded,
        /// Image url exceeds maximum length.
        MaxImageExceeded,
        /// Core not registered.
        NotRegistered,
        /// Halted.
        Halted,
        /// No halt change.
        NoHaltChange,
        /// Staker moved staking to the same core.
        MoveStakeToSameCore,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            // As long as pallet is disabled, we shouldn't allow any storage modifications.
            // This means we might prolong an era but it's acceptable.
            // Runtime upgrade should be timed so we ensure that we complete it before
            // a new era is triggered. This code is just a safety net to ensure nothing is broken
            // if we fail to do that.
            if Self::is_halted() {
                return T::DbWeight::get().reads(1);
            }

            let previous_era = Self::current_era();
            let next_era_starting_block = Self::next_era_starting_block();

            // Value is compared to 1 since genesis block is ignored
            if now >= next_era_starting_block || previous_era.is_zero() {
                let blocks_per_era = T::BlocksPerEra::get();
                let next_era = previous_era + 1;
                CurrentEra::<T>::put(next_era);

                NextEraStartingBlock::<T>::put(now + blocks_per_era);

                let reward = RewardAccumulator::<T>::take();
                let (consumed_weight, new_active_stake) = Self::rotate_staking_info(previous_era);
                Self::reward_balance_snapshot(previous_era, reward, new_active_stake);

                Self::deposit_event(Event::<T>::NewEra { era: next_era });

                consumed_weight + T::DbWeight::get().reads_writes(5, 3)
            } else {
                T::DbWeight::get().reads(3)
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        Result<INV4Origin<T>, <T as frame_system::Config>::RuntimeOrigin>:
            From<<T as frame_system::Config>::RuntimeOrigin>,
        T::AccountId: From<[u8; 32]>,
    {
        /// Used to register core for staking.
        /// The origin has to be from the multisig of the core.
        ///
        /// Depending on the pallet configuration/state it is possible that developer needs to be whitelisted prior to registration.
        ///
        /// As part of this call, `RegisterDeposit` will be reserved from the core account.
        ///
        /// - `name`: Name of the core.
        /// - `description`: Description of the core.
        /// - `image`: Image url of the core.
        #[pallet::call_index(0)]
        #[pallet::weight(
            <T as Config>::WeightInfo::register_core(
                name.len() as u32,
                description.len() as u32,
                image.len() as u32
            )
        )]
        pub fn register_core(
            origin: OriginFor<T>,
            name: BoundedVec<u8, T::MaxNameLength>,
            description: BoundedVec<u8, T::MaxDescriptionLength>,
            image: BoundedVec<u8, T::MaxImageUrlLength>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let core = ensure_multisig::<T, OriginFor<T>>(origin)?;
            let core_account = core.to_account_id();
            let core_id = core.id;

            ensure!(
                !RegisteredCore::<T>::contains_key(core_id),
                Error::<T>::CoreAlreadyRegistered,
            );

            let metadata: CoreMetadataOf<T> = CoreMetadata {
                name,
                description,
                image,
            };

            <T as pallet::Config>::Currency::reserve(&core_account, T::RegisterDeposit::get())?;

            RegisteredCore::<T>::insert(
                core_id,
                CoreInfo {
                    account: core_account,
                    metadata,
                },
            );

            Self::deposit_event(Event::<T>::CoreRegistered { core: core_id });

            Ok(PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::No,
            })
        }

        /// Unregister existing core for staking, making it ineligible for rewards from current era onwards and
        /// starts the unbonding period for the stakers.
        ///
        /// The origin has to be from the multisig of the core.
        ///
        /// Deposit is returned to the core account.
        ///
        /// - `core_id`: Id of the core to be unregistered.
        #[pallet::call_index(1)]
        #[pallet::weight(
            <T as Config>::WeightInfo::unregister_core() +
                <T as Config>::MaxStakersPerCore::get().div(100) * <T as Config>::WeightInfo::unstake()
        )]
        pub fn unregister_core(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let core = ensure_multisig::<T, OriginFor<T>>(origin)?;
            let core_account = core.to_account_id();
            let core_id = core.id;

            ensure!(
                RegisteredCore::<T>::get(core_id).is_some(),
                Error::<T>::NotRegistered
            );

            let current_era = Self::current_era();

            let staker_info_prefix = GeneralStakerInfo::<T>::iter_key_prefix(core_id);

            let mut corrected_staker_length_fee = Zero::zero();

            for staker in staker_info_prefix {
                let mut core_stake_info =
                    Self::core_stake_info(core_id, current_era).unwrap_or_default();

                let mut staker_info = Self::staker_info(core_id, &staker);

                let latest_staked_value = staker_info.latest_staked_value();

                let value_to_unstake = Self::internal_unstake(
                    &mut staker_info,
                    &mut core_stake_info,
                    latest_staked_value,
                    current_era,
                )?;

                let mut ledger = Self::ledger(&staker);
                ledger.unbonding_info.add(UnlockingChunk {
                    amount: value_to_unstake,
                    unlock_era: current_era + T::UnbondingPeriod::get(),
                });

                ensure!(
                    ledger.unbonding_info.len() <= T::MaxUnlocking::get(),
                    Error::<T>::TooManyUnlockingChunks
                );

                Self::update_ledger(&staker, ledger);

                GeneralEraInfo::<T>::mutate(current_era, |value| {
                    if let Some(x) = value {
                        x.staked = x.staked.saturating_sub(value_to_unstake);
                    }
                });
                Self::update_staker_info(&staker, core_id, staker_info);
                CoreEraStake::<T>::insert(core_id, current_era, core_stake_info);

                Self::deposit_event(Event::<T>::Unstaked {
                    staker,
                    core: core_id,
                    amount: value_to_unstake,
                });

                corrected_staker_length_fee += <T as Config>::WeightInfo::unstake();
            }

            RegisteredCore::<T>::remove(core_id);

            <T as pallet::Config>::Currency::unreserve(&core_account, T::RegisterDeposit::get());

            Self::deposit_event(Event::<T>::CoreUnregistered { core: core_id });

            Ok(
                Some(<T as Config>::WeightInfo::unregister_core() + corrected_staker_length_fee)
                    .into(),
            )
        }

        /// Used to change the metadata of a core.
        ///
        /// - `name`: Name of the core.
        /// - `description`: Description of the core.
        /// - `image`: Image url of the core.
        #[pallet::call_index(2)]
        #[pallet::weight(
            <T as Config>::WeightInfo::change_core_metadata(
                name.len() as u32,
                description.len() as u32,
                image.len() as u32
            )
        )]
        pub fn change_core_metadata(
            origin: OriginFor<T>,
            name: BoundedVec<u8, T::MaxNameLength>,
            description: BoundedVec<u8, T::MaxDescriptionLength>,
            image: BoundedVec<u8, T::MaxImageUrlLength>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let core_origin = ensure_multisig::<T, OriginFor<T>>(origin)?;
            let core_id = core_origin.id;

            RegisteredCore::<T>::try_mutate(core_id, |core| {
                let mut new_core = core.take().ok_or(Error::<T>::NotRegistered)?;

                let new_metadata: CoreMetadataOf<T> = CoreMetadata {
                    name: name.clone(),
                    description: description.clone(),
                    image: image.clone(),
                };

                let old_metadata = new_core.metadata;

                new_core.metadata = new_metadata;

                *core = Some(new_core);

                Self::deposit_event(Event::<T>::MetadataChanged {
                    core: core_id,
                    old_metadata: CoreMetadata {
                        name: old_metadata.name.into_inner(),
                        description: old_metadata.description.into_inner(),
                        image: old_metadata.image.into_inner(),
                    },
                    new_metadata: CoreMetadata {
                        name: name.to_vec(),
                        description: description.to_vec(),
                        image: image.to_vec(),
                    },
                });

                Ok(().into())
            })
        }

        /// Lock up and stake balance of the origin account.
        ///
        /// `value` must be more than the `minimum_stake` specified by `MinimumStakingAmount`
        /// unless account already has bonded value equal or more than 'minimum_stake'.
        ///
        /// The dispatch origin for this call must be _Signed_ by the staker's account.
        ///
        /// - `core_id`: Id of the core to stake towards.
        /// - `value`: Amount to stake.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::stake())]
        pub fn stake(
            origin: OriginFor<T>,
            core_id: T::CoreId,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let staker = ensure_signed(origin)?;

            ensure!(
                Self::core_info(core_id).is_some(),
                Error::<T>::NotRegistered
            );

            let mut ledger = Self::ledger(&staker);
            let available_balance = Self::available_staking_balance(&staker, &ledger);
            let value_to_stake = value.min(available_balance);

            ensure!(value_to_stake > Zero::zero(), Error::<T>::StakingNothing);

            let current_era = Self::current_era();
            let mut staking_info = Self::core_stake_info(core_id, current_era).unwrap_or_default();
            let mut staker_info = Self::staker_info(core_id, &staker);

            Self::internal_stake(
                &mut staker_info,
                &mut staking_info,
                value_to_stake,
                current_era,
            )?;

            ledger.locked = ledger.locked.saturating_add(value_to_stake);

            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_add(value_to_stake);
                    x.locked = x.locked.saturating_add(value_to_stake);
                }
            });

            Self::update_ledger(&staker, ledger);
            Self::update_staker_info(&staker, core_id, staker_info);
            CoreEraStake::<T>::insert(core_id, current_era, staking_info);

            Self::deposit_event(Event::<T>::Staked {
                staker,
                core: core_id,
                amount: value_to_stake,
            });
            Ok(().into())
        }

        /// Start unbonding process and unstake balance from the core.
        ///
        /// The unstaked amount will no longer be eligible for rewards but still won't be unlocked.
        /// User needs to wait for the unbonding period to finish before being able to withdraw
        /// the funds via `withdraw_unstaked` call.
        ///
        /// In case remaining staked balance on contract is below minimum staking amount,
        /// entire stake for that contract will be unstaked.
        ///
        /// - `core_id`: Id of the core to unstake from.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::unstake())]
        pub fn unstake(
            origin: OriginFor<T>,
            core_id: T::CoreId,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let staker = ensure_signed(origin)?;

            ensure!(value > Zero::zero(), Error::<T>::UnstakingNothing);
            ensure!(
                Self::core_info(core_id).is_some(),
                Error::<T>::NotRegistered
            );

            let current_era = Self::current_era();
            let mut staker_info = Self::staker_info(core_id, &staker);
            let mut core_stake_info =
                Self::core_stake_info(core_id, current_era).unwrap_or_default();

            let value_to_unstake =
                Self::internal_unstake(&mut staker_info, &mut core_stake_info, value, current_era)?;

            let mut ledger = Self::ledger(&staker);
            ledger.unbonding_info.add(UnlockingChunk {
                amount: value_to_unstake,
                unlock_era: current_era + T::UnbondingPeriod::get(),
            });

            ensure!(
                ledger.unbonding_info.len() <= T::MaxUnlocking::get(),
                Error::<T>::TooManyUnlockingChunks
            );

            Self::update_ledger(&staker, ledger);

            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(value_to_unstake);
                }
            });
            Self::update_staker_info(&staker, core_id, staker_info);
            CoreEraStake::<T>::insert(core_id, current_era, core_stake_info);

            Self::deposit_event(Event::<T>::Unstaked {
                staker,
                core: core_id,
                amount: value_to_unstake,
            });

            Ok(().into())
        }

        /// Withdraw all funds that have completed the unbonding process.
        ///
        /// If there are unbonding chunks which will be fully unbonded in future eras,
        /// they will remain and can be withdrawn later.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_unstaked())]
        pub fn withdraw_unstaked(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let staker = ensure_signed(origin)?;

            let mut ledger = Self::ledger(&staker);
            let current_era = Self::current_era();

            let (valid_chunks, future_chunks) = ledger.unbonding_info.partition(current_era);
            let withdraw_amount = valid_chunks.sum();

            ensure!(!withdraw_amount.is_zero(), Error::<T>::NothingToWithdraw);

            ledger.locked = ledger.locked.saturating_sub(withdraw_amount);
            ledger.unbonding_info = future_chunks;

            Self::update_ledger(&staker, ledger);
            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.locked = x.locked.saturating_sub(withdraw_amount)
                }
            });

            Self::deposit_event(Event::<T>::Withdrawn {
                staker,
                amount: withdraw_amount,
            });

            Ok(().into())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::staker_claim_rewards())]
        pub fn staker_claim_rewards(
            origin: OriginFor<T>,
            core_id: T::CoreId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let staker = ensure_signed(origin)?;

            let mut staker_info = Self::staker_info(core_id, &staker);
            let (era, staked) = staker_info.claim();
            ensure!(staked > Zero::zero(), Error::<T>::NoStakeAvailable);

            let current_era = Self::current_era();
            ensure!(era < current_era, Error::<T>::IncorrectEra);

            let staking_info = Self::core_stake_info(core_id, era).unwrap_or_default();
            let reward_and_stake =
                Self::general_era_info(era).ok_or(Error::<T>::UnknownEraReward)?;

            let (_, stakers_joint_reward) =
                Self::core_stakers_split(&staking_info, &reward_and_stake);
            let staker_reward =
                Perbill::from_rational(staked, staking_info.total) * stakers_joint_reward;

            let reward_imbalance = <T as pallet::Config>::Currency::withdraw(
                &Self::account_id(),
                staker_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            <T as pallet::Config>::Currency::resolve_creating(&staker, reward_imbalance);
            Self::update_staker_info(&staker, core_id, staker_info);
            Self::deposit_event(Event::<T>::StakerClaimed {
                staker,
                core: core_id,
                era,
                amount: staker_reward,
            });

            Ok(().into())
        }

        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::core_claim_rewards())]
        pub fn core_claim_rewards(
            origin: OriginFor<T>,
            core_id: T::CoreId,
            #[pallet::compact] era: Era,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            ensure_signed(origin)?;

            let current_era = Self::current_era();
            ensure!(era < current_era, Error::<T>::IncorrectEra);

            let mut core_stake_info = Self::core_stake_info(core_id, era).unwrap_or_default();
            ensure!(
                !core_stake_info.reward_claimed,
                Error::<T>::RewardAlreadyClaimed,
            );
            ensure!(
                core_stake_info.total > Zero::zero(),
                Error::<T>::NoStakeAvailable,
            );

            let reward_and_stake =
                Self::general_era_info(era).ok_or(Error::<T>::UnknownEraReward)?;

            let (reward, _) = Self::core_stakers_split(&core_stake_info, &reward_and_stake);

            let reward_imbalance = <T as pallet::Config>::Currency::withdraw(
                &Self::account_id(),
                reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            let core_account =
                <pallet_inv4::Pallet<T> as CoreAccountDerivation<T>>::derive_core_account(core_id);

            <T as pallet::Config>::Currency::resolve_creating(&core_account, reward_imbalance);
            Self::deposit_event(Event::<T>::CoreClaimed {
                core: core_id,
                destination_account: core_account,
                era,
                amount: reward,
            });

            core_stake_info.reward_claimed = true;
            CoreEraStake::<T>::insert(core_id, era, core_stake_info);

            Ok(().into())
        }

        #[pallet::call_index(8)]
        #[pallet::weight((<T as Config>::WeightInfo::halt_unhalt_pallet(), Pays::No))]
        pub fn halt_unhalt_pallet(origin: OriginFor<T>, halt: bool) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let is_halted = Self::is_halted();

            ensure!(is_halted ^ halt, Error::<T>::NoHaltChange);

            Self::internal_halt_unhalt(halt);

            Self::deposit_event(Event::<T>::HaltChanged { is_halted: halt });

            Ok(().into())
        }

        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::move_stake())]
        pub fn move_stake(
            origin: OriginFor<T>,
            from_core: T::CoreId,
            #[pallet::compact] amount: BalanceOf<T>,
            to_core: T::CoreId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let staker = ensure_signed(origin)?;

            ensure!(from_core != to_core, Error::<T>::MoveStakeToSameCore);
            ensure!(
                Self::core_info(to_core).is_some(),
                Error::<T>::NotRegistered
            );

            let current_era = Self::current_era();
            let mut from_staker_info = Self::staker_info(from_core, &staker);
            let mut from_core_info =
                Self::core_stake_info(from_core, current_era).unwrap_or_default();

            let unstaked_amount = Self::internal_unstake(
                &mut from_staker_info,
                &mut from_core_info,
                amount,
                current_era,
            )?;

            let mut to_staker_info = Self::staker_info(to_core, &staker);
            let mut to_core_info = Self::core_stake_info(to_core, current_era).unwrap_or_default();

            Self::internal_stake(
                &mut to_staker_info,
                &mut to_core_info,
                unstaked_amount,
                current_era,
            )?;

            CoreEraStake::<T>::insert(from_core, current_era, from_core_info);
            Self::update_staker_info(&staker, from_core, from_staker_info);

            CoreEraStake::<T>::insert(to_core, current_era, to_core_info);
            Self::update_staker_info(&staker, to_core, to_staker_info);

            Self::deposit_event(Event::<T>::StakeMoved {
                staker,
                from_core,
                to_core,
                amount: unstaked_amount,
            });

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn internal_stake(
            staker_info: &mut StakerInfo<BalanceOf<T>>,
            staking_info: &mut CoreStakeInfo<BalanceOf<T>>,
            amount: BalanceOf<T>,
            current_era: Era,
        ) -> Result<(), Error<T>> {
            ensure!(
                !staker_info.latest_staked_value().is_zero()
                    || staking_info.number_of_stakers < T::MaxStakersPerCore::get(),
                Error::<T>::MaxStakersReached
            );
            if staker_info.latest_staked_value().is_zero() {
                staking_info.number_of_stakers = staking_info.number_of_stakers.saturating_add(1);
            }

            staker_info
                .stake(current_era, amount)
                .map_err(|_| Error::<T>::UnexpectedStakeInfoEra)?;
            ensure!(
                staker_info.len() < T::MaxEraStakeValues::get(),
                Error::<T>::TooManyEraStakeValues
            );
            ensure!(
                staker_info.latest_staked_value() >= T::MinimumStakingAmount::get(),
                Error::<T>::InsufficientBalance,
            );

            let new_total = staking_info.total.saturating_add(amount);

            staking_info.total = new_total;

            Ok(())
        }

        fn internal_unstake(
            staker_info: &mut StakerInfo<BalanceOf<T>>,
            core_stake_info: &mut CoreStakeInfo<BalanceOf<T>>,
            amount: BalanceOf<T>,
            current_era: Era,
        ) -> Result<BalanceOf<T>, Error<T>> {
            let staked_value = staker_info.latest_staked_value();
            ensure!(staked_value > Zero::zero(), Error::<T>::NoStakeAvailable);

            let remaining = staked_value.saturating_sub(amount);
            let value_to_unstake = if remaining < T::MinimumStakingAmount::get() {
                core_stake_info.number_of_stakers =
                    core_stake_info.number_of_stakers.saturating_sub(1);
                staked_value
            } else {
                amount
            };

            let new_total = core_stake_info.total.saturating_sub(value_to_unstake);

            core_stake_info.total = new_total;

            ensure!(
                value_to_unstake > Zero::zero(),
                Error::<T>::UnstakingNothing
            );

            staker_info
                .unstake(current_era, value_to_unstake)
                .map_err(|_| Error::<T>::UnexpectedStakeInfoEra)?;
            ensure!(
                staker_info.len() < T::MaxEraStakeValues::get(),
                Error::<T>::TooManyEraStakeValues
            );

            Ok(value_to_unstake)
        }

        pub(crate) fn account_id() -> T::AccountId {
            T::PotId::get().into_account_truncating()
        }

        /// Update the ledger for a staker. This will also update the stash lock.
        /// This lock will lock the entire funds except paying for further transactions.
        fn update_ledger(staker: &T::AccountId, ledger: AccountLedger<BalanceOf<T>>) {
            if ledger.is_empty() {
                Ledger::<T>::remove(staker);
                <T as pallet::Config>::Currency::remove_lock(LOCK_ID, staker);
            } else {
                <T as pallet::Config>::Currency::set_lock(
                    LOCK_ID,
                    staker,
                    ledger.locked,
                    WithdrawReasons::all(),
                );
                Ledger::<T>::insert(staker, ledger);
            }
        }

        /// The block rewards are accumulated on the pallets's account during an era.
        /// This function takes a snapshot of the pallet's balance accrued during current era
        /// and stores it for future distribution
        ///
        /// This is called just at the beginning of an era.
        fn reward_balance_snapshot(
            era: Era,
            rewards: RewardInfo<BalanceOf<T>>,
            new_active_stake: BalanceOf<T>,
        ) {
            let mut era_info = Self::general_era_info(era).unwrap_or_default();

            GeneralEraInfo::<T>::insert(
                era + 1,
                EraInfo {
                    rewards: Default::default(),
                    staked: era_info.staked,
                    active_stake: new_active_stake,
                    locked: era_info.locked,
                },
            );

            era_info.rewards = rewards;
            era_info.active_stake = new_active_stake;

            GeneralEraInfo::<T>::insert(era, era_info);
        }

        /// Adds `stakers` and `cores` rewards to the reward pool.
        ///
        /// - `inflation`: Total inflation for the era.
        pub fn rewards(inflation: NegativeImbalanceOf<T>) {
            let (core_part, stakers_part) = <T as Config>::RewardRatio::get();

            let (core, stakers) = inflation.ration(core_part, stakers_part);

            RewardAccumulator::<T>::mutate(|accumulated_reward| {
                accumulated_reward.core = accumulated_reward.core.saturating_add(core.peek());
                accumulated_reward.stakers =
                    accumulated_reward.stakers.saturating_add(stakers.peek());
            });

            <T as pallet::Config>::Currency::resolve_creating(
                &Self::account_id(),
                stakers.merge(core),
            );
        }

        /// Updates staker info for a core.
        fn update_staker_info(
            staker: &T::AccountId,
            core_id: T::CoreId,
            staker_info: StakerInfo<BalanceOf<T>>,
        ) {
            if staker_info.is_empty() {
                GeneralStakerInfo::<T>::remove(core_id, staker)
            } else {
                GeneralStakerInfo::<T>::insert(core_id, staker, staker_info)
            }
        }

        /// Returns available staking balance for the potential staker.
        fn available_staking_balance(
            staker: &T::AccountId,
            ledger: &AccountLedger<BalanceOf<T>>,
        ) -> BalanceOf<T> {
            let free_balance = <T as pallet::Config>::Currency::free_balance(staker)
                .saturating_sub(<T as pallet::Config>::ExistentialDeposit::get());

            free_balance.saturating_sub(ledger.locked)
        }

        /// Returns total value locked by core-staking.
        ///
        /// Note that this can differ from _total staked value_ since some funds might be undergoing the unbonding period.
        pub fn tvl() -> BalanceOf<T> {
            let current_era = Self::current_era();
            if let Some(era_info) = Self::general_era_info(current_era) {
                era_info.locked
            } else {
                Zero::zero()
            }
        }

        /// Calculate reward split between core and stakers.
        ///
        /// Returns (core reward, joint stakers reward)
        pub(crate) fn core_stakers_split(
            core_info: &CoreStakeInfo<BalanceOf<T>>,
            era_info: &EraInfo<BalanceOf<T>>,
        ) -> (BalanceOf<T>, BalanceOf<T>) {
            let core_stake_portion = if core_info.active {
                Perbill::from_rational(core_info.total, era_info.active_stake)
            } else {
                Perbill::zero()
            };
            let stakers_stake_portion = Perbill::from_rational(core_info.total, era_info.staked);

            let core_reward_part = core_stake_portion * era_info.rewards.core;
            let stakers_joint_reward = stakers_stake_portion * era_info.rewards.stakers;

            (core_reward_part, stakers_joint_reward)
        }

        /// Used to copy all `CoreStakeInfo` from the ending era over to the next era.
        fn rotate_staking_info(current_era: Era) -> (Weight, BalanceOf<T>) {
            let next_era = current_era + 1;

            let mut consumed_weight = Weight::zero();

            let mut new_active_stake: BalanceOf<T> = Zero::zero();

            for core_id in RegisteredCore::<T>::iter_keys() {
                consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));

                if let Some(mut staking_info) = Self::core_stake_info(core_id, current_era) {
                    if staking_info.total >= <T as Config>::StakeThresholdForActiveCore::get() {
                        staking_info.active = true;
                        new_active_stake += staking_info.total;
                    } else {
                        staking_info.active = false;
                    }

                    staking_info.reward_claimed = false;
                    CoreEraStake::<T>::insert(core_id, next_era, staking_info);

                    consumed_weight =
                        consumed_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                } else {
                    consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
                }
            }

            (consumed_weight, new_active_stake)
        }

        /// Sets the halt state of the pallet.
        pub fn internal_halt_unhalt(halt: bool) {
            Halted::<T>::put(halt);
        }

        /// Ensure the pallet is not halted.
        pub fn ensure_not_halted() -> Result<(), Error<T>> {
            if Self::is_halted() {
                Err(Error::<T>::Halted)
            } else {
                Ok(())
            }
        }
    }
}
