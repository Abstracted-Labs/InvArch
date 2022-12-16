#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt::Display;
use frame_support::{
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
use sp_arithmetic::traits::AtLeast32BitUnsigned;
use sp_runtime::{
    traits::{AccountIdConversion, Saturating, Zero},
    Perbill,
};
use sp_std::convert::{From, TryInto};

pub mod primitives;
use primitives::*;

#[cfg(test)]
mod testing;

const LOCK_ID: LockIdentifier = *b"ocif-stk";

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use pallet_inv4::util::derive_ips_account;

    use super::*;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    pub type CoreMetadataOf<T> = CoreMetadata<
        BoundedVec<u8, <T as Config>::MaxNameLength>,
        BoundedVec<u8, <T as Config>::MaxDescriptionLength>,
        BoundedVec<u8, <T as Config>::MaxImageUrlLength>,
    >;

    pub type CoreInfoOf<T> = CoreInfo<<T as frame_system::Config>::AccountId, CoreMetadataOf<T>>;

    pub type Era = u32;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
            + ReservableCurrency<Self::AccountId>;

        type CoreId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen
            + Clone;

        #[pallet::constant]
        type BlocksPerEra: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type RegisterDeposit: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxStakersPerCore: Get<u32>;

        #[pallet::constant]
        type MinimumStakingAmount: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type PotId: Get<PalletId>;

        #[pallet::constant]
        type ExistentialDeposit: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxUnlocking: Get<u32>;

        #[pallet::constant]
        type UnbondingPeriod: Get<u32>;

        #[pallet::constant]
        type MaxEraStakeValues: Get<u32>;

        #[pallet::constant]
        type RewardRatio: Get<(u32, u32)>;

        #[pallet::constant]
        type StakeThresholdForActiveCore: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxNameLength: Get<u32>;

        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;

        #[pallet::constant]
        type MaxImageUrlLength: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn ledger)]
    pub type Ledger<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AccountLedger<BalanceOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, Era, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn reward_accumulator)]
    pub type RewardAccumulator<T> = StorageValue<_, RewardInfo<BalanceOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_era_starting_block)]
    pub type NextEraStartingBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn core_info)]
    pub(crate) type RegisteredCore<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CoreId, CoreInfoOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn general_era_info)]
    pub type GeneralEraInfo<T: Config> = StorageMap<_, Twox64Concat, Era, EraInfo<BalanceOf<T>>>;

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

    #[pallet::storage]
    #[pallet::getter(fn is_halted)]
    pub type Halted<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        Staked {
            staker: T::AccountId,
            core: <T as Config>::CoreId,
            amount: BalanceOf<T>,
        },
        Unstaked {
            staker: T::AccountId,
            core: <T as Config>::CoreId,
            amount: BalanceOf<T>,
        },
        Withdrawn {
            staker: T::AccountId,
            amount: BalanceOf<T>,
        },
        CoreRegistered {
            core: <T as Config>::CoreId,
        },
        CoreUnregistered {
            core: <T as Config>::CoreId,
        },
        NewEra {
            era: u32,
        },
        StakerClaimed {
            staker: T::AccountId,
            core: <T as Config>::CoreId,
            era: u32,
            amount: BalanceOf<T>,
        },
        CoreClaimed {
            core: <T as Config>::CoreId,
            destination_account: T::AccountId,
            era: u32,
            amount: BalanceOf<T>,
        },
        HaltChanged {
            is_halted: bool,
        },
        MetadataChanged {
            core: <T as Config>::CoreId,
            old_metadata: CoreMetadataOf<T>,
            new_metadata: CoreMetadataOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        StakingNothing,
        InsufficientBalance,
        MaxStakersReached,
        CoreNotFound,
        NoStakeAvailable,
        NotUnregisteredCore,
        UnclaimedRewardsAvailable,
        UnstakingNothing,
        NothingToWithdraw,
        CoreAlreadyRegistered,
        UnknownEraReward,
        UnexpectedStakeInfoEra,
        TooManyUnlockingChunks,
        RewardAlreadyClaimed,
        IncorrectEra,
        TooManyEraStakeValues,
        NotAStaker,
        NoPermission,
        MaxNameExceeded,
        MaxDescriptionExceeded,
        NotRegistered,
        Halted,
        NoHaltChange,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            if Self::is_halted() {
                return T::DbWeight::get().reads(1);
            }

            let previous_era = Self::current_era();
            let next_era_starting_block = Self::next_era_starting_block();

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
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1000000000)]
        pub fn register_core(
            origin: OriginFor<T>,
            core_id: <T as pallet::Config>::CoreId,
            metadata: CoreMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let caller = ensure_signed(origin)?;

            ensure!(
                caller
                    == pallet_inv4::util::derive_ips_account::<T, T::CoreId, T::AccountId>(
                        core_id, None
                    ),
                Error::<T>::NoPermission
            );

            ensure!(
                !RegisteredCore::<T>::contains_key(&core_id),
                Error::<T>::CoreAlreadyRegistered,
            );

            T::Currency::reserve(&caller, T::RegisterDeposit::get())?;

            RegisteredCore::<T>::insert(
                core_id,
                CoreInfo {
                    account: caller,
                    metadata,
                },
            );

            Self::deposit_event(Event::<T>::CoreRegistered { core: core_id });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn unregister_core(
            origin: OriginFor<T>,
            core_id: <T as pallet::Config>::CoreId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let caller = ensure_signed(origin)?;

            ensure!(
                caller
                    == pallet_inv4::util::derive_ips_account::<T, T::CoreId, T::AccountId>(
                        core_id, None
                    ),
                Error::<T>::NoPermission
            );

            ensure!(
                RegisteredCore::<T>::get(&core_id).is_some(),
                Error::<T>::NotRegistered
            );

            let current_era = Self::current_era();

            let staker_info_prefix = GeneralStakerInfo::<T>::iter_key_prefix(core_id);

            for staker in staker_info_prefix {
                let mut core_stake_info =
                    Self::core_stake_info(&core_id, current_era).unwrap_or_default();

                let mut staker_info = Self::staker_info(&core_id, &staker);

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

                GeneralEraInfo::<T>::mutate(&current_era, |value| {
                    if let Some(x) = value {
                        x.staked = x.staked.saturating_sub(value_to_unstake);
                    }
                });
                Self::update_staker_info(&staker, core_id, staker_info);
                CoreEraStake::<T>::insert(&core_id, current_era, core_stake_info);

                Self::deposit_event(Event::<T>::Unstaked {
                    staker,
                    core: core_id,
                    amount: value_to_unstake,
                });
            }

            RegisteredCore::<T>::remove(&core_id);

            T::Currency::unreserve(&caller, T::RegisterDeposit::get());

            Self::deposit_event(Event::<T>::CoreUnregistered { core: core_id });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn change_core_metadata(
            origin: OriginFor<T>,
            core_id: <T as pallet::Config>::CoreId,
            new_metadata: CoreMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let caller = ensure_signed(origin)?;

            ensure!(
                caller
                    == pallet_inv4::util::derive_ips_account::<T, T::CoreId, T::AccountId>(
                        core_id, None
                    ),
                Error::<T>::NoPermission
            );

            RegisteredCore::<T>::try_mutate(core_id, |core| {
                let mut new_core = core.take().ok_or(Error::<T>::NotRegistered)?;

                let old_metadata = new_core.metadata;

                new_core.metadata = new_metadata.clone();

                *core = Some(new_core);

                Self::deposit_event(Event::<T>::MetadataChanged {
                    core: core_id,
                    old_metadata,
                    new_metadata,
                });

                Ok(().into())
            })
        }

        #[pallet::weight(1000000000)]
        pub fn stake(
            origin: OriginFor<T>,
            core_id: <T as pallet::Config>::CoreId,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let staker = ensure_signed(origin)?;

            ensure!(
                Self::core_info(&core_id).is_some(),
                Error::<T>::NotRegistered
            );

            let mut ledger = Self::ledger(&staker);
            let available_balance = Self::available_staking_balance(&staker, &ledger);
            let value_to_stake = value.min(available_balance);

            ensure!(value_to_stake > Zero::zero(), Error::<T>::StakingNothing);

            let current_era = Self::current_era();
            let mut staking_info = Self::core_stake_info(&core_id, current_era).unwrap_or_default();
            let mut staker_info = Self::staker_info(&core_id, &staker);

            Self::internal_stake(
                &mut staker_info,
                &mut staking_info,
                value_to_stake,
                current_era,
            )?;

            ledger.locked = ledger.locked.saturating_add(value_to_stake);

            GeneralEraInfo::<T>::mutate(&current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_add(value_to_stake);
                    x.locked = x.locked.saturating_add(value_to_stake);
                }
            });

            Self::update_ledger(&staker, ledger);
            Self::update_staker_info(&staker, core_id, staker_info);
            CoreEraStake::<T>::insert(&core_id, current_era, staking_info);

            Self::deposit_event(Event::<T>::Staked {
                staker,
                core: core_id,
                amount: value_to_stake,
            });
            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn unstake(
            origin: OriginFor<T>,
            core_id: <T as pallet::Config>::CoreId,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let staker = ensure_signed(origin)?;

            ensure!(value > Zero::zero(), Error::<T>::UnstakingNothing);
            ensure!(
                Self::core_info(&core_id).is_some(),
                Error::<T>::NotRegistered
            );

            let current_era = Self::current_era();
            let mut staker_info = Self::staker_info(&core_id, &staker);
            let mut core_stake_info =
                Self::core_stake_info(&core_id, current_era).unwrap_or_default();

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

            GeneralEraInfo::<T>::mutate(&current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(value_to_unstake);
                }
            });
            Self::update_staker_info(&staker, core_id, staker_info);
            CoreEraStake::<T>::insert(&core_id, current_era, core_stake_info);

            Self::deposit_event(Event::<T>::Unstaked {
                staker,
                core: core_id,
                amount: value_to_unstake,
            });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
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
            GeneralEraInfo::<T>::mutate(&current_era, |value| {
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

        #[pallet::weight(1000000000)]
        pub fn staker_claim_rewards(
            origin: OriginFor<T>,
            core_id: <T as pallet::Config>::CoreId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            let staker = ensure_signed(origin)?;

            let mut staker_info = Self::staker_info(&core_id, &staker);
            let (era, staked) = staker_info.claim();
            ensure!(staked > Zero::zero(), Error::<T>::NoStakeAvailable);

            let current_era = Self::current_era();
            ensure!(era < current_era, Error::<T>::IncorrectEra);

            let staking_info = Self::core_stake_info(&core_id, era).unwrap_or_default();
            let reward_and_stake =
                Self::general_era_info(era).ok_or(Error::<T>::UnknownEraReward)?;

            let (_, stakers_joint_reward) =
                Self::core_stakers_split(&staking_info, &reward_and_stake);
            let staker_reward =
                Perbill::from_rational(staked, staking_info.total) * stakers_joint_reward;

            let reward_imbalance = T::Currency::withdraw(
                &Self::account_id(),
                staker_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            T::Currency::resolve_creating(&staker, reward_imbalance);
            Self::update_staker_info(&staker, core_id, staker_info);
            Self::deposit_event(Event::<T>::StakerClaimed {
                staker,
                core: core_id,
                era,
                amount: staker_reward,
            });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn core_claim_rewards(
            origin: OriginFor<T>,
            core_id: <T as pallet::Config>::CoreId,
            #[pallet::compact] era: Era,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_not_halted()?;

            ensure_signed(origin)?;

            let current_era = Self::current_era();
            ensure!(era < current_era, Error::<T>::IncorrectEra);

            let mut core_stake_info = Self::core_stake_info(&core_id, era).unwrap_or_default();
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

            let reward_imbalance = T::Currency::withdraw(
                &Self::account_id(),
                reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            let core_account = derive_ips_account::<T, T::CoreId, T::AccountId>(core_id, None);

            T::Currency::resolve_creating(&core_account, reward_imbalance);
            Self::deposit_event(Event::<T>::CoreClaimed {
                core: core_id,
                destination_account: core_account,
                era,
                amount: reward,
            });

            core_stake_info.reward_claimed = true;
            CoreEraStake::<T>::insert(&core_id, era, core_stake_info);

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn halt_unhalt_pallet(origin: OriginFor<T>, halt: bool) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let is_halted = Self::is_halted();

            ensure!(is_halted ^ halt, Error::<T>::NoHaltChange);

            Self::internal_halt_unhalt(halt);

            Self::deposit_event(Event::<T>::HaltChanged { is_halted: halt });

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

        fn update_ledger(staker: &T::AccountId, ledger: AccountLedger<BalanceOf<T>>) {
            if ledger.is_empty() {
                Ledger::<T>::remove(&staker);
                T::Currency::remove_lock(LOCK_ID, staker);
            } else {
                T::Currency::set_lock(LOCK_ID, staker, ledger.locked, WithdrawReasons::all());
                Ledger::<T>::insert(staker, ledger);
            }
        }

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

            GeneralEraInfo::<T>::insert(era, era_info);
        }

        pub fn rewards(inflation: NegativeImbalanceOf<T>) {
            let (core_part, stakers_part) = <T as Config>::RewardRatio::get();

            let (core, stakers) = inflation.ration(core_part, stakers_part);

            RewardAccumulator::<T>::mutate(|accumulated_reward| {
                accumulated_reward.core = accumulated_reward.core.saturating_add(core.peek());
                accumulated_reward.stakers =
                    accumulated_reward.stakers.saturating_add(stakers.peek());
            });

            T::Currency::resolve_creating(&Self::account_id(), stakers.merge(core));
        }

        fn update_staker_info(
            staker: &T::AccountId,
            core_id: <T as pallet::Config>::CoreId,
            staker_info: StakerInfo<BalanceOf<T>>,
        ) {
            if staker_info.is_empty() {
                GeneralStakerInfo::<T>::remove(core_id, staker)
            } else {
                GeneralStakerInfo::<T>::insert(core_id, staker, staker_info)
            }
        }

        fn available_staking_balance(
            staker: &T::AccountId,
            ledger: &AccountLedger<BalanceOf<T>>,
        ) -> BalanceOf<T> {
            let free_balance =
                T::Currency::free_balance(staker).saturating_sub(T::ExistentialDeposit::get());

            free_balance.saturating_sub(ledger.locked)
        }

        pub fn tvl() -> BalanceOf<T> {
            let current_era = Self::current_era();
            if let Some(era_info) = Self::general_era_info(current_era) {
                era_info.locked
            } else {
                Zero::zero()
            }
        }

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

        fn rotate_staking_info(current_era: Era) -> (Weight, BalanceOf<T>) {
            let next_era = current_era + 1;

            let mut consumed_weight = Weight::zero();

            let mut new_active_stake: BalanceOf<T> = Zero::zero();

            for core_id in RegisteredCore::<T>::iter_keys() {
                consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));

                if let Some(mut staking_info) = Self::core_stake_info(&core_id, current_era) {
                    if staking_info.total >= <T as Config>::StakeThresholdForActiveCore::get() {
                        staking_info.active = true;
                        new_active_stake += staking_info.total;
                    } else {
                        staking_info.active = false;
                    }

                    staking_info.reward_claimed = false;
                    CoreEraStake::<T>::insert(&core_id, next_era, staking_info);

                    consumed_weight =
                        consumed_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                } else {
                    consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
                }
            }

            (consumed_weight, new_active_stake)
        }

        pub fn internal_halt_unhalt(halt: bool) {
            Halted::<T>::put(halt);
        }

        pub fn ensure_not_halted() -> Result<(), Error<T>> {
            if Self::is_halted() {
                Err(Error::<T>::Halted)
            } else {
                Ok(())
            }
        }
    }
}
