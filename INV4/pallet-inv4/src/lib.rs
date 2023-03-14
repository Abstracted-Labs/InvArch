//! # Pallet IPS
//! Intellectual Property Sets
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to create and manage IP Sets, which are sets of tokenized IP components, or IP Tokens.
//!
//! ### Pallet Functions
//!
//! - `create` - Create a new IP Set
//! - `send` - Transfer IP Set owner account address
//! - `list` - List an IP Set for sale
//! - `buy` - Buy an IP Set
//! - `destroy` - Delete an IP Set and all of its contents

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub use pallet::*;

pub mod inv4_core;
mod lookup;
//pub mod migrations;
mod dispatch;
pub mod multisig;
pub mod origin;
pub mod util;
pub mod voting;

pub use lookup::INV4Lookup;

#[frame_support::pallet]
pub mod pallet {
    use core::iter::Sum;

    use crate::voting::{Tally, VoteRecord};

    use super::*;
    use frame_support::{
        dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
        pallet_prelude::*,
        storage::Key,
        traits::{fungibles, Currency, Get, GetCallMetadata, ReservableCurrency},
        Parameter,
    };
    use frame_system::{pallet_prelude::*, RawOrigin};
    use primitives::CoreInfo;
    use scale_info::prelude::fmt::Display;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, Member},
        Perbill,
    };
    use sp_std::{boxed::Box, convert::TryInto, vec::Vec};

    pub use super::{inv4_core, multisig};

    use crate::origin::INV4Origin;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type CoreInfoOf<T> =
        CoreInfo<<T as frame_system::Config>::AccountId, inv4_core::CoreMetadataOf<T>>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        /// The IPS Pallet Events
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The IPS ID type
        type CoreId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen
            + Clone;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<
                RuntimeOrigin = <Self as pallet::Config>::RuntimeOrigin,
                PostInfo = PostDispatchInfo,
            > + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + GetCallMetadata
            + Encode;

        /// The maximum numbers of caller accounts on a single Multisig call
        #[pallet::constant]
        type MaxCallers: Get<u32>;

        #[pallet::constant]
        type MaxSubAssets: Get<u32>;

        #[pallet::constant]
        type MaxMetadata: Get<u32>;

        /// The outer `Origin` type.
        type RuntimeOrigin: From<Origin<Self>>
            + From<<Self as frame_system::Config>::RuntimeOrigin>
            + From<RawOrigin<<Self as frame_system::Config>::AccountId>>;

        #[pallet::constant]
        type CoreSeedBalance: Get<BalanceOf<Self>>;

        type AssetsProvider: fungibles::Inspect<Self::AccountId, Balance = BalanceOf<Self>, AssetId = Self::CoreId>
            + fungibles::Mutate<Self::AccountId, AssetId = Self::CoreId>
            + fungibles::Transfer<Self::AccountId, AssetId = Self::CoreId>
            + fungibles::Create<Self::AccountId, AssetId = Self::CoreId>
            + fungibles::Destroy<Self::AccountId, AssetId = Self::CoreId>
            + fungibles::metadata::Mutate<Self::AccountId, AssetId = Self::CoreId>
            + multisig::FreezeAsset<Self::CoreId>;
    }

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::origin]
    pub type Origin<T> =
        INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Next available IPS ID.
    #[pallet::storage]
    #[pallet::getter(fn next_core_id)]
    pub type NextCoreId<T: Config> = StorageValue<_, T::CoreId, ValueQuery>;

    /// Store IPS info. Core IP Set storage
    ///
    /// Return `None` if IPS info not set or removed
    #[pallet::storage]
    #[pallet::getter(fn core_storage)]
    pub type CoreStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::CoreId, CoreInfoOf<T>>;

    /// IPS existence check by owner and IPS ID
    #[pallet::storage]
    #[pallet::getter(fn core_by_account)]
    pub type CoreByAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::CoreId>;

    /// Details of a multisig call. Only holds data for calls while they are in the voting stage.
    ///
    /// Key: (IP Set ID, call hash)
    #[pallet::storage]
    #[pallet::getter(fn multisig)]
    pub type Multisig<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CoreId,
        Blake2_128Concat,
        T::Hash,
        crate::multisig::MultisigOperationOf<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type VoteStorage<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, T::CoreId>,
            Key<Blake2_128Concat, T::Hash>,
            Key<Blake2_128Concat, T::AccountId>,
        ),
        VoteRecord<T>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An IP Set was created
        CoreCreated {
            core_account: T::AccountId,
            core_id: T::CoreId,
        },
        /// IP Tokens were minted
        Minted {
            core_id: T::CoreId,
            target: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// IP Tokens were burned
        Burned {
            core_id: T::CoreId,
            target: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// A vote to execute a call has begun. The call needs more votes to pass.
        ///
        /// Params: caller derived account ID, caller weighted balance, IPT0 token supply, the call hash, the `Call`
        MultisigVoteStarted {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            votes_added: VoteRecord<T>,
            call_hash: T::Hash,
            call: crate::multisig::OpaqueCall<T>,
        },
        /// Voting weight was added towards the vote threshold, but not enough to execute the `Call`
        ///
        /// Params: caller derived account ID, caller weighted balance, IPT0 token supply, the call hash, the `Call`
        MultisigVoteAdded {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            votes_added: VoteRecord<T>,
            current_votes: Tally<T>,
            call_hash: T::Hash,
            call: crate::multisig::OpaqueCall<T>,
        },
        MultisigVoteWithdrawn {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            votes_removed: VoteRecord<T>,
            call_hash: T::Hash,
            call: crate::multisig::OpaqueCall<T>,
        },
        /// Multisig call was executed.
        ///
        /// Params: caller derived account ID, OpaqueCall, dispatch result is ok
        MultisigExecuted {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            call_hash: T::Hash,
            call: crate::multisig::OpaqueCall<T>,
            result: DispatchResult,
        },
        /// The vote on a multisig call was cancelled/withdrawn
        ///
        /// Params: caller derived account ID, the call hash
        MultisigCanceled {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            call_hash: T::Hash,
        },
    }

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IP ID
        NoAvailableCoreId,
        /// Core not found
        CoreNotFound,
        /// The operator has no permission
        /// Ex: Attempting to add a file owned by another account to your IP set
        NoPermission,
        /// The IPS is already owned
        AlreadyOwned,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Can not destroy Core
        CannotDestroyCore,
        /// Value Not Changed
        ValueNotChanged,
        /// Core not found
        CoreDoesntExist,
        NotEnoughAmount,
        /// Max amount of multisig signers reached
        TooManySignatories,
        UnexistentBalance,
        MultisigOperationUninitialized,
        CouldntDecodeCall,
        /// Multisig operation already exists and is available for voting
        MultisigOperationAlreadyExists,
        /// Cannot withdraw a vote on a multisig transaction you have not voted on
        NotAVoter,
        UnknownError,
        /// Sub-asset not found
        SubAssetNotFound,
        /// Sub-asset already exists
        SubAssetAlreadyExists,
        /// Max amount of sub-assets reached
        TooManySubAssets,
        /// This sub-asset has no permission to execute this call
        SubAssetHasNoPermission,
        FailedDivision,
        /// Failed to extract metadata from a `Call`
        CallHasTooFewBytes,

        /// Multisig is not allowed to call these extrinsics
        CantExecuteThisCall,

        /// Division by 0 happened somewhere, maybe you have IPT assets with no decimal points?
        DivisionByZero,
        Overflow,
        Underflow,

        IncompleteVoteCleanup,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        Result<
            INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
            <T as frame_system::Config>::RuntimeOrigin,
        >: From<<T as frame_system::Config>::RuntimeOrigin>,
        <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: Sum,
    {
        /// Create IP (Intellectual Property) Set (IPS)
        #[pallet::call_index(0)]
        #[pallet::weight(900_000_000)]
        pub fn create_core(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            minimum_support: Perbill,
            required_approval: Perbill,
        ) -> DispatchResult {
            Pallet::<T>::inner_create_core(
                owner,
                metadata,
                minimum_support,
                required_approval,
            )
        }

        /// Mint `amount` of specified token to `target` account
        #[pallet::call_index(1)]
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn token_mint(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            Pallet::<T>::inner_token_mint(origin, amount, target)
        }

        /// Burn `amount` of specified token from `target` account
        #[pallet::call_index(2)]
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn token_burn(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            Pallet::<T>::inner_token_burn(origin, amount, target)
        }

        #[pallet::call_index(3)]
        #[pallet::weight(400_000_000)]
        pub fn operate_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            metadata: Option<Vec<u8>>,
            call: Box<<T as pallet::Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_operate_multisig(caller, core_id, metadata, call)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(350_000_000)]
        pub fn vote_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            call_hash: T::Hash,
            aye: bool,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_vote_multisig(caller, core_id, call_hash, aye)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(250_000_000)]
        pub fn withdraw_vote_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            call_hash: T::Hash,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_withdraw_vote_multisig(caller, core_id, call_hash)
        }

        #[pallet::call_index(9)]
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn set_parameters(
            origin: OriginFor<T>,
            metadata: Option<Vec<u8>>,
            minimum_support: Option<Perbill>,
            required_approval: Option<Perbill>,
        ) -> DispatchResult {
            Pallet::<T>::inner_set_parameters(origin, metadata, minimum_support, required_approval)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
