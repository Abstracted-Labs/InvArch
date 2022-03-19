//! # IP Staking FRAME Pallet.

//! Intellectual Property Staking
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to stake and unstake IP.
//!
//! ### Pallet Functions
//!
//! - `register` - 
//! - `unregister` - 
//! - `bond_and_stake` - 
//! - `unbond_and_unstake` - 
//! - `withdraw_unbonded` - 
//! - `claim` - 
//! - `force_new_era` - 

use super::*;
use frame_support::{
    dispatch::DispatchResult,
    ensure,
    pallet_prelude::*,
    traits::{
        Currency, ExistenceRequirement, Get, Imbalance, LockIdentifier, LockableCurrency,
        OnUnbalanced, ReservableCurrency, WithdrawReasons,
    },
    weights::Weight,
    PalletId,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, Saturating, Zero},
    ArithmeticError, Perbill,
};
use sp_std::convert::From;

const STAKING_ID: LockIdentifier = *b"ipstake";

pub(crate) const REWARD_SCALING: u32 = 2;

#[frame_support::pallet]
pub mod pallet{
    use super::*;

    /// The balance type of this pallet.
    pub type BalanceOf<T> = 
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    // Negative imbalance type of this pallet.
    type NegativeImbalanceOf<T> = 
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

    impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for Pallet<T> {
        fn on_nonzero_unbalanced(block_reward: NegativeImbalanceOf<T>) {
            BlockRewardAccumulator::<T>::mutate(|accumulated_reward| {
                *accumulated_reward = accumulated_reward.saturating_add(block_reward.peek());
            });
            T::Currency::resolve_creating(&Self::account_id(), block_reward);
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {

        /// The staking balance.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
            + ReservableCurrency<Self::AccountId>;

        /// IPS 
        type IpsId = IpsId;

        /// Number of blocks per era.
        #[pallet::constant]
        type BlockPerEra: Get<BlockNumberFor<Self>>;

        /// Minimum bonded deposit for new IPS registration.
        #[pallet::constant]
        type RegisterDeposit: Get<BalanceOf<Self>>;

        /// Percentage of reward paid to IPS owners.
        #[pallet::constant]
        type OwnerRewardPercentage: Get<Perbill>;

        /// Maximum number of unique stakers per IPS.
        #[pallet::constant]
        type MaxNumberOfStakersPerIps: Get<u32>;

        /// Minimum amount user must stake on IPS.
        /// User can stake less if they already have the minimum staking amount staked on that particular IPS.
        #[pallet::constant]
        type MinimumStakingAmount: Get<BalanceOf<Self>>;

        /// Number of eras that are valid when claiming rewards.
        ///
        /// All the rest will be either claimed by the treasury or discarded.
        #[pallet::constant]
        type HistoryDepth: Get<u32>;

        /// Number of eras of doubled claim rewards.
        #[pallet::constant]
        type BonusEraDuration: Get<u32>;

        /// IP Staking Pallet Id
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Minimum amount that should be left on staker account after staking.
        #[pallet::constant]
        type MinimumRemainingAmount: Get<BalanceOf<Self>>;

        /// Max number of unlocking chunks per account Id <-> IPS Id pairing.
        /// If value is zero, unlocking becomes impossible.
        #[pallet::constant]
        type MaxUnlockingChunks: Get<u32>;

        /// Number of eras that need to pass until unstaked value can be withdrawn.
        /// Current era is always counted as full era (regardless how much blocks are remaining).
        /// When set to `0`, it's equal to having no unbonding period.
        #[pallet::constant]
        type UnbondingPeriod: Get<u32>;

        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// Bonded amount for the staker
    #[pallet::storage]
    #[pallet::getter(fn ledger)]
    pub(crate) type Ledger<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AccountLedger<BalanceOf<T>>, ValueQuery>;

    /// The current era index.
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, EraIndex, ValueQuery>;

    /// Accumulator for block rewards during an era. It is reset at every new era
    #[pallet::storage]
    #[pallet::getter(fn block_reward_accumulator)]
    pub type BlockRewardAccumulator<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::type_value]
    pub fn ForceEraOnEmpty() -> Forcing {
        Forcing::ForceNone
    }

    /// Mode of era forcing.
    #[pallet::storage]
    #[pallet::getter(fn force_era)]
    pub type ForceEra<T> = StorageValue<_, Forcing, ValueQuery, ForceEraOnEmpty>;

    /// Registered IPS Owner accounts points to corresponding IPS
    #[pallet::storage]
    #[pallet::getter(fn registered_ips)]
    pub(crate) type RegisteredOwners<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::IpsId>;

    /// Registered IPS points to the owner who registered it
    #[pallet::storage]
    #[pallet::getter(fn registered_owner)]
    pub(crate) type RegisteredIpStaking<T: Config> =
        StorageMap<_, Blake2_128Concat, T::IpsId, T::AccountId>;

    /// Total block rewards for the pallet per era and total staked funds
    #[pallet::storage]
    #[pallet::getter(fn era_reward_and_stake)]
    pub(crate) type EraRewardsAndStakes<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, EraRewardAndStake<BalanceOf<T>>>;
    
    /// Stores amount staked and stakers for an IPS per era
    #[pallet::storage]
    #[pallet::getter(fn ips_era_stake)]
    pub(crate) type IpEraStake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::IpsId,
        Twox64Concat,
        EraIndex,
        EraStakingPoints<T::AccountId, BalanceOf<T>>,
    >;

    /// Stores the current pallet storage version.
    #[pallet::storage]
    #[pallet::getter(fn storage_version)]
    pub(crate) type StorageVersion<T> = StorageValue<_, Version, ValueQuery>;

    #[pallet::type_value]
    pub(crate) fn PreApprovalOnEmpty() -> bool {
        false
    }

    /// Enable or disable pre-approval list for new IPS registration
    #[pallet::storage]
    #[pallet::getter(fn pre_approval_is_enabled)]
    pub(crate) type PreApprovalIsEnabled<T> = StorageValue<_, bool, ValueQuery, PreApprovalOnEmpty>;

    /// List of pre-approved IPS Owners
    #[pallet::storage]
    #[pallet::getter(fn pre_approved_owners)]
    pub(crate) type PreApprovedOwners<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, (), ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Account has bonded and staked funds on an IPS.
        BondAndStake(T::AccountId, T::IpsId, BalanceOf<T>),
        /// Account has unbonded & unstaked some funds. Unbonding process begins.
        UnbondAndUnstake(T::AccountId, T::IpsId, BalanceOf<T>),
        /// Account has withdrawn unbonded funds.
        Withdrawn(T::AccountId, BalanceOf<T>),
        /// New IPS added for staking.
        NewIpStaking(T::AccountId, T::IpsId),
        /// IPS removed from IP staking.
        IpStakingtRemoved(T::AccountId, T::IpsId),
        /// New IP staking era. Distribute era rewards to IPS.
        NewIpStakingEra(EraIndex),
        /// Reward paid to staker or owner.
        Reward(T::AccountId, T::IpsId, EraIndex, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Can not stake with zero value.
        StakingWithNoValue,
        /// Can not stake with value less than minimum staking value
        InsufficientValue,
        /// Number of stakers per IPS exceeded.
        MaxNumberOfStakersExceeded,
        /// Targets must be operated IP Staking
        NotOperatedIpStaking,
        /// IPS isn't staked.
        NotStakedIps,
        /// Unstaking a IPS with zero value
        UnstakingWithNoValue,
        /// There are no previously unbonded funds that can be unstaked and withdrawn.
        NothingToWithdraw,
        /// The IPS is already registered by other account
        AlreadyRegisteredIps,
        /// User attempts to register with address which is not IPS
        IpsIsNotValid,
        /// This account was already used to register IP Staking
        AlreadyUsedOwnerAccount,
        /// IPS not owned by the account id.
        NotOwnedIps,
        /// Report issue on github if this is ever emitted
        UnknownEraReward,
        /// IPS hasn't been staked on in this era.
        NotStaked,
        /// IPS has too many unlocking chunks. Withdraw the existing chunks if possible
        /// or wait for current chunks to complete unlocking process to withdraw them.
        TooManyUnlockingChunks,
        /// IP Staking already claimed in this era and reward is distributed
        AlreadyClaimedInThisEra,
        /// Era parameter is out of bounds
        EraOutOfBounds,
        /// To register a IPS, pre-approval is needed for this address
        RequiredIpsPreApproval,
        /// Owner's account is already part of pre-approved list
        AlreadyPreApprovedOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let force_new_era = Self::force_era().eq(&Forcing::ForceNew);
            let blocks_per_era = T::BlockPerEra::get();
            let previous_era = Self::current_era();

            // Value is compared to 1 since genesis block is ignored
            if now % blocks_per_era == BlockNumberFor::<T>::from(1u32)
                || force_new_era
                || previous_era.is_zero()
            {
                let next_era = previous_era + 1;
                CurrentEra::<T>::put(next_era);

                let reward = BlockRewardAccumulator::<T>::take();
                Self::reward_balance_snapshoot(previous_era, reward);

                if force_new_era {
                    ForceEra::<T>::put(Forcing::ForceNone);
                }

                Self::deposit_event(Event::<T>::NewDappStakingEra(next_era));
            }

            T::DbWeight::get().writes(5)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// register IPS into staking targets.
        ///
        /// Any user can call this function.
        /// However, caller have to have deposit amount.
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn register(
            origin: OriginFor<T>,
            ips_id: IpsId,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;

            ensure!(
                !RegisteredOwners::<T>::contains_key(&owner),
                Error::<T>::AlreadyUsedOwnerAccount,
            );
            ensure!(
                !RegisteredIpStaking::<T>::contains_key(&ips_id),
                Error::<T>::AlreadyRegisteredIpStaking,
            );
            ensure!(ips_id.is_valid(), Error::<T>::IpsIsNotValid);

            if Self::pre_approval_is_enabled() {
                ensure!(
                    PreApprovalOwners::<T>::contains_key(&owner),
                    Error::<T>::RequiredIpsPreApproval,
                );
            }

            T::Currency::reserve(&owner, T::RegisterDeposit::get())?;

            RegisteredIpStaking::<T>::insert(ips_id.clone(), owner.clone());
            RegisteredOwners::<T>::insert(&owner, ips_id.clone());

            Self::deposit_event(Event::<T>::NewIpStaking(owner, ips_id));

            Ok(().into())
        }

        /// Unregister existing IPS from IP staking
        ///
        /// This must be called by the owner who registered the IPS.
        ///
        /// Warning: After this action, IPS can not be assigned again.
        /// 
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn unregister(
            origin: OriginFor<T>,
            ips_id: IpsId,
        ) -> DispatchResultWithPostInfo {
            let Owner = ensure_signed(origin)?;

            let registered_ips = 
                RegisteredOwners::<T>::get(&owner).ok_or(Error::<T>::NotOwnedIps)?;
            
            // This is a sanity check for the unregistration since it requires the caller
            // to input the correct IPS Id.

            ensure!(
                registered_ips == ips_id,
                Error::<T>::NotOwnedIps,
            );

            // We need to unstake all funds that are currently staked
            let current_era = Self::current_era();
            let staking_info = Self::staking_info(&ips_id, current_era);
            for (staker, amount) in staking_info.stakers.iter() {
                let mut ledger = Self::ledger(staker);
                ledger.locked = ledger.locked.saturating_sub(*amount);
                Self::update_ledger(staker, ledger);
            }

            // Need to update total amount staked
            let staking_total = staking_info.total;
            EraRewardsAndStakes::<T>::mutate(
                &current_era,
                // XXX: RewardsAndStakes should be set by `on_initialize` for each era
                |value| {
                    if let Some(x) = value {
                        x.staked = x.staked.saturating_sub(staking_total)
                    }
                },
            );

            // Nett to update staking data for next era
            let empty_staking_info = EraStakingPoints::<T::AccountId, BalanceOf<T>>::default();
            IpEraStake::<T>::insert(ips_id.clone(), current_era, empty_staking_info);

            // Owner account released but IPS can not be released more.
            T::Currency::unreserve(&owner, T::RegisterDeposit::get());
            RegisteredOwners::<T>::remove(&owner);

            Self::deposit_event(Event::<T>::IpStakingtRemoved(owner, ips_id));

            let number_of_stakers = staking_info.stakers.len();
            Ok(Some(T::WeightInfo::unregister(number_of_stakers as u32)).into())

        }

        /// Lock up and stake balance of the origin account.
        ///
        /// `value` must be more than the `minimum_balance` specified by `T::Currency`
        /// unless account already has bonded value equal or more than 'minimum_balance'.
        ///
        /// The dispatch origin for this call must be _Signed_ by the staker's account.
        ///
        /// Effects of staking will be felt at the beginning of the next era.
        ///
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn bond_and_stake(
            origin: OriginFor<T>,
            ips_id: IpsId, 
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;

            // Check that IPS is ready for staking.
            ensure!(
                Self::is_active(&ips_id),
                Error::<T>::NotOperatedIpStaking
            );

            // Ensure that staker has enough balance to bond & stake.
            let free_balance =
                T::Currency::free_balance(&staker).saturating_sub(T::MinimumRemainingAmount::get());

            // Remove already locked funds from the free balance
            let available_balance = free_balance.saturating_sub(ledger.locked);
            let value_to_stake = value.min(available_balance);
            ensure!(
                value_to_stake > Zero::zero(),
                Error::<T>::StakingWithNoValue
            );

            // Get the latest era staking point info or create it if IPS hasn't been staked yet so far.
            let current_era = Self::current_era();
            let mut staking_info = Self::staking_info(&ips_id, current_era);

            // Ensure that we can add additional staker for the IPS
            if !staking_info.stakers.contains_key(&staker) {
                ensure!(
                    staking_info.stakers.len() < T::MaxNumberOfStakersPerIps::get() as usize,
                    Error::<T>::MaxNumberOfStakersExceeded,
                );
            }

            // Increment ledger and total staker value for IPS. 
            // Overflow shouldn't be possible but the check is here just for safety.
            ledger.locked = ledger
                .locked
                .checked_add(&value_to_stake)
                .ok_or(ArithmeticError::Overflow)?;
            staking_info.total = staking_info
                .total
                .checked_add(&value_to_stake)
                .ok_or(ArithmeticError::Overflow)?;

            // Increment personal staking amount.
            let entry = staking_info.stakers.entry(staker.clone()).or_default();
            *entry = entry
                .checked_add(&value_to_stake)
                .ok_or(ArithmeticError::Overflow)?;

            ensure!(
                *entry >= T::MinimumStakingAmount::get(),
                Error::<T>::InsufficientValue,
            );

            // Update total staked value in era.
            EraRewardsAndStakes::<T>::mutate(&current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_add(value_to_stake)
                }
            });

            // Update ledger and payee
            Self::update_ledger(&staker, ledger);

            // Update staked information for IPS in current era
            IpEraStake::<T>::insert(ips_id.clone(), current_era, staking_info);

            Self::deposit_event(Event::<T>::BondAndStake(
                staker,
                ips_id,
                value_to_stake,
            ));
            Ok(().into())
        }

        /// Start unbonding process and unstake balance from the IP Staking.
        ///
        /// The unstaked amount will no longer be eligible for rewards but still won't be unlocked.
        /// User needs to wait for the unbonding period to finish before being able to withdraw
        /// the funds via `withdraw_unbonded` call.
        ///
        /// In case remaining staked balance on IP Staking is below minimum staking amount,
        /// entire stake for that IP Staking will be unstaked.
        /// 
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn unbond_and_unstake(
            origin: OriginFor<T>,
            ips_id: IpsId,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> {
            let staker = ensure_signed(origin)?;

            ensure!(value > Zero::zero(), Error::<T>::UnstakingWithNoValue);
            ensure!(
                Self::is_active(&ips_id),
                Error::<T>::NotOperatedIpStaking,
            );

            // Get the latest era staking points for the IP Staking.
            let current_era = Self::current_era();
            let mut staking_info = Self::staking_info(&ips_id, current_era);

            ensure!(
                staking_info.stakers.contains_key(&staker),
                Error::<T>::NotStakedIps,
            );
            let staked_value = staking_info.stakers[&staker];

            // Calculate the value which will be unstaked.
            let remaining = staked_value.saturating_sub(value);
            let value_to_unstake = if remaining < T::MinimumStakingAmount::get() {
                staking_info.stakers.remove(&staker);
                staked_value
            } else {
                staking_info.stakers.insert(staker.clone(), remaining);
                value
            };
            staking_info.total = staking_info.total.saturating_sub(value_to_unstake);

            // Sanity check
            ensure!(
                value_to_unstake > Zero::zero(),
                Error::<T>::UnstakingWithNoValue
            );

            let mut ledger = Self::ledger(&staker);

            // Update the chunks and write them to storage
            ledger.unbonding_info.add(UnlockingChunk {
                amount: value_to_unstake,
                unlock_era: current_era + T::UnbondingPeriod::get(),
            });
            // This should be done AFTER insertion since it's possible for chunks to merge
            ensure!(
                ledger.unbonding_info.len() <= T::MaxUnlockingChunks::get(),
                Error::<T>::TooManyUnlockingChunks
            );

            Self::update_ledger(&staker, ledger);

            // Update total staked value in era.
            EraRewardsAndStakes::<T>::mutate(&current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(value_to_unstake)
                }
            });

            // Update the era staking points
            IpEraStake::<T>::insert(ips_id.clone(), current_era, staking_info);

            Self::deposit_event(Event::<T>::UnbondAndUnstake(
                staker,
                ips_id,
                value_to_unstake,
            ));

            Ok(().into())

        }

