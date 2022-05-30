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
    dispatch::{Dispatchable, RawOrigin},
    pallet_prelude::*,
    traits::{Currency as FSCurrency, Get, GetCallMetadata, WrapperKeepOpaque},
    weights::{GetDispatchInfo, PostDispatchInfo, WeightToFeePolynomial},
    BoundedVec, Parameter,
};
use frame_system::pallet_prelude::*;
use sp_arithmetic::traits::Zero;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Member, One};
use sp_std::{convert::TryInto, vec::Vec};

/// Import the primitives crate
use primitives::IpInfo;

pub use pallet::*;

pub trait LicenseList {
    type IpfsHash: core::hash::Hash;
    type MaxLicenseMetadata;

    fn get_hash_and_metadata(&self) -> (BoundedVec<u8, Self::MaxLicenseMetadata>, Self::IpfsHash);
}

type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::Call>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct MultisigOperation<AccountId, Signers, Call> {
    signers: Signers,
    include_original_caller: bool,
    original_caller: AccountId,
    actual_call: Call,
    call_metadata: [u8; 2],
    call_weight: Weight,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use primitives::utils::multi_account_id;
    use primitives::{AnyId, IpsType, OneOrPercent, Parentage, SubIptInfo};
    use rmrk_traits::Nft;
    use scale_info::prelude::fmt::Display;
    use sp_runtime::traits::StaticLookup;
    use sp_std::iter::Sum;
    use sp_std::vec;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + ipf::Config
        + pallet_balances::Config
        + pallet_rmrk_core::Config
        + pallet_uniques::Config<
            ClassId = rmrk_traits::primitives::CollectionId,
            InstanceId = rmrk_traits::primitives::NftId,
        >
    {
        /// The IPS Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The IPS ID type
        type IpsId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen;

        /// The maximum size of an IPS's metadata
        type MaxIpsMetadata: Get<u32>;
        /// Currency
        type Currency: FSCurrency<Self::AccountId>;

        type IpsData: IntoIterator + Clone;

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
            + IsType<
                <<Self as pallet::Config>::WeightToFeePolynomial as WeightToFeePolynomial>::Balance,
            >;

        #[pallet::constant]
        type ExistentialDeposit: Get<<Self as pallet::Config>::Balance>;

        type Licenses: Parameter
            + LicenseList<
                IpfsHash = <Self as frame_system::Config>::Hash,
                MaxLicenseMetadata = <Self as Config>::MaxLicenseMetadata,
            >;

        #[pallet::constant]
        type MaxLicenseMetadata: Get<u32>;

        /// The overarching call type.
        type Call: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + GetCallMetadata
            + Encode;

        type WeightToFeePolynomial: WeightToFeePolynomial;

        /// The maximum numbers of caller accounts on a single Multisig call
        #[pallet::constant]
        type MaxCallers: Get<u32>;

        #[pallet::constant]
        type MaxSubAssets: Get<u32>;

        #[pallet::constant]
        type MaxIptMetadata: Get<u32>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as FSCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type IpsIndexOf<T> = <T as Config>::IpsId;

    pub type IpsMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpsMetadata>;

    pub type IpInfoOf<T> = IpInfo<
        <T as frame_system::Config>::AccountId,
        BoundedVec<
            AnyId<
                <T as Config>::IpsId,
                (
                    rmrk_traits::primitives::CollectionId,
                    rmrk_traits::primitives::NftId,
                ),
            >,
            <T as Config>::MaxIpsMetadata,
        >,
        IpsMetadataOf<T>,
        <T as Config>::IpsId,
        <T as Config>::Balance,
        BoundedVec<u8, <T as Config>::MaxLicenseMetadata>,
        <T as frame_system::Config>::Hash,
    >;

    pub type GenesisIps<T> = (
        <T as frame_system::Config>::AccountId, // IPS owner
        Vec<u8>,                                // IPS metadata
        BoundedVec<
            AnyId<<T as Config>::IpsId, <T as ipf::Config>::IpfId>,
            <T as Config>::MaxIpsMetadata,
        >, // IPS data
        Vec<ipf::GenesisIpfData<T>>,            // Vector of IPFs belong to this IPS
    );

    pub type AnyIdWithNewOwner<T> = (
        AnyId<
            <T as pallet::Config>::IpsId,
            (
                rmrk_traits::primitives::CollectionId,
                rmrk_traits::primitives::NftId,
            ),
        >,
        <T as frame_system::Config>::AccountId,
    );

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Next available IPS ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ips_id)]
    pub type NextIpsId<T: Config> = StorageValue<_, T::IpsId, ValueQuery>;

    /// Store IPS info
    ///
    /// Return `None` if IPS info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ips_storage)]
    pub type IpStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpsId, IpInfoOf<T>>;

    /// IPS existence check by owner and IPS ID
    #[pallet::storage]
    #[pallet::getter(fn ips_by_owner)]
    pub type IpsByOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // owner
        Blake2_128Concat,
        T::IpsId,
        (),
    >;

    #[pallet::storage]
    #[pallet::getter(fn multisig)]
    /// Details of a multisig call.
    pub type Multisig<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::IpsId, [u8; 32]), MultisigOperationOf<T>>;

    pub type MultisigOperationOf<T> = MultisigOperation<
        <T as frame_system::Config>::AccountId,
        BoundedVec<
            (
                <T as frame_system::Config>::AccountId,
                Option<<T as pallet::Config>::IpsId>,
            ),
            <T as Config>::MaxCallers,
        >,
        OpaqueCall<T>,
    >;

    type SubAssetsWithEndowment<T> = Vec<(
        SubIptInfo<
            <T as pallet::Config>::IpsId,
            BoundedVec<u8, <T as pallet::Config>::MaxIpsMetadata>,
        >,
        (
            <T as frame_system::Config>::AccountId,
            <T as pallet::Config>::Balance,
        ),
    )>;

    #[pallet::storage]
    #[pallet::getter(fn sub_assets)]
    /// Details of a sub asset.
    pub type SubAssets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::IpsId,
        Blake2_128Concat,
        T::IpsId,
        SubIptInfo<T::IpsId, BoundedVec<u8, T::MaxIpsMetadata>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn balance)]
    /// The holdings of a specific account for a specific asset.
    pub type Balance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::IpsId, Option<T::IpsId>),
        Blake2_128Concat,
        T::AccountId,
        <T as pallet::Config>::Balance,
    >;

    #[pallet::storage]
    #[pallet::getter(fn asset_weight_storage)]
    /// Details of a multisig call.
    pub type AssetWeight<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::IpsId, Blake2_128Concat, T::IpsId, OneOrPercent>;

    #[pallet::storage]
    #[pallet::getter(fn permissions)]
    /// Details of a multisig call.
    pub type Permissions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::IpsId, T::IpsId),
        Blake2_128Concat,
        [u8; 2],
        bool,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        Created(T::AccountId, T::IpsId),
        Destroyed(T::AccountId, T::IpsId),
        Appended(
            T::AccountId,
            T::IpsId,
            Vec<u8>,
            Vec<
                AnyId<
                    T::IpsId,
                    (
                        rmrk_traits::primitives::CollectionId,
                        rmrk_traits::primitives::NftId,
                    ),
                >,
            >,
        ),
        Removed(T::AccountId, T::IpsId, Vec<u8>, Vec<AnyIdWithNewOwner<T>>),
        AllowedReplica(T::IpsId),
        DisallowedReplica(T::IpsId),
        ReplicaCreated(T::AccountId, T::IpsId, T::IpsId),

        Minted(
            (T::IpsId, Option<T::IpsId>),
            T::AccountId,
            <T as pallet::Config>::Balance,
        ),
        Burned(
            (T::IpsId, Option<T::IpsId>),
            T::AccountId,
            <T as pallet::Config>::Balance,
        ),
        MultisigVoteStarted(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            OpaqueCall<T>,
        ),
        MultisigVoteAdded(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            OpaqueCall<T>,
        ),
        MultisigVoteWithdrawn(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            OpaqueCall<T>,
        ),
        MultisigExecuted(T::AccountId, OpaqueCall<T>, bool),
        MultisigCanceled(T::AccountId, [u8; 32]),
        SubAssetCreated(Vec<(T::IpsId, T::IpsId)>),
    }

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPS ID
        NoAvailableIpsId,
        /// No available IPF ID
        NoAvailableIpfId,
        /// IPF (IpsId, IpfId) not found
        IpfNotFound,
        /// IPS not found
        IpsNotFound,
        /// The operator is not the owner of the IPF and has no permission
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

        IpDoesntExist,
        NotEnoughAmount,
        TooManySignatories,
        UnexistentBalance,
        MultisigOperationUninitialized,
        CouldntDecodeCall,
        MultisigOperationAlreadyExists,
        NotAVoter,
        UnknownError,
        SubAssetNotFound,
        SubAssetAlreadyExists,
        TooManySubAssets,
        SubAssetHasNoPermission,
        IplDoesntExist,
        FailedDivision,
        CallHasTooFewBytes,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create IP (Intellectual Property) Set (IPS)
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn create_ips(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            data: Vec<(
                rmrk_traits::primitives::CollectionId,
                rmrk_traits::primitives::NftId,
            )>,
            allow_replica: bool,
            ipl_license: <T as Config>::Licenses,
            ipl_execution_threshold: OneOrPercent,
            ipl_default_asset_weight: OneOrPercent,
            ipl_default_permission: bool,
        ) -> DispatchResultWithPostInfo {
            NextIpsId::<T>::try_mutate(|ips_id| -> DispatchResultWithPostInfo {
                let creator = ensure_signed(owner.clone())?;

                let bounded_metadata: BoundedVec<u8, T::MaxIpsMetadata> = metadata
                    .try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

                let current_id = *ips_id;
                *ips_id = ips_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableIpsId)?;

                ensure!(
                    !data.clone().into_iter().any(|ipf_id| {
                        pallet_rmrk_core::NftsByOwner::<T>::get(creator.clone())
                            .unwrap()
                            .into_iter()
                            .find(|nft| *nft == ipf_id)
                            .is_none()
                    }),
                    Error::<T>::NoPermission
                );

                let ips_account = primitives::utils::multi_account_id::<T, <T as Config>::IpsId>(
                    current_id, None,
                );

                for ipf in data.clone() {
                    pallet_rmrk_core::Pallet::<T>::nft_send(
                        creator.clone(),
                        ipf.0,
                        ipf.1,
                        rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(ips_account.clone()),
                    )?;
                }

                pallet_balances::Pallet::<T>::transfer_keep_alive(
                    owner.clone(),
                    T::Lookup::unlookup(ips_account.clone()),
                    <T as pallet_balances::Config>::ExistentialDeposit::get(),
                )?;

                let info = IpInfo {
                    parentage: Parentage::Parent(ips_account.clone()),
                    metadata: bounded_metadata,
                    data: data
                        .into_iter()
                        .map(AnyId::IpfId)
                        .collect::<Vec<
                            AnyId<
                                <T as Config>::IpsId,
                                (
                                    rmrk_traits::primitives::CollectionId,
                                    rmrk_traits::primitives::NftId,
                                ),
                            >,
                        >>()
                        .try_into()
                        .unwrap(),
                    ips_type: IpsType::Normal,
                    allow_replica,

                    supply: Zero::zero(),

                    license: ipl_license.get_hash_and_metadata(),
                    execution_threshold: ipl_execution_threshold,
                    default_asset_weight: ipl_default_asset_weight,
                    default_permission: ipl_default_permission,
                };

                IpStorage::<T>::insert(current_id, info);
                IpsByOwner::<T>::insert(ips_account.clone(), current_id, ());

                Self::deposit_event(Event::Created(ips_account, current_id));

                Ok(().into())
            })
        }

        /// Delete an IP Set and all of its contents
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn destroy(owner: OriginFor<T>, ips_id: T::IpsId) -> DispatchResult {
            IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                match info.parentage {
                    Parentage::Parent(ips_account) => {
                        ensure!(ips_account == owner, Error::<T>::NoPermission)
                    }
                    Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
                }

                IpsByOwner::<T>::remove(owner.clone(), ips_id);

                // TODO: Destroy IPT.

                Self::deposit_event(Event::Destroyed(owner, ips_id));

                Ok(())
            })
        }

        /// Append new assets to an IP Set
        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn append(
            owner: OriginFor<T>,
            ips_id: T::IpsId,
            assets: Vec<
                AnyId<
                    T::IpsId,
                    (
                        rmrk_traits::primitives::CollectionId,
                        rmrk_traits::primitives::NftId,
                    ),
                >,
            >,
            new_metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let caller_account = ensure_signed(owner.clone())?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                let parent_id = ips_id;

                let ips_account = match info.parentage.clone() {
                    Parentage::Parent(ips_account) => ips_account,
                    Parentage::Child(_, absolute_parent_account) => absolute_parent_account,
                };

                ensure!(
                    !assets.is_empty() || new_metadata.is_some(),
                    Error::<T>::ValueNotChanged
                );

                for asset in assets.clone() {
                    match asset {
                        AnyId::IpsId(ips_id) => {
                            if let Parentage::Parent(acc) = IpStorage::<T>::get(ips_id)
                                .ok_or(Error::<T>::IpsNotFound)?
                                .parentage
                            {
                                ensure!(
                                    caller_account
                                        == multi_account_id::<T, T::IpsId>(parent_id, Some(acc)),
                                    Error::<T>::NoPermission
                                );
                            } else {
                                return Err(Error::<T>::NotParent.into());
                            }
                        }
                        AnyId::IpfId(ipf_id) => {
                            let this_ipf_owner =
                                pallet_rmrk_core::Nfts::<T>::get(ipf_id.0, ipf_id.1)
                                    .ok_or(Error::<T>::IpfNotFound)?
                                    .owner;
                            ensure!(
                                this_ipf_owner.clone() == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(ips_account.clone())
                                    || if let rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(a) = pallet_rmrk_core::Nfts::<T>::get(ipf_id.0, ipf_id.1).ok_or(Error::<T>::IpfNotFound)?.owner {
                                    caller_account
                                        == multi_account_id::<T, T::IpsId>(
                                            parent_id,
                                            Some(a)
                                        )} else {false},
                                Error::<T>::NoPermission
                            );

                            if let rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(acc) =
                                this_ipf_owner
                            {
                                pallet_rmrk_core::Pallet::<T>::nft_send(
                                    acc,
                                    ipf_id.0,
                                    ipf_id.1,
                                    rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                                        ips_account.clone(),
                                    ),
                                )?;
                            } else {
                                panic!()
                            }
                        }
                    }
                }

                for any_id in assets.clone().into_iter() {
                    if let AnyId::IpsId(ips_id) = any_id {
                        IpStorage::<T>::try_mutate_exists(ips_id, |ips| -> DispatchResult {
                            let old = ips.take().ok_or(Error::<T>::IpsNotFound)?;

                            let prefix: (<T as Config>::IpsId, Option<<T as Config>::IpsId>) =
                                (ips_id.into(), None);
                            for (account, amount) in Balance::<T>::iter_prefix(prefix) {
                                let id: (<T as Config>::IpsId, Option<<T as Config>::IpsId>) =
                                    (parent_id.into(), None);
                                Pallet::<T>::internal_mint(id, account.clone(), amount)?;
                                Pallet::<T>::internal_burn(account, prefix, amount)?;
                            }

                            *ips = Some(IpInfo {
                                parentage: Parentage::Child(parent_id, ips_account.clone()),
                                metadata: old.metadata,
                                data: old.data,
                                ips_type: old.ips_type,
                                allow_replica: old.allow_replica,

                                supply: old.supply,

                                license: old.license,
                                execution_threshold: old.execution_threshold,
                                default_asset_weight: old.default_asset_weight,
                                default_permission: old.default_permission,
                            });

                            Ok(())
                        })?;
                    }
                }

                *ips_info = Some(IpInfo {
                    parentage: info.parentage,
                    metadata: if let Some(metadata) = new_metadata.clone() {
                        metadata
                            .try_into()
                            .map_err(|_| Error::<T>::MaxMetadataExceeded)?
                    } else {
                        info.metadata.clone()
                    },
                    data: info
                        .data
                        .into_iter()
                        .chain(assets.clone().into_iter())
                        .collect::<Vec<
                            AnyId<
                                <T as Config>::IpsId,
                                (
                                    rmrk_traits::primitives::CollectionId,
                                    rmrk_traits::primitives::NftId,
                                ),
                            >,
                        >>()
                        .try_into()
                        .unwrap(), // TODO: Remove unwrap.
                    ips_type: info.ips_type,
                    allow_replica: info.allow_replica,

                    supply: info.supply,

                    license: info.license,
                    execution_threshold: info.execution_threshold,
                    default_asset_weight: info.default_asset_weight,
                    default_permission: info.default_permission,
                });

                Self::deposit_event(Event::Appended(
                    caller_account,
                    ips_id,
                    if let Some(metadata) = new_metadata {
                        metadata
                    } else {
                        info.metadata.to_vec()
                    },
                    assets,
                ));

                Ok(())
            })
        }

        /// Remove assets from an IP Set
        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn remove(
            owner: OriginFor<T>,
            ips_id: T::IpsId,
            assets: Vec<AnyIdWithNewOwner<T>>,
            new_metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let caller_account = ensure_signed(owner.clone())?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                let ips_account = match info.parentage.clone() {
                    Parentage::Parent(ips_account) => ips_account,
                    Parentage::Child(_, absolute_parent_account) => absolute_parent_account,
                };

                ensure!(ips_account == caller_account, Error::<T>::NoPermission);

                ensure!(
                    !assets
                        .clone()
                        .into_iter()
                        .any(|id| { !info.data.contains(&id.0) }),
                    Error::<T>::NoPermission
                );

                let mut old_assets = info.data.clone();

                for any_id in assets.clone().into_iter() {
                    match any_id {
                        (AnyId::IpsId(this_ips_id), new_owner) => {
                            IpStorage::<T>::try_mutate_exists(
                                this_ips_id,
                                |ips| -> DispatchResult {
                                    let id: (<T as Config>::IpsId, Option<<T as Config>::IpsId>) =
                                        (this_ips_id.into(), None);
                                    Pallet::<T>::internal_mint(
                                        id,
                                        new_owner,
                                        <T as Config>::ExistentialDeposit::get(),
                                    )?;

                                    ips.clone().unwrap().parentage = Parentage::Parent(
                                        multi_account_id::<T, T::IpsId>(this_ips_id, None),
                                    );

                                    Ok(())
                                },
                            )?;
                        }

                        (AnyId::IpfId(this_ipf_id), new_owner) => {
                            pallet_rmrk_core::Pallet::<T>::nft_send(
                                ips_account.clone(),
                                this_ipf_id.0,
                                this_ipf_id.1,
                                rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(new_owner),
                            )?;
                        }
                    }
                }

                let just_ids = assets.clone().into_iter().map(|(x, _)| x).collect::<Vec<
                    AnyId<
                        T::IpsId,
                        (
                            rmrk_traits::primitives::CollectionId,
                            rmrk_traits::primitives::NftId,
                        ),
                    >,
                >>();
                old_assets.retain(|x| !just_ids.clone().contains(x));

                *ips_info = Some(IpInfo {
                    parentage: info.parentage,
                    metadata: if let Some(metadata) = new_metadata.clone() {
                        metadata
                            .try_into()
                            .map_err(|_| Error::<T>::MaxMetadataExceeded)?
                    } else {
                        info.metadata.clone()
                    },
                    data: old_assets,
                    ips_type: info.ips_type,
                    allow_replica: info.allow_replica,

                    supply: info.supply,

                    license: info.license,
                    execution_threshold: info.execution_threshold,
                    default_asset_weight: info.default_asset_weight,
                    default_permission: info.default_permission,
                });

                Self::deposit_event(Event::Removed(
                    caller_account,
                    ips_id,
                    if let Some(metadata) = new_metadata {
                        metadata
                    } else {
                        info.metadata.to_vec()
                    },
                    assets,
                ));

                Ok(())
            })
        }

        /// Allows replicas of this IPS to be made.
        #[pallet::weight(100_000)]
        pub fn allow_replica(owner: OriginFor<T>, ips_id: T::IpsId) -> DispatchResult {
            IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                match info.parentage.clone() {
                    Parentage::Parent(ips_account) => {
                        ensure!(ips_account == owner, Error::<T>::NoPermission)
                    }
                    Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
                }

                ensure!(!info.allow_replica, Error::<T>::ValueNotChanged);

                ensure!(
                    !matches!(info.ips_type, IpsType::Replica(_)),
                    Error::<T>::ReplicaCannotAllowReplicas
                );

                *ips_info = Some(IpInfo {
                    parentage: info.parentage,
                    metadata: info.metadata,
                    data: info.data,
                    ips_type: info.ips_type,
                    allow_replica: true,

                    supply: info.supply,

                    license: info.license,
                    execution_threshold: info.execution_threshold,
                    default_asset_weight: info.default_asset_weight,
                    default_permission: info.default_permission,
                });

                Self::deposit_event(Event::AllowedReplica(ips_id));

                Ok(())
            })
        }

        /// Disallows replicas of this IPS to be made.
        #[pallet::weight(100_000)]
        pub fn disallow_replica(owner: OriginFor<T>, ips_id: T::IpsId) -> DispatchResult {
            IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                match info.parentage.clone() {
                    Parentage::Parent(ips_account) => {
                        ensure!(ips_account == owner, Error::<T>::NoPermission)
                    }
                    Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
                }

                ensure!(
                    !matches!(info.ips_type, IpsType::Replica(_)),
                    Error::<T>::ReplicaCannotAllowReplicas
                );

                ensure!(info.allow_replica, Error::<T>::ValueNotChanged);

                *ips_info = Some(IpInfo {
                    parentage: info.parentage,
                    metadata: info.metadata,
                    data: info.data,
                    ips_type: info.ips_type,
                    allow_replica: false,

                    supply: info.supply,

                    license: info.license,
                    execution_threshold: info.execution_threshold,
                    default_asset_weight: info.default_asset_weight,
                    default_permission: info.default_permission,
                });

                Self::deposit_event(Event::DisallowedReplica(ips_id));

                Ok(())
            })
        }

        #[pallet::weight(100_000)]
        pub fn create_replica(
            owner: OriginFor<T>,
            original_ips_id: T::IpsId,
            ipl_license: <T as Config>::Licenses,
            ipl_execution_threshold: OneOrPercent,
            ipl_default_asset_weight: OneOrPercent,
            ipl_default_permission: bool,
        ) -> DispatchResultWithPostInfo {
            NextIpsId::<T>::try_mutate(|ips_id| -> DispatchResultWithPostInfo {
                let creator = ensure_signed(owner.clone())?;

                let original_ips =
                    IpStorage::<T>::get(original_ips_id).ok_or(Error::<T>::IpsNotFound)?;

                ensure!(original_ips.allow_replica, Error::<T>::ReplicaNotAllowed);

                let current_id = *ips_id;
                *ips_id = ips_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableIpsId)?;

                let ips_account = primitives::utils::multi_account_id::<T, <T as Config>::IpsId>(
                    current_id, None,
                );

                pallet_balances::Pallet::<T>::transfer_keep_alive(
                    owner.clone(),
                    T::Lookup::unlookup(ips_account.clone()),
                    <T as pallet_balances::Config>::ExistentialDeposit::get(),
                )?;

                let info = IpInfo {
                    parentage: Parentage::Parent(ips_account.clone()),
                    metadata: original_ips.metadata,
                    data: original_ips.data,
                    ips_type: IpsType::Replica(original_ips_id),
                    allow_replica: false,

                    supply: Zero::zero(),

                    license: ipl_license.get_hash_and_metadata(),
                    execution_threshold: ipl_execution_threshold,
                    default_asset_weight: ipl_default_asset_weight,
                    default_permission: ipl_default_permission,
                };

                Pallet::<T>::internal_mint(
                    (current_id, None),
                    creator,
                    <T as Config>::ExistentialDeposit::get(),
                )?;

                IpStorage::<T>::insert(current_id, info);
                IpsByOwner::<T>::insert(ips_account.clone(), current_id, ());

                Self::deposit_event(Event::ReplicaCreated(
                    ips_account,
                    current_id,
                    original_ips_id,
                ));

                Ok(().into())
            })
        }

        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn ipt_mint(
            owner: OriginFor<T>,
            ipt_id: (T::IpsId, Option<T::IpsId>),
            amount: <T as pallet::Config>::Balance,
            target: T::AccountId,
        ) -> DispatchResult {
            let owner = ensure_signed(owner)?;

            let ip = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

            match &ip.parentage {
                Parentage::Parent(ips_account) => {
                    ensure!(ips_account == &owner, Error::<T>::NoPermission)
                }
                Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
            }

            if let Some(sub_asset) = ipt_id.1 {
                ensure!(
                    SubAssets::<T>::get(ipt_id.0, sub_asset).is_some(),
                    Error::<T>::SubAssetNotFound
                );
            }

            Pallet::<T>::internal_mint(ipt_id, target.clone(), amount)?;

            Self::deposit_event(Event::Minted(ipt_id, target, amount));

            Ok(())
        }

        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn burn(
            owner: OriginFor<T>,
            ipt_id: (T::IpsId, Option<T::IpsId>),
            amount: <T as pallet::Config>::Balance,
            target: T::AccountId,
        ) -> DispatchResult {
            let owner = ensure_signed(owner)?;

            let ip = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

            match &ip.parentage {
                Parentage::Parent(ips_account) => {
                    ensure!(ips_account == &owner, Error::<T>::NoPermission)
                }
                Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
            }

            if let Some(sub_asset) = ipt_id.1 {
                ensure!(
                    SubAssets::<T>::get(ipt_id.0, sub_asset).is_some(),
                    Error::<T>::SubAssetNotFound
                );
            }

            Pallet::<T>::internal_burn(target.clone(), ipt_id, amount)?;

            Self::deposit_event(Event::Burned(ipt_id, target, amount));

            Ok(())
        }

        #[pallet::weight(100_000)]
        pub fn operate_multisig(
            caller: OriginFor<T>,
            include_caller: bool,
            ipt_id: (T::IpsId, Option<T::IpsId>),
            call: Box<<T as pallet::Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(caller.clone())?;
            let ipt = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

            let total_issuance = ipt.supply
                + SubAssets::<T>::iter_prefix_values(ipt_id.0)
                    .map(|sub_asset| {
                        let supply = IpStorage::<T>::get(sub_asset.id)?.supply;

                        if let OneOrPercent::ZeroPoint(weight) =
                            Pallet::<T>::asset_weight(ipt_id.0, sub_asset.id)?
                        {
                            Some(weight * supply)
                        } else {
                            Some(supply)
                        }
                    })
                    .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                    .ok_or(Error::<T>::IplDoesntExist)?
                    .into_iter()
                    .sum();

            let total_per_threshold: <T as pallet::Config>::Balance =
                if let OneOrPercent::ZeroPoint(percent) =
                    Pallet::<T>::execution_threshold(ipt_id.0).ok_or(Error::<T>::IplDoesntExist)?
                {
                    percent * total_issuance
                } else {
                    total_issuance
                };

            let call_metadata: [u8; 2] = call
                .encode()
                .split_at(2)
                .0
                .try_into()
                .map_err(|_| Error::<T>::CallHasTooFewBytes)?;

            let owner_balance: <T as Config>::Balance = if let OneOrPercent::ZeroPoint(percent) = {
                if let Some(sub_asset) = ipt_id.1 {
                    ensure!(
                        Pallet::<T>::has_permission(ipt_id.0, sub_asset, call_metadata)
                            .ok_or(Error::<T>::IplDoesntExist)?,
                        Error::<T>::SubAssetHasNoPermission
                    );

                    Pallet::<T>::asset_weight(ipt_id.0, sub_asset)
                        .ok_or(Error::<T>::IplDoesntExist)?
                } else {
                    OneOrPercent::One
                }
            } {
                percent
                    * Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
            } else {
                Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
            };

            let opaque_call: OpaqueCall<T> = WrapperKeepOpaque::from_encoded(call.encode());

            let call_hash: [u8; 32] = blake2_256(&call.encode());

            ensure!(
                Multisig::<T>::get((ipt_id.0, blake2_256(&call.encode()))).is_none(),
                Error::<T>::MultisigOperationAlreadyExists
            );

            if owner_balance > total_per_threshold {
                pallet_balances::Pallet::<T>::transfer(
                    caller,
                    <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                        multi_account_id::<T, T::IpsId>(ipt_id.0, None),
                    ),
                    <T as pallet::Config>::Balance::from(T::WeightToFeePolynomial::calc(
                        &call.get_dispatch_info().weight,
                    ))
                    .into(),
                )?;

                let dispatch_result = call.dispatch(
                    RawOrigin::Signed(multi_account_id::<T, T::IpsId>(
                        ipt_id.0,
                        if include_caller {
                            Some(owner.clone())
                        } else {
                            None
                        },
                    ))
                    .into(),
                );

                Self::deposit_event(Event::MultisigExecuted(
                    multi_account_id::<T, T::IpsId>(
                        ipt_id.0,
                        if include_caller { Some(owner) } else { None },
                    ),
                    opaque_call,
                    dispatch_result.is_ok(),
                ));
            } else {
                pallet_balances::Pallet::<T>::transfer(
                    caller,
                    <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                        multi_account_id::<T, T::IpsId>(ipt_id.0, None),
                    ),
                    <T as pallet::Config>::Balance::from(
                        (T::WeightToFeePolynomial::calc(&call.get_dispatch_info().weight)
                            / total_per_threshold.into())
                            * owner_balance.into(),
                    )
                    .into(),
                )?;

                Multisig::<T>::insert(
                    (ipt_id.0, call_hash),
                    MultisigOperation {
                        signers: vec![(owner.clone(), ipt_id.1)]
                            .try_into()
                            .map_err(|_| Error::<T>::TooManySignatories)?,
                        include_original_caller: include_caller,
                        original_caller: owner.clone(),
                        actual_call: opaque_call.clone(),
                        call_metadata,
                        call_weight: call.get_dispatch_info().weight,
                    },
                );

                Self::deposit_event(Event::MultisigVoteStarted(
                    multi_account_id::<T, T::IpsId>(
                        ipt_id.0,
                        if include_caller { Some(owner) } else { None },
                    ),
                    owner_balance,
                    ipt.supply,
                    call_hash,
                    opaque_call,
                ));
            }

            Ok(().into())
        }

        #[pallet::weight(100_000)]
        pub fn vote_multisig(
            caller: OriginFor<T>,
            ipt_id: (T::IpsId, Option<T::IpsId>),
            call_hash: [u8; 32],
        ) -> DispatchResultWithPostInfo {
            Multisig::<T>::try_mutate_exists((ipt_id.0, call_hash), |data| {
                let owner = ensure_signed(caller.clone())?;

                let ipt = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

                let mut old_data = data
                    .take()
                    .ok_or(Error::<T>::MultisigOperationUninitialized)?;

                let voter_balance = if let OneOrPercent::ZeroPoint(percent) = {
                    if let Some(sub_asset) = ipt_id.1 {
                        ensure!(
                            Pallet::<T>::has_permission(
                                ipt_id.0,
                                sub_asset,
                                old_data.call_metadata
                            )
                            .ok_or(Error::<T>::IplDoesntExist)?,
                            Error::<T>::SubAssetHasNoPermission
                        );

                        Pallet::<T>::asset_weight(ipt_id.0, sub_asset)
                            .ok_or(Error::<T>::IplDoesntExist)?
                    } else {
                        OneOrPercent::One
                    }
                } {
                    percent
                        * Balance::<T>::get(ipt_id, owner.clone())
                            .ok_or(Error::<T>::NoPermission)?
                } else {
                    Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
                };

                let total_in_operation: <T as pallet::Config>::Balance = old_data
                    .signers
                    .clone()
                    .into_iter()
                    .map(|(voter, sub_asset): (T::AccountId, Option<T::IpsId>)| {
                        Balance::<T>::get((ipt_id.0, sub_asset), voter).map(|balance| {
                            if let OneOrPercent::ZeroPoint(percent) =
                                if let Some(sub_asset) = ipt_id.1 {
                                    Pallet::<T>::asset_weight(ipt_id.0, sub_asset).unwrap()
                                } else {
                                    OneOrPercent::One
                                }
                            {
                                percent * balance
                            } else {
                                balance
                            }
                        })
                    })
                    .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                    .ok_or(Error::<T>::NoPermission)?
                    .into_iter()
                    .sum();

                let total_issuance = ipt.supply
                    + SubAssets::<T>::iter_prefix_values(ipt_id.0)
                        .map(|sub_asset| {
                            let supply = IpStorage::<T>::get(sub_asset.id)?.supply;

                            if let OneOrPercent::ZeroPoint(weight) =
                                Pallet::<T>::asset_weight(ipt_id.0, sub_asset.id)?
                            {
                                Some(weight * supply)
                            } else {
                                Some(supply)
                            }
                        })
                        .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                        .ok_or(Error::<T>::IplDoesntExist)?
                        .into_iter()
                        .sum();

                let total_per_threshold: <T as pallet::Config>::Balance =
                    if let OneOrPercent::ZeroPoint(percent) =
                        Pallet::<T>::execution_threshold(ipt_id.0)
                            .ok_or(Error::<T>::IplDoesntExist)?
                    {
                        percent * total_issuance
                    } else {
                        total_issuance
                    };

                let fee: <T as pallet::Config>::Balance =
                    T::WeightToFeePolynomial::calc(&old_data.call_weight).into();

                if (total_in_operation + voter_balance) > total_per_threshold {
                    pallet_balances::Pallet::<T>::transfer(
                        caller,
                        <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                            multi_account_id::<T, T::IpsId>(ipt_id.0, None),
                        ),
                        // Voter will pay the remainder of the fee after subtracting the total IPTs already in the operation converted to real fee value.
                        fee.checked_sub(&(total_in_operation * (fee / total_per_threshold)))
                            .ok_or(Error::<T>::NotEnoughAmount)?
                            .into(),
                    )?;

                    *data = None;

                    let dispatch_result = old_data
                        .actual_call
                        .try_decode()
                        .ok_or(Error::<T>::CouldntDecodeCall)?
                        .dispatch(
                            RawOrigin::Signed(multi_account_id::<T, T::IpsId>(
                                ipt_id.0,
                                if old_data.include_original_caller {
                                    Some(old_data.original_caller.clone())
                                } else {
                                    None
                                },
                            ))
                            .into(),
                        );

                    Self::deposit_event(Event::MultisigExecuted(
                        multi_account_id::<T, T::IpsId>(
                            ipt_id.0,
                            if old_data.include_original_caller {
                                Some(old_data.original_caller.clone())
                            } else {
                                None
                            },
                        ),
                        old_data.actual_call,
                        dispatch_result.is_ok(),
                    ));
                } else {
                    pallet_balances::Pallet::<T>::transfer(
                        caller,
                        <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                            multi_account_id::<T, T::IpsId>(ipt_id.0, None),
                        ),
                        <T as pallet::Config>::Balance::from(
                            (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                                / total_per_threshold.into())
                                * voter_balance.into(),
                        )
                        .into(),
                    )?;

                    old_data.signers = {
                        let mut v = old_data.signers.to_vec();
                        v.push((owner, ipt_id.1));
                        v.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?
                    };
                    *data = Some(old_data.clone());

                    Self::deposit_event(Event::MultisigVoteAdded(
                        multi_account_id::<T, T::IpsId>(
                            ipt_id.0,
                            if old_data.include_original_caller {
                                Some(old_data.original_caller.clone())
                            } else {
                                None
                            },
                        ),
                        voter_balance,
                        ipt.supply,
                        call_hash,
                        old_data.actual_call,
                    ));
                }

                Ok(().into())
            })
        }

        #[pallet::weight(100_000)]
        pub fn withdraw_vote_multisig(
            caller: OriginFor<T>,
            ipt_id: (T::IpsId, Option<T::IpsId>),
            call_hash: [u8; 32],
        ) -> DispatchResultWithPostInfo {
            Multisig::<T>::try_mutate_exists((ipt_id.0, call_hash), |data| {
                let owner = ensure_signed(caller.clone())?;

                let ipt = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

                let mut old_data = data
                    .take()
                    .ok_or(Error::<T>::MultisigOperationUninitialized)?;

                ensure!(
                    old_data.signers.iter().any(|signer| signer.0 == owner),
                    Error::<T>::NotAVoter
                );

                if owner == old_data.original_caller {
                    let total_issuance = ipt.supply
                        + SubAssets::<T>::iter_prefix_values(ipt_id.0)
                            .map(|sub_asset| {
                                let supply = IpStorage::<T>::get(sub_asset.id)?.supply;

                                if let OneOrPercent::ZeroPoint(weight) =
                                    Pallet::<T>::asset_weight(ipt_id.0, sub_asset.id)?
                                {
                                    Some(weight * supply)
                                } else {
                                    Some(supply)
                                }
                            })
                            .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                            .ok_or(Error::<T>::IplDoesntExist)?
                            .into_iter()
                            .sum();

                    let total_per_threshold: <T as pallet::Config>::Balance =
                        if let OneOrPercent::ZeroPoint(percent) =
                            Pallet::<T>::execution_threshold(ipt_id.0)
                                .ok_or(Error::<T>::IplDoesntExist)?
                        {
                            percent * total_issuance
                        } else {
                            total_issuance
                        };

                    for signer in old_data.signers {
                        pallet_balances::Pallet::<T>::transfer(
                            <T as frame_system::Config>::Origin::from(RawOrigin::Signed(
                                multi_account_id::<T, T::IpsId>(ipt_id.0, None),
                            )),
                            <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                                signer.0.clone(),
                            ),
                            <T as pallet::Config>::Balance::from(
                                (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                                    / total_per_threshold.into())
                                    * Balance::<T>::get((ipt_id.0, signer.1), signer.0)
                                        .ok_or(Error::<T>::UnknownError)?
                                        .into(),
                            )
                            .into(),
                        )?;
                    }

                    *data = None;
                    Self::deposit_event(Event::MultisigCanceled(
                        multi_account_id::<T, T::IpsId>(
                            ipt_id.0,
                            if old_data.include_original_caller {
                                Some(old_data.original_caller)
                            } else {
                                None
                            },
                        ),
                        call_hash,
                    ));
                } else {
                    let voter_balance = if let OneOrPercent::ZeroPoint(percent) = {
                        if let Some(sub_asset) = ipt_id.1 {
                            Pallet::<T>::asset_weight(ipt_id.0, sub_asset)
                                .ok_or(Error::<T>::IplDoesntExist)?
                        } else {
                            OneOrPercent::One
                        }
                    } {
                        percent
                            * Balance::<T>::get(ipt_id, owner.clone())
                                .ok_or(Error::<T>::NoPermission)?
                    } else {
                        Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
                    };

                    let total_issuance = ipt.supply
                        + SubAssets::<T>::iter_prefix_values(ipt_id.0)
                            .map(|sub_asset| {
                                let supply = IpStorage::<T>::get(sub_asset.id)?.supply;

                                if let OneOrPercent::ZeroPoint(weight) =
                                    Pallet::<T>::asset_weight(ipt_id.0, sub_asset.id)?
                                {
                                    Some(weight * supply)
                                } else {
                                    Some(supply)
                                }
                            })
                            .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                            .ok_or(Error::<T>::IplDoesntExist)?
                            .into_iter()
                            .sum();

                    let total_per_threshold: <T as pallet::Config>::Balance =
                        if let OneOrPercent::ZeroPoint(percent) =
                            Pallet::<T>::execution_threshold(ipt_id.0)
                                .ok_or(Error::<T>::IplDoesntExist)?
                        {
                            percent * total_issuance
                        } else {
                            total_issuance
                        };

                    old_data.signers = old_data
                        .signers
                        .into_iter()
                        .filter(|signer| signer.0 != owner)
                        .collect::<Vec<(T::AccountId, Option<T::IpsId>)>>()
                        .try_into()
                        .map_err(|_| Error::<T>::TooManySignatories)?;

                    pallet_balances::Pallet::<T>::transfer(
                        <T as frame_system::Config>::Origin::from(RawOrigin::Signed(
                            multi_account_id::<T, T::IpsId>(ipt_id.0, None),
                        )),
                        <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(owner),
                        <T as pallet::Config>::Balance::from(
                            (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                                / total_per_threshold.into())
                                * voter_balance.into(),
                        )
                        .into(),
                    )?;

                    *data = Some(old_data.clone());

                    Self::deposit_event(Event::MultisigVoteWithdrawn(
                        multi_account_id::<T, T::IpsId>(
                            ipt_id.0,
                            if old_data.include_original_caller {
                                Some(old_data.original_caller.clone())
                            } else {
                                None
                            },
                        ),
                        voter_balance,
                        ipt.supply,
                        call_hash,
                        old_data.actual_call,
                    ));
                }

                Ok(().into())
            })
        }

        #[pallet::weight(100_000)]
        pub fn create_sub_asset(
            caller: OriginFor<T>,
            ipt_id: T::IpsId,
            sub_assets: SubAssetsWithEndowment<T>,
        ) -> DispatchResultWithPostInfo {
            IpStorage::<T>::try_mutate_exists(ipt_id, |ipt| -> DispatchResultWithPostInfo {
                let caller = ensure_signed(caller.clone())?;

                let old_ipt = ipt.clone().ok_or(Error::<T>::IpDoesntExist)?;

                match old_ipt.parentage.clone() {
                    Parentage::Parent(ips_account) => {
                        ensure!(ips_account == caller, Error::<T>::NoPermission)
                    }
                    Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
                }

                for sub in sub_assets.clone() {
                    ensure!(
                        !SubAssets::<T>::contains_key(ipt_id, sub.0.id),
                        Error::<T>::SubAssetAlreadyExists
                    );

                    SubAssets::<T>::insert(ipt_id, sub.0.id, &sub.0);

                    Balance::<T>::insert((ipt_id, Some(sub.0.id)), sub.1 .0, sub.1 .1);
                }

                Self::deposit_event(Event::SubAssetCreated(
                    sub_assets
                        .into_iter()
                        .map(|sub| (ipt_id, sub.0.id))
                        .collect(),
                ));

                Ok(().into())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn internal_mint(
            ipt_id: (T::IpsId, Option<T::IpsId>),
            target: T::AccountId,
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            IpStorage::<T>::try_mutate(ipt_id.0, |ipt| -> DispatchResult {
                Balance::<T>::try_mutate(ipt_id, target, |balance| -> DispatchResult {
                    let old_balance = balance.take().unwrap_or_default();
                    *balance = Some(old_balance + amount);

                    let mut old_ipt = ipt.take().ok_or(Error::<T>::IpDoesntExist)?;

                    if ipt_id.1.is_none() {
                        old_ipt.supply += amount;
                    }

                    *ipt = Some(old_ipt);

                    Ok(())
                })
            })
        }

        pub fn internal_burn(
            target: T::AccountId,
            ipt_id: (T::IpsId, Option<T::IpsId>),
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            IpStorage::<T>::try_mutate(ipt_id.0, |ipt| -> DispatchResult {
                Balance::<T>::try_mutate(ipt_id, target, |balance| -> DispatchResult {
                    let old_balance = balance.take().ok_or(Error::<T>::IpDoesntExist)?;
                    *balance = Some(
                        old_balance
                            .checked_sub(&amount)
                            .ok_or(Error::<T>::NotEnoughAmount)?,
                    );

                    let mut old_ipt = ipt.take().ok_or(Error::<T>::IpDoesntExist)?;

                    if ipt_id.1.is_none() {
                        old_ipt.supply = old_ipt
                            .supply
                            .checked_sub(&amount)
                            .ok_or(Error::<T>::NotEnoughAmount)?;
                    }

                    *ipt = Some(old_ipt);

                    Ok(())
                })
            })
        }

        pub fn execution_threshold(ipl_id: T::IpsId) -> Option<OneOrPercent> {
            IpStorage::<T>::get(ipl_id).map(|ipl| ipl.execution_threshold)
        }

        pub fn asset_weight(ipl_id: T::IpsId, sub_asset: T::IpsId) -> Option<OneOrPercent> {
            AssetWeight::<T>::get(ipl_id, sub_asset)
                .or_else(|| IpStorage::<T>::get(ipl_id).map(|ipl| ipl.default_asset_weight))
        }

        pub fn has_permission(
            ipl_id: T::IpsId,
            sub_asset: T::IpsId,
            call_metadata: [u8; 2],
        ) -> Option<bool> {
            Permissions::<T>::get((ipl_id, sub_asset), call_metadata)
                .or_else(|| IpStorage::<T>::get(ipl_id).map(|ipl| ipl.default_permission))
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
