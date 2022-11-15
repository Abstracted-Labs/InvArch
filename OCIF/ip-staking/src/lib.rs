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
use sp_std::convert::From;
use sp_std::convert::TryInto;

pub mod primitives;
use primitives::*;

const LOCK_ID: LockIdentifier = *b"ip-stake";

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    pub type Era = u32;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
            + ReservableCurrency<Self::AccountId>;

        type IpId: Parameter
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
        type MaxStakersPerIp: Get<u32>;

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
        type PercentForIp: Get<u32>;

        #[pallet::constant]
        type StakeThresholdForActiveIp: Get<BalanceOf<Self>>;
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
    #[pallet::getter(fn ip_info)]
    pub(crate) type RegisteredIp<T: Config> =
        StorageMap<_, Blake2_128Concat, T::IpId, IpInfo<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn general_era_info)]
    pub type GeneralEraInfo<T: Config> = StorageMap<_, Twox64Concat, Era, EraInfo<BalanceOf<T>>>;

    #[pallet::storage]
    #[pallet::getter(fn ip_stake_info)]
    pub type IpEraStake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::IpId,
        Twox64Concat,
        Era,
        IpStakeInfo<BalanceOf<T>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn staker_info)]
    pub type GeneralStakerInfo<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::IpId,
        Blake2_128Concat,
        T::AccountId,
        StakerInfo<BalanceOf<T>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn active_stake)]
    pub type ActiveStake<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        Staked {
            staker: T::AccountId,
            ip: <T as Config>::IpId,
            amount: BalanceOf<T>,
        },
        Unstaked {
            staker: T::AccountId,
            ip: <T as Config>::IpId,
            amount: BalanceOf<T>,
        },
        Withdrawn {
            staker: T::AccountId,
            amount: BalanceOf<T>,
            unregistered: Option<<T as Config>::IpId>,
        },
        IpRegistered {
            ip: <T as Config>::IpId,
        },
        IpUnregistered {
            ip: <T as Config>::IpId,
        },
        NewEra {
            era: u32,
        },
        StakerClaimed {
            staker: T::AccountId,
            ip: <T as Config>::IpId,
            era: u32,
            amount: BalanceOf<T>,
        },
        IpClaimed {
            ip: <T as Config>::IpId,
            destination_account: T::AccountId,
            era: u32,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        StakingNothing,
        InsufficientBalance,
        MaxStakersReached,
        IpNotFound,
        NoStakeAvailable,
        NotUnregisteredIp,
        UnclaimedRewardsAvailable,
        UnstakingNothing,
        NothingToWithdraw,
        IpAlreadyRegistered,
        UnknownEraReward,
        UnexpectedStakeInfoEra,
        TooManyUnlockingChunks,
        RewardAlreadyClaimed,
        IncorrectEra,
        TooManyEraStakeValues,
        NotAStaker,
        NoPermission,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let previous_era = Self::current_era();
            let next_era_starting_block = Self::next_era_starting_block();

            if now >= next_era_starting_block || previous_era.is_zero() {
                let blocks_per_era = T::BlocksPerEra::get();
                let next_era = previous_era + 1;
                CurrentEra::<T>::put(next_era);

                NextEraStartingBlock::<T>::put(now + blocks_per_era);

                let reward = RewardAccumulator::<T>::take();
                Self::reward_balance_snapshot(previous_era, reward);
                let consumed_weight = Self::rotate_staking_info(previous_era);

                Self::deposit_event(Event::<T>::NewEra { era: next_era });

                consumed_weight + T::DbWeight::get().reads_writes(5, 3)
            } else {
                T::DbWeight::get().reads(4)
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1000000000)]
        pub fn register_ip(
            origin: OriginFor<T>,
            ip_id: <T as pallet::Config>::IpId,
        ) -> DispatchResultWithPostInfo {
            let caller = ensure_signed(origin)?;

            ensure!(
                caller
                    == pallet_inv4::util::derive_ips_account::<T, T::IpId, T::AccountId>(
                        ip_id, None
                    ),
                Error::<T>::NoPermission
            );

            ensure!(
                !RegisteredIp::<T>::contains_key(&ip_id),
                Error::<T>::IpAlreadyRegistered,
            );

            T::Currency::reserve(&caller, T::RegisterDeposit::get())?;

            RegisteredIp::<T>::insert(ip_id, IpInfo { account: caller });

            Self::deposit_event(Event::<T>::IpRegistered { ip: ip_id });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn unregister_ip(
            origin: OriginFor<T>,
            ip_id: <T as pallet::Config>::IpId,
        ) -> DispatchResultWithPostInfo {
            let caller = ensure_signed(origin)?;

            ensure!(
                caller
                    == pallet_inv4::util::derive_ips_account::<T, T::IpId, T::AccountId>(
                        ip_id, None
                    ),
                Error::<T>::NoPermission
            );

            ensure!(
                RegisteredIp::<T>::get(&ip_id).is_some(),
                Error::<T>::IpNotFound
            );

            let current_era = Self::current_era();

            let staker_info_prefix = GeneralStakerInfo::<T>::iter_key_prefix(ip_id);

            for staker in staker_info_prefix {
                let mut ip_stake_info =
                    Self::ip_stake_info(&ip_id, current_era).unwrap_or_default();

                let mut staker_info = Self::staker_info(&ip_id, &staker);

                let latest_staked_value = staker_info.latest_staked_value();

                let value_to_unstake = Self::internal_unstake(
                    &mut staker_info,
                    &mut ip_stake_info,
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
                Self::update_staker_info(&staker, ip_id, staker_info);
                IpEraStake::<T>::insert(&ip_id, current_era, ip_stake_info);

                Self::deposit_event(Event::<T>::Unstaked {
                    staker,
                    ip: ip_id,
                    amount: value_to_unstake,
                });
            }

            RegisteredIp::<T>::remove(&ip_id);

            T::Currency::unreserve(&caller, T::RegisterDeposit::get());

            Self::deposit_event(Event::<T>::IpUnregistered { ip: ip_id });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn stake(
            origin: OriginFor<T>,
            ip_id: <T as pallet::Config>::IpId,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;

            ensure!(Self::ip_info(&ip_id).is_some(), Error::<T>::IpNotFound);

            let mut ledger = Self::ledger(&staker);
            let available_balance = Self::available_staking_balance(&staker, &ledger);
            let value_to_stake = value.min(available_balance);
            ensure!(value_to_stake > Zero::zero(), Error::<T>::StakingNothing);

            let current_era = Self::current_era();
            let mut staking_info = Self::ip_stake_info(&ip_id, current_era).unwrap_or_default();
            let mut staker_info = Self::staker_info(&ip_id, &staker);

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
            Self::update_staker_info(&staker, ip_id, staker_info);
            IpEraStake::<T>::insert(&ip_id, current_era, staking_info);

            Self::deposit_event(Event::<T>::Staked {
                staker,
                ip: ip_id,
                amount: value_to_stake,
            });
            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn unstake(
            origin: OriginFor<T>,
            ip_id: <T as pallet::Config>::IpId,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;

            ensure!(value > Zero::zero(), Error::<T>::UnstakingNothing);
            ensure!(Self::ip_info(&ip_id).is_some(), Error::<T>::IpNotFound);

            let current_era = Self::current_era();
            let mut staker_info = Self::staker_info(&ip_id, &staker);
            let mut ip_stake_info = Self::ip_stake_info(&ip_id, current_era).unwrap_or_default();

            let value_to_unstake =
                Self::internal_unstake(&mut staker_info, &mut ip_stake_info, value, current_era)?;

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
            Self::update_staker_info(&staker, ip_id, staker_info);
            IpEraStake::<T>::insert(&ip_id, current_era, ip_stake_info);

            Self::deposit_event(Event::<T>::Unstaked {
                staker,
                ip: ip_id,
                amount: value_to_unstake,
            });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn withdraw_unstaked(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
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
                unregistered: None,
                amount: withdraw_amount,
            });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn staker_claim_rewards(
            origin: OriginFor<T>,
            ip_id: <T as pallet::Config>::IpId,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;

            let mut staker_info = Self::staker_info(&ip_id, &staker);
            let (era, staked) = staker_info.claim();
            ensure!(staked > Zero::zero(), Error::<T>::NoStakeAvailable);

            let current_era = Self::current_era();
            ensure!(era < current_era, Error::<T>::IncorrectEra);

            let staking_info = Self::ip_stake_info(&ip_id, era).unwrap_or_default();
            let reward_and_stake =
                Self::general_era_info(era).ok_or(Error::<T>::UnknownEraReward)?;

            let (_, stakers_joint_reward) =
                Self::ip_stakers_split(&staking_info, &reward_and_stake);
            let staker_reward =
                Perbill::from_rational(staked, staking_info.total) * stakers_joint_reward;

            let reward_imbalance = T::Currency::withdraw(
                &Self::account_id(),
                staker_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            T::Currency::resolve_creating(&staker, reward_imbalance);
            Self::update_staker_info(&staker, ip_id, staker_info);
            Self::deposit_event(Event::<T>::StakerClaimed {
                staker,
                ip: ip_id,
                era,
                amount: staker_reward,
            });

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn ip_claim_rewards(
            origin: OriginFor<T>,
            ip_id: <T as pallet::Config>::IpId,
            #[pallet::compact] era: Era,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let ip_info = RegisteredIp::<T>::get(&ip_id).ok_or(Error::<T>::IpNotFound)?;

            let current_era = Self::current_era();
            ensure!(era < current_era, Error::<T>::IncorrectEra);

            let mut ip_stake_info = Self::ip_stake_info(&ip_id, era).unwrap_or_default();
            ensure!(
                !ip_stake_info.reward_claimed,
                Error::<T>::RewardAlreadyClaimed,
            );
            ensure!(
                ip_stake_info.total > Zero::zero(),
                Error::<T>::NoStakeAvailable,
            );

            let reward_and_stake =
                Self::general_era_info(era).ok_or(Error::<T>::UnknownEraReward)?;

            let (reward, _) = Self::ip_stakers_split(&ip_stake_info, &reward_and_stake);

            let reward_imbalance = T::Currency::withdraw(
                &Self::account_id(),
                reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            T::Currency::resolve_creating(&ip_info.account, reward_imbalance);
            Self::deposit_event(Event::<T>::IpClaimed {
                ip: ip_id,
                destination_account: ip_info.account,
                era,
                amount: reward,
            });

            ip_stake_info.reward_claimed = true;
            IpEraStake::<T>::insert(&ip_id, era, ip_stake_info);

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn internal_stake(
            staker_info: &mut StakerInfo<BalanceOf<T>>,
            staking_info: &mut IpStakeInfo<BalanceOf<T>>,
            amount: BalanceOf<T>,
            current_era: Era,
        ) -> Result<(), Error<T>> {
            ensure!(
                !staker_info.latest_staked_value().is_zero()
                    || staking_info.number_of_stakers < T::MaxStakersPerIp::get(),
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

            if new_total <= <T as Config>::StakeThresholdForActiveIp::get() {
                staking_info.active = true;
            }

            Ok(())
        }

        fn internal_unstake(
            staker_info: &mut StakerInfo<BalanceOf<T>>,
            ip_stake_info: &mut IpStakeInfo<BalanceOf<T>>,
            amount: BalanceOf<T>,
            current_era: Era,
        ) -> Result<BalanceOf<T>, Error<T>> {
            let staked_value = staker_info.latest_staked_value();
            ensure!(staked_value > Zero::zero(), Error::<T>::NoStakeAvailable);

            let remaining = staked_value.saturating_sub(amount);
            let value_to_unstake = if remaining < T::MinimumStakingAmount::get() {
                ip_stake_info.number_of_stakers = ip_stake_info.number_of_stakers.saturating_sub(1);
                staked_value
            } else {
                amount
            };

            let new_total = ip_stake_info.total.saturating_sub(value_to_unstake);

            ip_stake_info.total = new_total;

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

            if new_total <= <T as Config>::StakeThresholdForActiveIp::get() {
                ip_stake_info.active = true;
            } else {
                ip_stake_info.active = false;
            }

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

        fn reward_balance_snapshot(era: Era, rewards: RewardInfo<BalanceOf<T>>) {
            let mut era_info = Self::general_era_info(era).unwrap_or_default();

            GeneralEraInfo::<T>::insert(
                era + 1,
                EraInfo {
                    rewards: Default::default(),
                    staked: era_info.staked,
                    active_stake: Self::active_stake(),
                    locked: era_info.locked,
                },
            );

            era_info.rewards = rewards;

            GeneralEraInfo::<T>::insert(era, era_info);
        }

        pub fn rewards(inflation: NegativeImbalanceOf<T>) {
            let (ip, stakers) = inflation.ration(
                <T as Config>::PercentForIp::get(),
                100 - <T as Config>::PercentForIp::get(),
            );

            RewardAccumulator::<T>::mutate(|accumulated_reward| {
                accumulated_reward.ip = accumulated_reward.ip.saturating_add(ip.peek());
                accumulated_reward.stakers =
                    accumulated_reward.stakers.saturating_add(stakers.peek());
            });

            T::Currency::resolve_creating(&Self::account_id(), stakers.merge(ip));
        }

        fn update_staker_info(
            staker: &T::AccountId,
            ip_id: <T as pallet::Config>::IpId,
            staker_info: StakerInfo<BalanceOf<T>>,
        ) {
            if staker_info.is_empty() {
                GeneralStakerInfo::<T>::remove(ip_id, staker)
            } else {
                GeneralStakerInfo::<T>::insert(ip_id, staker, staker_info)
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

        pub(crate) fn ip_stakers_split(
            ip_info: &IpStakeInfo<BalanceOf<T>>,
            era_info: &EraInfo<BalanceOf<T>>,
        ) -> (BalanceOf<T>, BalanceOf<T>) {
            let ip_stake_portion = Perbill::from_rational(ip_info.total, era_info.active_stake);
            let stakers_stake_portion = Perbill::from_rational(ip_info.total, era_info.staked);

            let ip_reward_part = ip_stake_portion * era_info.rewards.ip;
            let stakers_joint_reward = stakers_stake_portion * era_info.rewards.stakers;

            (ip_reward_part, stakers_joint_reward)
        }

        fn rotate_staking_info(current_era: Era) -> Weight {
            let next_era = current_era + 1;

            let mut consumed_weight = Weight::zero();

            let mut new_active_stake: BalanceOf<T> = Zero::zero();

            for ip_id in RegisteredIp::<T>::iter_keys() {
                consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));

                if let Some(mut staking_info) = Self::ip_stake_info(&ip_id, current_era) {
                    if staking_info.active {
                        new_active_stake += staking_info.total;
                    }

                    staking_info.reward_claimed = false;
                    IpEraStake::<T>::insert(&ip_id, next_era, staking_info);

                    consumed_weight =
                        consumed_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                } else {
                    consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
                }
            }

            ActiveStake::<T>::put(new_active_stake);

            consumed_weight
        }
    }
}
