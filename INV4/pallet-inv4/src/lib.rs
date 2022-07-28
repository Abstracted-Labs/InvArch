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

use frame_support::{
    dispatch::Dispatchable,
    pallet_prelude::*,
    traits::{Currency as FSCurrency, Get, GetCallMetadata},
    weights::{GetDispatchInfo, PostDispatchInfo, WeightToFee},
    BoundedVec, Parameter,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, Member};
use sp_std::{boxed::Box, convert::TryInto, vec::Vec};

/// Import the primitives crate
use primitives::IpInfo;

pub use pallet::*;

pub mod ipl;
pub mod ips;
pub mod ipt;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use primitives::{OneOrPercent, SubIptInfo};
    use scale_info::prelude::fmt::Display;
    use sp_std::iter::Sum;

    pub use super::{ipl, ips, ipt};

    use crate::ipl::LicenseList;

    use rmrk_traits::primitives::{CollectionId, NftId};

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + ipf::Config
        + pallet_balances::Config
        + pallet_rmrk_core::Config
        + pallet_uniques::Config<
            CollectionId = rmrk_traits::primitives::CollectionId,
            ItemId = rmrk_traits::primitives::NftId,
        >
    {
        /// The IPS Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The IPS ID type
        type IpId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen
            + Clone;

        /// Currency
        type Currency: FSCurrency<Self::AccountId>;

        type Balance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo
            + Sum<<Self as pallet::Config>::Balance>
            + IsType<<Self as pallet_balances::Config>::Balance>
            + IsType<<<Self as pallet::Config>::WeightToFee as WeightToFee>::Balance>
            + From<u128>;

        #[pallet::constant]
        type ExistentialDeposit: Get<<Self as pallet::Config>::Balance>;

        type Licenses: Parameter + LicenseList<Self>;

        /// The overarching call type.
        type Call: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + GetCallMetadata
            + Encode;

        type WeightToFee: WeightToFee;

        /// The maximum numbers of caller accounts on a single Multisig call
        #[pallet::constant]
        type MaxCallers: Get<u32>;

        #[pallet::constant]
        type MaxSubAssets: Get<u32>;

        #[pallet::constant]
        type MaxMetadata: Get<u32>;

        /// Max bytes for asset permission WASM code
        #[pallet::constant]
        type MaxWasmPermissionBytes: Get<u32>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as FSCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type IpInfoOf<T> = IpInfo<
        <T as frame_system::Config>::AccountId,
        BoundedVec<AnyIdOf<T>, <T as Config>::MaxMetadata>,
        ips::IpsMetadataOf<T>,
        <T as Config>::IpId,
        <T as Config>::Balance,
        BoundedVec<u8, <T as Config>::MaxMetadata>,
        <T as frame_system::Config>::Hash,
    >;

    /// Valid types that an IP Set can hold
    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    pub enum AnyId<IpsId, IpfId, RmrkNftTuple, RmrkCollectionId> {
        IpfId(IpfId),
        RmrkNft(RmrkNftTuple),
        RmrkCollection(RmrkCollectionId),
        IpsId(IpsId),
    }

    pub type AnyIdOf<T> =
        AnyId<<T as Config>::IpId, <T as ipf::Config>::IpfId, (CollectionId, NftId), CollectionId>;

    pub type AnyIdWithNewOwner<T> = (AnyIdOf<T>, <T as frame_system::Config>::AccountId);

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Next available IPS ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ips_id)]
    pub type NextIpId<T: Config> = StorageValue<_, T::IpId, ValueQuery>;

    /// Store IPS info. Core IP Set storage
    ///
    /// Return `None` if IPS info not set or removed
    #[pallet::storage]
    #[pallet::getter(fn ips_storage)]
    pub type IpStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpId, IpInfoOf<T>>;

    /// IPS existence check by owner and IPS ID
    #[pallet::storage]
    #[pallet::getter(fn ips_by_owner)]
    pub type IpsByOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // owner
        Blake2_128Concat,
        T::IpId,
        (),
    >;

    /// Details of a multisig call. Only holds data for calls while they are in the voting stage.
    ///
    /// Key: (IP Set ID, call hash)
    #[pallet::storage]
    #[pallet::getter(fn multisig)]
    pub type Multisig<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::IpId, [u8; 32]), crate::ipt::MultisigOperationOf<T>>;

    /// Details of a sub token.
    ///
    /// Key: (IP Set ID, sub token ID)
    #[pallet::storage]
    #[pallet::getter(fn sub_assets)]
    pub type SubAssets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::IpId,
        Blake2_128Concat,
        T::IpId,
        SubIptInfo<T::IpId, BoundedVec<u8, T::MaxMetadata>>,
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
        (T::IpId, Option<T::IpId>),
        Blake2_128Concat,
        T::AccountId,
        <T as pallet::Config>::Balance,
    >;

    /// Sub asset voting weight (non IPT0).
    ///
    /// Key: (IP Set ID, sub token ID)
    #[pallet::storage]
    #[pallet::getter(fn asset_weight_storage)]
    pub type AssetWeight<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::IpId, Blake2_128Concat, T::IpId, OneOrPercent>;

    // StorageDoubleMap Store wasm function, input = call/pallet id + arguments, output = boolean
    // (T::IpId, T::IpId), [u8; 2] -> Wasm function or simple boolean via BoolOrWasm enum

    pub use primitives::BoolOrWasm as BOW;

    pub type BoolOrWasm<T> = BOW<BoundedVec<u8, <T as Config>::MaxWasmPermissionBytes>>;

    /// Store WASM function? What permissions does a sub token have?
    ///
    /// Key: (Ip Set ID sub token ID arguments), call metadata
    #[pallet::storage]
    #[pallet::getter(fn permissions)]
    pub type Permissions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::IpId, T::IpId),
        Blake2_128Concat,
        [u8; 2],
        BoolOrWasm<T>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An IP Set was created
        Created(T::AccountId, T::IpId),
        /// An IP Set was destroyed/deleted
        Destroyed(T::AccountId, T::IpId),
        /// IpInfo (IPS) struct updated in storage to hold either new assets, new metadata, or both
        Appended(T::AccountId, T::IpId, Vec<u8>, Vec<AnyIdOf<T>>),
        /// IpInfo (IPS) struct updated: assets removed from IPS. Optionally, new metadata set
        Removed(T::AccountId, T::IpId, Vec<u8>, Vec<AnyIdWithNewOwner<T>>),
        /// Replicas of this IP Set are now allowed
        AllowedReplica(T::IpId),
        /// Replicas of this IP Set are no longer allowed
        DisallowedReplica(T::IpId),
        /// A replica of this IP Set was created
        ReplicaCreated(T::AccountId, T::IpId, T::IpId),

        /// Sub tokens were minted
        Minted(
            (T::IpId, Option<T::IpId>),
            T::AccountId,
            <T as pallet::Config>::Balance,
        ),
        /// Sub tokens were burned
        Burned(
            (T::IpId, Option<T::IpId>),
            T::AccountId,
            <T as pallet::Config>::Balance,
        ),
        /// A vote to execute a call has begun. The call needs more votes to pass.
        /// 
        /// Params: caller derived account ID, caller weighted balance, IPT0 token supply, the call hash, the `Call`
        MultisigVoteStarted(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            crate::ipt::OpaqueCall<T>,
        ),
        /// Voting weight was added towards the vote threshold, but not enough to execute the `Call`
        ///
        /// Params: caller derived account ID, caller weighted balance, IPT0 token supply, the call hash, the `Call`
        MultisigVoteAdded(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            crate::ipt::OpaqueCall<T>,
        ),
        MultisigVoteWithdrawn(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            crate::ipt::OpaqueCall<T>,
        ),
        /// Multisig call was executed.
        /// 
        /// Params: caller derived account ID, OpaqueCall, dispatch result is ok
        MultisigExecuted(T::AccountId, crate::ipt::OpaqueCall<T>, bool),
        /// The vote on a multisig call was cancelled/withdrawn
        /// 
        /// Params: caller derived account ID, the call hash
        MultisigCanceled(T::AccountId, [u8; 32]),
        /// One of more sub tokens were created
        SubAssetCreated(Vec<(T::IpId, T::IpId)>),
        PermissionSet(T::IpId, T::IpId, [u8; 2], BoolOrWasm<T>),
        WeightSet(T::IpId, T::IpId, OneOrPercent),
    }

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IP ID
        NoAvailableIpId,
        /// IPF (IpId, IpfId) not found
        IpfNotFound,
        /// IPS not found
        IpsNotFound,
        /// The operator has no permission
        /// Ex: Attempting to add a file owned by another account to your IP set
        NoPermission,
        /// The IPS is already owned
        AlreadyOwned,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Can not destroy IPS
        CannotDestroyIps,
        /// IPS is not a parent IPS
        NotParent,
        /// Replicas cannot allow themselves to be replicable
        ReplicaCannotAllowReplicas,
        /// Value Not Changed
        ValueNotChanged,
        /// Replicas of this IPS are not allowed
        ReplicaNotAllowed,

        /// IP not found
        IpDoesntExist,
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

        /// IPS inside of another IPS is disabled temporarily
        IpsInsideIpsDisabled,
        /// Wasm IPL Permissions are disabled temporarily
        WasmPermissionsDisabled,
        /// Multisig is not allowed to call these extrinsics
        CantExecuteThisCall,

        InvalidWasmPermission,
        WasmPermissionFailedExecution,

        /// Division by 0 happened somewhere, maybe you have IPT assets with no decimal points?
        DivisionByZero,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create IP (Intellectual Property) Set (IPS)
        #[pallet::weight(900_000_000)]
        pub fn create_ips(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            assets: Vec<AnyIdOf<T>>,
            allow_replica: bool,
            ipl_license: <T as Config>::Licenses,
            ipl_execution_threshold: OneOrPercent,
            ipl_default_asset_weight: OneOrPercent,
            ipl_default_permission: bool,
        ) -> DispatchResult {
            Pallet::<T>::inner_create_ips(
                owner,
                metadata,
                assets,
                allow_replica,
                ipl_license,
                ipl_execution_threshold,
                ipl_default_asset_weight,
                ipl_default_permission,
            )
        }

        // /// Delete an IP Set and all of its contents
        // #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        // pub fn destroy(owner: OriginFor<T>, ips_id: T::IpId) -> DispatchResult {
        //     IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
        //         let owner = ensure_signed(owner)?;
        //         let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

        //         match info.parentage {
        //             Parentage::Parent(ips_account) => {
        //                 ensure!(ips_account == owner, Error::<T>::NoPermission)
        //             }
        //             Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        //         }

        //         IpsByOwner::<T>::remove(owner.clone(), ips_id);

        //         Self::deposit_event(Event::Destroyed(owner, ips_id));

        //         Ok(())
        //     })
        // }
        // TODO: Rewrite

        /// Append new assets to an IP Set
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn append(
            owner: OriginFor<T>,
            ips_id: T::IpId,
            assets: Vec<AnyIdOf<T>>,
            new_metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            Pallet::<T>::inner_append(owner, ips_id, assets, new_metadata)
        }

        /// Remove assets from an IP Set
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn remove(
            owner: OriginFor<T>,
            ips_id: T::IpId,
            assets: Vec<AnyIdWithNewOwner<T>>,
            new_metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            Pallet::<T>::inner_remove(owner, ips_id, assets, new_metadata)
        }

        /// Allows replicas of this IPS to be made.
        #[pallet::weight(200_000_000)]
        pub fn allow_replica(owner: OriginFor<T>, ips_id: T::IpId) -> DispatchResult {
            Pallet::<T>::inner_allow_replica(owner, ips_id)
        }

        /// Disallows replicas of this IPS to be made.
        #[pallet::weight(200_000_000)]
        pub fn disallow_replica(owner: OriginFor<T>, ips_id: T::IpId) -> DispatchResult {
            Pallet::<T>::inner_disallow_replica(owner, ips_id)
        }

        // #[pallet::weight(100_000)]
        // pub fn create_replica(
        //     owner: OriginFor<T>,
        //     original_ips_id: T::IpId,
        //     ipl_license: <T as Config>::Licenses,
        //     ipl_execution_threshold: OneOrPercent,
        //     ipl_default_asset_weight: OneOrPercent,
        //     ipl_default_permission: bool,
        // ) -> DispatchResultWithPostInfo {
        //     Pallet::<T>::inner_create_replica(
        //         owner,
        //         original_ips_id,
        //         ipl_license,
        //         ipl_execution_threshold,
        //         ipl_default_asset_weight,
        //         ipl_default_permission,
        //     )
        // }

        /// Mint `amount` of specified token to `target` account
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn ipt_mint(
            owner: OriginFor<T>,
            ipt_id: (T::IpId, Option<T::IpId>),
            amount: <T as pallet::Config>::Balance,
            target: T::AccountId,
        ) -> DispatchResult {
            Pallet::<T>::inner_ipt_mint(owner, ipt_id, amount, target)
        }

        /// Burn `amount` of specified token from `target` account
        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn ipt_burn(
            owner: OriginFor<T>,
            ipt_id: (T::IpId, Option<T::IpId>),
            amount: <T as pallet::Config>::Balance,
            target: T::AccountId,
        ) -> DispatchResult {
            Pallet::<T>::inner_ipt_burn(owner, ipt_id, amount, target)
        }

        #[pallet::weight(350_000_000)]
        pub fn operate_multisig(
            caller: OriginFor<T>,
            include_caller: bool,
            ipt_id: (T::IpId, Option<T::IpId>),
            call: Box<<T as pallet::Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_operate_multisig(caller, include_caller, ipt_id, call)
        }

        #[pallet::weight(350_000_000)]
        pub fn vote_multisig(
            caller: OriginFor<T>,
            ipt_id: (T::IpId, Option<T::IpId>),
            call_hash: [u8; 32],
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_vote_multisig(caller, ipt_id, call_hash)
        }

        #[pallet::weight(250_000_000)]
        pub fn withdraw_vote_multisig(
            caller: OriginFor<T>,
            ipt_id: (T::IpId, Option<T::IpId>),
            call_hash: [u8; 32],
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_withdraw_vote_multisig(caller, ipt_id, call_hash)
        }

        /// Create one or more sub tokens for an IP Set
        #[pallet::weight(200_000_000)]
        pub fn create_sub_asset(
            caller: OriginFor<T>,
            ipt_id: T::IpId,
            sub_assets: crate::ipt::SubAssetsWithEndowment<T>,
        ) -> DispatchResultWithPostInfo {
            Pallet::<T>::inner_create_sub_asset(caller, ipt_id, sub_assets)
        }

        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn set_permission(
            owner: OriginFor<T>,
            ipl_id: T::IpId,
            sub_asset: T::IpId,
            call_metadata: [u8; 2],
            permission: BoolOrWasm<T>,
        ) -> DispatchResult {
            Pallet::<T>::inner_set_permission(owner, ipl_id, sub_asset, call_metadata, permission)
        }

        #[pallet::weight(200_000_000)] // TODO: Set correct weight
        pub fn set_asset_weight(
            owner: OriginFor<T>,
            ipl_id: T::IpId,
            sub_asset: T::IpId,
            asset_weight: OneOrPercent,
        ) -> DispatchResult {
            Pallet::<T>::inner_set_asset_weight(owner, ipl_id, sub_asset, asset_weight)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
