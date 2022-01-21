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
        type IpsOwnerRewardPercentage: Get<Perbill>;

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
    pub(crate) type RegisteredIps<T: Config> =
        StorageMap<_, Blake2_128Concat, T::IpsId, T::AccountId>;

    /// Total block rewards for the pallet per era and total staked funds
    #[pallet::storage]
    #[pallet::getter(fn era_reward_and_stake)]
    pub(crate) type EraRewardsAndStakes<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, EraRewardAndStake<BalanceOf<T>>>;
    
    /// Stores amount staked and stakers for an IPS per era
    #[pallet::storage]
    #[pallet::getter(fn ips_era_stake)]
    pub(crate) type IpsEraStake<T: Config> = StorageDoubleMap<
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
        NewIpsStaking(T::AccountId, T::IpsId),
        /// IPS removed from IPS staking.
        IpStakingtRemoved(T::AccountId, T::IpsId),
        /// New IPS staking era. Distribute era rewards to IPS.
        NewIpsStakingEra(EraIndex),
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
        /// Targets must be operated IPS
        NotOperatedIps,
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
        /// This account was already used to register contract
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
        RequiredContractPreApproval,
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
                !RegisteredIps::<T>::contains_key(&ips_id),
                Error::<T>::AlreadyRegisteredIps,
            );
            ensure!(ips_id.is_valid(), Error::<T>::IpsIsNotValid);

            if Self::pre_approval_is_enabled() {
                ensure!(
                    PreApprovalOwners::<T>::contains_key(&owner),
                    Error::<T>::RequiredIpsApproval,
                );
            }

            T::Currency::reserve(&owner, T::RegisterDeposit::get())?;

            RegisteredIps::<T>::insert(ips_id.clone(), owner.clone());
            RegisteredOwners::<T>::insert(&owner, ips_id.clone());

            Self::deposit_event(Event::<T>::NewIpsStaking(owner, ips_id));

            Ok(().into())
        }

        /// Unregister existing IPS from IP staking
        ///
        /// This must be called by the owner who registered the IPS.
        ///
        /// Warning: After this action, IPS can not be assigned again.
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
            IpsEraStake::<T>::insert(ips_id.clone(), current_era, empty_staking_info);

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
                Error::<T>::NotOperatedIps
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
            IpsEraStake::<T>::insert(ips_id.clone(), current_era, staking_info);

            Self::deposit_event(Event::<T>::BondAndStake(
                staker,
                ips_id,
                value_to_stake,
            ));
            Ok(().into())
        }

        // TODO: other functions WIP

        impl<T: Config> Pallet<T> {
            // TODO: WIP
        }


    }






}