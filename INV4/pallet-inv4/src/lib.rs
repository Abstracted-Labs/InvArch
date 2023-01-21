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
pub mod migrations;
pub mod multisig;
pub mod permissions;
pub mod util;

pub use lookup::INV4Lookup;

#[frame_support::pallet]
pub mod pallet {
    use core::iter::Sum;

    use super::*;
    use frame_support::{
        dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::{Currency, Get, GetCallMetadata, ReservableCurrency},
        BoundedVec, Parameter,
    };
    use frame_system::pallet_prelude::*;
    use primitives::CoreInfo;
    use primitives::{OneOrPercent, SubTokenInfo};
    use scale_info::prelude::fmt::Display;
    use sp_runtime::traits::{AtLeast32BitUnsigned, Member};
    use sp_std::{boxed::Box, convert::TryInto, vec::Vec};

    pub use super::{inv4_core, multisig, permissions};

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
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
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

        #[pallet::constant]
        type CoreSeedBalance: Get<BalanceOf<Self>>;
    }

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

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
        [u8; 32],
        crate::multisig::MultisigOperationOf<T>,
    >;

    /// Details of a sub token.
    ///
    /// Key: (IP Set ID, sub token ID)
    #[pallet::storage]
    #[pallet::getter(fn sub_assets)]
    pub type SubAssets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CoreId,
        Blake2_128Concat,
        T::CoreId,
        SubTokenInfo<T::CoreId, BoundedVec<u8, T::MaxMetadata>>,
    >;

    /// The holdings of a specific account for a specific token.
    ///
    /// Get `account123` balance for the primary token (IPT0) pegged to IP Set `id123`:
    /// `Self::balance((id123, None), account123);`
    /// Replace `None` with `Some(id234)` to get specific sub token balance
    #[pallet::storage]
    #[pallet::getter(fn balance)]
    pub type Balance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::CoreId, Option<T::CoreId>),
        Blake2_128Concat,
        T::AccountId,
        BalanceOf<T>,
    >;

    /// The total issuance of a main token or sub_token.
    #[pallet::storage]
    #[pallet::getter(fn total_issuance)]
    pub type TotalIssuance<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::CoreId,
        Twox64Concat,
        Option<T::CoreId>,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Sub asset voting weight (non IPT0).
    ///
    /// Key: (IP Set ID, sub token ID)
    #[pallet::storage]
    #[pallet::getter(fn asset_weight_storage)]
    pub type AssetWeight<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::CoreId, Blake2_128Concat, T::CoreId, OneOrPercent>;

    /// What pallet functions a sub token has permission to call
    ///
    /// Key: (Ip Set ID, sub token ID), call metadata
    #[pallet::storage]
    #[pallet::getter(fn permissions)]
    pub type Permissions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::CoreId, T::CoreId),
        Blake2_128Concat,
        [u8; 2],
        bool,
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
            token: (T::CoreId, Option<T::CoreId>),
            target: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// IP Tokens were burned
        Burned {
            token: (T::CoreId, Option<T::CoreId>),
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
            votes_added: BalanceOf<T>,
            votes_required: BalanceOf<T>,
            call_hash: [u8; 32],
            call: crate::multisig::OpaqueCall<T>,
        },
        /// Voting weight was added towards the vote threshold, but not enough to execute the `Call`
        ///
        /// Params: caller derived account ID, caller weighted balance, IPT0 token supply, the call hash, the `Call`
        MultisigVoteAdded {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            votes_added: BalanceOf<T>,
            current_votes: BalanceOf<T>,
            votes_required: BalanceOf<T>,
            call_hash: [u8; 32],
            call: crate::multisig::OpaqueCall<T>,
        },
        MultisigVoteWithdrawn {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            votes_removed: BalanceOf<T>,
            call_hash: [u8; 32],
            call: crate::multisig::OpaqueCall<T>,
        },
        /// Multisig call was executed.
        ///
        /// Params: caller derived account ID, OpaqueCall, dispatch result is ok
        MultisigExecuted {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            call_hash: [u8; 32],
            call: crate::multisig::OpaqueCall<T>,
            result: DispatchResult,
        },
        /// The vote on a multisig call was cancelled/withdrawn
        ///
        /// Params: caller derived account ID, the call hash
        MultisigCanceled {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            call_hash: [u8; 32],
        },
        /// One of more sub tokens were created
        SubTokenCreated { id: T::CoreId, metadata: Vec<u8> },
        /// Permission for a given function was just set for a sub token
        ///
        /// Params: IP Set ID, Sub token ID, call_metadata(pallet index, function index), true/false permission
        PermissionSet {
            core_id: T::CoreId,
            sub_token_id: T::CoreId,
            call_index: [u8; 2],
            permission: bool,
        },
        /// The voting weight was set for a sub token
        ///
        /// Params: IP Set ID, Sub token ID, voting power percentage
        WeightSet {
            core_id: T::CoreId,
            sub_token_id: T::CoreId,
            voting_weight: OneOrPercent,
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
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: Sum,
    {
        /// Create IP (Intellectual Property) Set (IPS)
        #[pallet::call_index(0)]
        #[pallet::weight(900_000_000)]
        pub fn create_core(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            ipl_execution_threshold: OneOrPercent,
            ipl_default_asset_weight: OneOrPercent,
            ipl_default_permission: bool,
        ) -> DispatchResult {
            Pallet::<T>::inner_create_core(
                owner,
                metadata,
                ipl_execution_threshold,
                ipl_default_asset_weight,
                ipl_default_permission,
            )
        }

        /// Mint `amount` of specified token to `target` account
        #[pallet::call_index(1)]
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn token_mint(
            owner: OriginFor<T>,
            core_id: T::CoreId,
            sub_token: Option<T::CoreId>,
            amount: BalanceOf<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            Pallet::<T>::inner_token_mint(owner, core_id, sub_token, amount, target)
        }

        /// Burn `amount` of specified token from `target` account
        #[pallet::call_index(2)]
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn token_burn(
            owner: OriginFor<T>,
            core_id: T::CoreId,
            sub_token: Option<T::CoreId>,
            amount: BalanceOf<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            Pallet::<T>::inner_token_burn(owner, core_id, sub_token, amount, target)
        }

        #[pallet::call_index(3)]
        #[pallet::weight(400_000_000)]
        pub fn operate_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            sub_token: Option<T::CoreId>,
            metadata: Option<Vec<u8>>,
            call: Box<<T as pallet::Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_operate_multisig(caller, core_id, sub_token, metadata, call)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(350_000_000)]
        pub fn vote_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            sub_token: Option<T::CoreId>,
            call_hash: [u8; 32],
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_vote_multisig(caller, core_id, sub_token, call_hash)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(250_000_000)]
        pub fn withdraw_vote_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            sub_token: Option<T::CoreId>,
            call_hash: [u8; 32],
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_withdraw_vote_multisig(caller, core_id, sub_token, call_hash)
        }

        /// Create one or more sub tokens for an IP Set
        #[pallet::call_index(6)]
        #[pallet::weight(200_000_000)]
        pub fn create_sub_token(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            sub_token_id: T::CoreId,
            sub_token_metadata: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_create_sub_token(caller, core_id, sub_token_id, sub_token_metadata)
        }

        #[pallet::call_index(7)]
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn set_permission(
            owner: OriginFor<T>,
            core_id: T::CoreId,
            sub_token_id: T::CoreId,
            call_index: [u8; 2],
            permission: bool,
        ) -> DispatchResult {
            Pallet::<T>::inner_set_permission(owner, core_id, sub_token_id, call_index, permission)
        }

        #[pallet::call_index(8)]
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn set_sub_token_weight(
            owner: OriginFor<T>,
            core_id: T::CoreId,
            sub_token_id: T::CoreId,
            voting_weight: OneOrPercent,
        ) -> DispatchResult {
            Pallet::<T>::inner_set_sub_token_weight(owner, core_id, sub_token_id, voting_weight)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
