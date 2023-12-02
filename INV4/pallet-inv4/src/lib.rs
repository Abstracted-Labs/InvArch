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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

//pub mod migrations;

mod dispatch;
pub mod fee_handling;
pub mod inv4_core;
mod lookup;
pub mod multisig;
pub mod origin;
pub mod util;
pub mod voting;
pub mod weights;

use fee_handling::FeeAsset;
pub use lookup::INV4Lookup;
pub use util::CoreAccountConversion;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use core::iter::Sum;

    use crate::{
        fee_handling::MultisigFeeHandler,
        voting::{Tally, VoteRecord},
    };

    use super::*;
    use frame_support::{
        dispatch::{Dispatchable, GetDispatchInfo, Pays, PostDispatchInfo},
        pallet_prelude::*,
        traits::{
            fungibles,
            fungibles::{Balanced, Inspect},
            Currency, Get, GetCallMetadata, ReservableCurrency,
        },
        transactional, Parameter,
    };
    use frame_system::{pallet_prelude::*, RawOrigin};
    use primitives::CoreInfo;
    use scale_info::prelude::fmt::Display;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, Member},
        Perbill,
    };
    use sp_std::{boxed::Box, convert::TryInto, vec::Vec};
    use xcm::latest::NetworkId;

    pub use super::{inv4_core, multisig};

    use crate::origin::INV4Origin;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type CoreInfoOf<T> =
        CoreInfo<<T as frame_system::Config>::AccountId, inv4_core::CoreMetadataOf<T>>;

    pub type CallOf<T> = <T as Config>::RuntimeCall;

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
            + Clone
            + Into<u32>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<
                Info = frame_support::dispatch::DispatchInfo,
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

        #[pallet::constant]
        type CoreCreationFee: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type KSMCoreCreationFee: Get<
            <<Self as Config>::Tokens as Inspect<<Self as frame_system::Config>::AccountId>>::Balance,
        >;

        #[pallet::constant]
        type KSMAssetId: Get<<<Self as Config>::Tokens as Inspect<<Self as frame_system::Config>::AccountId>>::AssetId>;

        type AssetsProvider: fungibles::Inspect<Self::AccountId, Balance = BalanceOf<Self>, AssetId = Self::CoreId>
            + fungibles::Mutate<Self::AccountId, AssetId = Self::CoreId>;

        type Tokens: Balanced<Self::AccountId> + Inspect<Self::AccountId>;

        type FeeCharger: MultisigFeeHandler<Self>;

        #[pallet::constant]
        type GenesisHash: Get<<Self as frame_system::Config>::Hash>;

        #[pallet::constant]
        type GlobalNetworkId: Get<NetworkId>;

        #[pallet::constant]
        type ParaId: Get<u32>;

        #[pallet::constant]
        type MaxCallSize: Get<u32>;

        type WeightInfo: WeightInfo;
    }

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    #[pallet::origin]
    pub type Origin<T> = INV4Origin<T>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Next available Core ID.
    #[pallet::storage]
    #[pallet::getter(fn next_core_id)]
    pub type NextCoreId<T: Config> = StorageValue<_, T::CoreId, ValueQuery>;

    /// Store Core info.
    #[pallet::storage]
    #[pallet::getter(fn core_storage)]
    pub type CoreStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::CoreId, CoreInfoOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn core_by_account)]
    pub type CoreByAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::CoreId>;

    /// Details of a multisig call. Only holds data for calls while they are in the voting stage.
    ///
    /// Key: (Core ID, call hash)
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

    /// Stores a list of members for each Core.
    /// This storage should be always handled by the runtime and mutated by CoreAssets hooks.
    // We make this a StorageDoubleMap so we don't have to bound the list.
    #[pallet::storage]
    #[pallet::getter(fn core_members)]
    pub type CoreMembers<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::CoreId, Blake2_128Concat, T::AccountId, ()>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An IP Set was created
        CoreCreated {
            core_account: T::AccountId,
            core_id: T::CoreId,
            metadata: Vec<u8>,
            minimum_support: Perbill,
            required_approval: Perbill,
        },
        ParametersSet {
            core_id: T::CoreId,
            metadata: Option<Vec<u8>>,
            minimum_support: Option<Perbill>,
            required_approval: Option<Perbill>,
            frozen_tokens: Option<bool>,
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
        },
        MultisigVoteWithdrawn {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            votes_removed: VoteRecord<T>,
            call_hash: T::Hash,
        },
        /// Multisig call was executed.
        ///
        /// Params: caller derived account ID, OpaqueCall, dispatch result is ok
        MultisigExecuted {
            core_id: T::CoreId,
            executor_account: T::AccountId,
            voter: T::AccountId,
            call_hash: T::Hash,
            call: CallOf<T>,
            result: DispatchResult,
        },
        /// A multisig call was cancelled
        MultisigCanceled {
            core_id: T::CoreId,
            call_hash: T::Hash,
        },
    }

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available Core ID
        NoAvailableCoreId,
        /// Core not found
        CoreNotFound,
        /// The operator has no permission
        /// Ex: Attempting to add a file owned by another account to your IP set
        NoPermission,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Failed because the multisig call has been voted by more than the limit amount of members.
        MaxCallersExceeded,
        /// Multisig call not found.
        MultisigCallNotFound,
        /// Failed to decode stored multisig call.
        FailedDecodingCall,
        /// Multisig operation already exists and is available for voting.
        MultisigCallAlreadyExists,
        /// Cannot withdraw a vote on a multisig transaction you have not voted on.
        NotAVoter,
        /// Failed to extract metadata from a `Call`
        CallHasTooFewBytes,
        /// Incomplete vote cleanup.
        IncompleteVoteCleanup,
        /// Multisig fee payment failed, probably due to lack of funds to pay for fees.
        CallFeePaymentFailed,

        MaxCallLengthExceeded,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        Result<
            INV4Origin<T>,
            <T as frame_system::Config>::RuntimeOrigin,
        >: From<<T as frame_system::Config>::RuntimeOrigin>,
        <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: Sum,
    <T as frame_system::Config>::AccountId: From<[u8; 32]>,
    {
        /// Create IP (Intellectual Property) Set (IPS)
        #[pallet::call_index(0)]
        #[transactional]
        #[pallet::weight(<T as Config>::WeightInfo::create_core(metadata.len() as u32))]
        pub fn create_core(
            owner: OriginFor<T>,
            metadata: BoundedVec<u8, T::MaxMetadata>,
            minimum_support: Perbill,
            required_approval: Perbill,
            creation_fee_asset: FeeAsset,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_create_core(
                owner,
                metadata,
                minimum_support,
                required_approval,
                creation_fee_asset,
            )?;

            Ok(PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::No,
            })
        }

        /// Mint `amount` of specified token to `target` account
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::token_mint())]
        pub fn token_mint(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            Pallet::<T>::inner_token_mint(origin, amount, target)
        }

        /// Burn `amount` of specified token from `target` account
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::token_burn())]
        pub fn token_burn(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            Pallet::<T>::inner_token_burn(origin, amount, target)
        }

        #[pallet::call_index(3)]
        #[pallet::weight(
            <T as Config>::WeightInfo::operate_multisig(
                metadata.clone().map(|m| m.len()).unwrap_or(0) as u32,
                call.using_encoded(|c| c.len() as u32)
            )
        )]
        pub fn operate_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            metadata: Option<BoundedVec<u8, T::MaxMetadata>>,
            fee_asset: FeeAsset,
            call: Box<<T as pallet::Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_operate_multisig(caller, core_id, metadata, fee_asset, call)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::vote_multisig())]
        pub fn vote_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            call_hash: T::Hash,
            aye: bool,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_vote_multisig(caller, core_id, call_hash, aye)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_vote_multisig())]
        pub fn withdraw_vote_multisig(
            caller: OriginFor<T>,
            core_id: T::CoreId,
            call_hash: T::Hash,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_withdraw_vote_multisig(caller, core_id, call_hash)
        }

        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_multisig_proposal())]
        pub fn cancel_multisig_proposal(
            caller: OriginFor<T>,
            call_hash: T::Hash,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_cancel_multisig_proposal(caller, call_hash)
        }

        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::set_parameters(
            metadata.clone().map(|m| m.len()).unwrap_or(0) as u32
        ))]
        pub fn set_parameters(
            origin: OriginFor<T>,
            metadata: Option<BoundedVec<u8, T::MaxMetadata>>,
            minimum_support: Option<Perbill>,
            required_approval: Option<Perbill>,
            frozen_tokens: Option<bool>,
        ) -> DispatchResult {
            Pallet::<T>::inner_set_parameters(origin, metadata, minimum_support, required_approval, frozen_tokens)
        }
    }
}