        /// Withdraw all funds that have completed the unbonding process.
        ///
        /// If there are unbonding chunks which will be fully unbonded in future eras,
        /// they will remain and can be withdrawn later.
        ///
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;

            let mut ledger = Self::ledger(&staker);
            let current_era = Self::current_era();

            let (valid_chunks, future_chunks) = ledger.unbonding_info.partition(current_era);
            let withdraw_amount = valid_chunks.sum();

            ensure!(!withdraw_amount.is_zero(), Error::<T>::NothingToWithdraw);

            // Get the staking ledger and update it
            ledger.locked = ledger.locked.saturating_sub(withdraw_amount);
            ledger.unbonding_info = future_chunks;

            Self::update_ledger(&staker, ledger);

            Self::deposit_event(Event::<T>::Withdrawn(staker, withdraw_amount));

            Ok(().into())
        }

        /// Claim the rewards earned by ips_id.
        /// All stakers and owner for this IP Staking will be paid out with single call.
        /// Claim is valid for all unclaimed eras but not longer than history_depth().
        /// Any reward older than history_depth() will go to Treasury.
        /// Any user can call this function.
        /// 
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn claim(
            origin: OriginFor<T>,
            ips_id: IpsId,
            #[pallet::compact] era: EraIndex,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;

            let owner =
                RegisteredIpStaking::<T>::get(&ips_id).ok_or(Error::<T>::NotOperatedIpStaking)?;

            let current_era = Self::current_era();
            let era_low_bound = current_era.saturating_sub(T::HistoryDepth::get());

            ensure!(
                era < current_era && era >= era_low_bound,
                Error::<T>::EraOutOfBounds,
            );
            let mut staking_info = Self::staking_info(&ips_id, era);

            ensure!(
                staking_info.claimed_rewards.is_zero(),
                Error::<T>::AlreadyClaimedInThisEra,
            );

            ensure!(!staking_info.stakers.is_empty(), Error::<T>::NotStaked,);

            let reward_and_stake =
                Self::era_reward_and_stake(era).ok_or(Error::<T>::UnknownEraReward)?;
            
            // Calculate the IP Staking reward for this era.
            let reward_ratio = Perbill::from_rational(staking_info.total, reward_and_stake.staked);
            let ip_staking_reward = if era < T::BonusEraDuration::get() {
                // Double reward as a bonus.
                reward_ratio
                    * reward_and_stake
                        .rewards
                        .saturating_mul(REWARD_SCALING.into())
            } else {
                reward_ratio * reward_and_stake.rewards
            };

            // Withdraw reward funds from the IP Staking
            let reward_pool = T::Currency::withdraw(
                &Self::account_id(),
                ip_staking_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            // Divide reward between stakers and the owner of the IPS Stasking
            let (owner_reward, mut stakers_reward) =
                reward_pool.split(T::OwnerRewardPercentage::get() * ip_staking_reward);

            Self::deposit_event(Event::<T>::Reward(
                owner.clone(),
                ips_id.clone(),
                era,
                owner_reward.peek(),
            ));
            T::Currency::resolve_creating(&owner, owner_reward);

            // Calculate & pay rewards for all stakers
            let stakers_total_reward = stakers_reward.peek();
            for (staker, staked_balance) in &staking_info.stakers {
                let ratio = Perbill::from_rational(*staked_balance, staking_info.total);
                let (reward, new_stakers_reward) =
                    stakers_reward.split(ratio * stakers_total_reward);
                stakers_reward = new_stakers_reward;

                Self::deposit_event(Event::<T>::Reward(
                    staker.clone(),
                    ips_id.clone(),
                    era,
                    reward.peek(),
                ));
                T::Currency::resolve_creating(staker, reward);
            }

            let number_of_payees = staking_info.stakers.len() + 1;

            // Updated counter for total rewards paid to the IP Staking
            staking_info.claimed_rewards = ip_staking_reward;
            <IpEraStake<T>>::insert(&ips_id, era, staking_info);

            Ok(Some(T::WeightInfo::claim(number_of_payees as u32)).into())

        }

        /// Force there to be a new era at the end of the next block. After this, it will be
        /// reset to normal (non-forced) behaviour.
        ///
        /// The dispatch origin must be Root.
        ///
        ///
        /// # <weight>
        /// - No arguments.
        /// - Weight: O(1)
        /// - Write ForceEra
        /// # </weight>
        /// 
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn force_new_era(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            ForceEra::<T>::put(Forcing::ForceNew);
            Ok(())
        }

        /// Add IP Staking to the pre-approved list.
        ///
        /// Sudo call is required
        /// 
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn owner_pre_approval(
        origin: OriginFor<T>,
        owner: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            ensure!(
                !PreApprovedOwners::<T>::contains_key(&owner),
                Error::<T>::AlreadyPreApprovedOwnerr
            );
            PreApprovedOwners::<T>::insert(owner, ());

            Ok(().into())
        }

        /// Enable or disable adding new IP Staking to the pre-approved list
        ///
        /// Sudo call is required
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn enable_owner_pre_approval(
            origin: OriginFor<T>,
            enabled: bool,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            PreApprovalIsEnabled::<T>::put(enabled);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get AccountId assigned to the pallet.
        fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Update the ledger for a staker. This will also update the stash lock.
        /// This lock will lock the entire funds except paying for further transactions.
        fn update_ledger(staker: &T::AccountId, ledger: AccountLedger<BalanceOf<T>>) {
            if ledger.locked.is_zero() && ledger.unbonding_info.is_empty() {
                Ledger::<T>::remove(&staker);
                T::Currency::remove_lock(STAKING_ID, &staker);
            } else {
                T::Currency::set_lock(STAKING_ID, &staker, ledger.locked, WithdrawReasons::all());
                Ledger::<T>::insert(staker, ledger);
            }
        }

        /// The block rewards are accumulated on the pallets's account during an era.
        /// This function takes a snapshot of the pallet's balance accrued during current era
        /// and stores it for future distribution
        ///
        /// This is called just at the beginning of an era.
        fn reward_balance_snapshoot(era: EraIndex, reward: BalanceOf<T>) {
            // Get the reward and stake information for previous era
            let mut reward_and_stake = Self::era_reward_and_stake(era).unwrap_or_default();

            // Prepare info for the next era
            EraRewardsAndStakes::<T>::insert(
                era + 1,
                EraRewardAndStake {
                    rewards: Zero::zero(),
                    staked: reward_and_stake.staked.clone(),
                },
            );

            // Set the reward for the previous era.
            reward_and_stake.rewards = reward;
            EraRewardsAndStakes::<T>::insert(era, reward_and_stake);
        }

        /// This helper returns `EraStakingPoints` for given era if possible or latest stored data
        /// or finally default value if storage have no data for it.
        pub(crate) fn staking_info(
            ips_id: &IpsId,
            era: EraIndex,
        ) -> EraStakingPoints<T::AccountId, BalanceOf<T>> {
            if let Some(staking_info) = IpEraStake::<T>::get(ips_id, era) {
                staking_info
            } else {
                let avail_era = IpEraStake::<T>::iter_key_prefix(&ips_id)
                    .filter(|x| *x <= era)
                    .max()
                    .unwrap_or(Zero::zero());

                let mut staking_points =
                    IpEraStake::<T>::get(ips_id, avail_era).unwrap_or_default();
                // Needs to be reset since otherwise it might seem as if rewards were already claimed for this era.
                staking_points.claimed_rewards = Zero::zero();
                staking_points
            }
        }
        
        /// Check that IP staking have active owner linkage.
        fn is_active(ips_id: &IpsId) -> bool {
            if let Some(owner) = RegisteredIpStaking::<T>::get(ips_id) {
                if let Some(r_ips_id) = RegisteredOwners::<T>::get(&owner) {
                    return r_ips_id == *ips_id;
                }
            }
            false
        }   
    }
}