//! # IPT
//! Intellectual Property Tokens
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to create and manage IP Tokens, which are components in a set.
//!
//! ### Pallet Functions
//!
//! `mint` - Create a new IP Token and add to an IP Set
//! `burn` - Burn an IP Token from an IP Set
//! `amend` - Amend the data stored inside an IP Token

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Get, BoundedVec, Parameter};
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One,
    },
    ArithmeticError, DispatchError, DispatchResult,
};
use sp_std::{convert::TryInto, vec::Vec};

/// IPS info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen)]
pub struct IpsInfo<IptId, AccountId, Data, IpsMetadataOf> {
    // TODO: WIP
    /// IPS metadata
    pub metadata: IpsMetadataOf,
    /// Total issuance for the IPS
    pub total_issuance: IptId,
    /// IPS owner
    pub owner: AccountId,
    /// IPS Properties
    pub data: Data,
}

/// IPT Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen)]
pub struct IptInfo<AccountId, Data, IptMetadataOf> {
    /// IPT owner
    pub owner: AccountId,
    /// IPT metadata
    pub metadata: IptMetadataOf,
    /// IPT data
    pub data: Data,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The IPS ID type
        type IpsId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy; // TODO: WIP
        /// The IPS properties type
        type IpsData: Parameter + Member + MaybeSerializeDeserialize; // TODO: WIP
        /// The IPT ID type
        type IptId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// IPT properties type
        type IptData: Parameter + Member + MaybeSerializeDeserialize; // TODO: WIP
        /// The maximum size of an IPS's metadata
        type MaxIpsMetadata: Get<u32>; // TODO: WIP
        /// The maximum size of an IPT's metadata
        type MaxIptMetadata: Get<u32>;
    }

    pub type IpsMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpsMetadata>;
    pub type IptMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIptMetadata>;
    pub type IpsInfoOf<T> = IpsInfo<
        <T as Config>::IptId,
        <T as frame_system::Config>::AccountId,
        <T as Config>::IpsData,
        IpsMetadataOf<T>,
    >;
    pub type IptInfoOf<T> =
        IptInfo<<T as frame_system::Config>::AccountId, <T as Config>::IptData, IptMetadataOf<T>>;

    pub type GenesisIptData<T> = (
        <T as frame_system::Config>::AccountId, // IPT owner
        Vec<u8>,                                // IPT metadata
        <T as Config>::IptData,                 // IPT data
    );
    pub type GenesisIps<T> = (
        <T as frame_system::Config>::AccountId, // IPS owner
        Vec<u8>,                                // IPS metadata
        <T as Config>::IpsData,                 // IPS data
        Vec<GenesisIptData<T>>,                 // Vector of IPTs belong to this IPS
    );

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Next available IPS ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ips_id)]
    pub type NextIpsId<T: Config> = StorageValue<_, T::IpsId, ValueQuery>;

    /// Next available IPT ID
    #[pallet::storage]
    #[pallet::getter(fn next_ipt_id)]
    pub type NextIptId<T: Config> = StorageMap<_, Blake2_128Concat, T::IpsId, T::IptId, ValueQuery>;

    /// Store IPS info
    ///
    /// Return `None` if IPS info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ips_storage)]
    pub type IpsStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpsId, IpsInfoOf<T>>;

    /// Store IPT info
    ///
    /// Returns `None` if IPT info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ipt_storage)]
    pub type IptStorage<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::IpsId, Blake2_128Concat, T::IptId, IptInfoOf<T>>;

    /// IPT existence check by owner and IPS ID
    #[pallet::storage]
    #[pallet::getter(fn ipt_by_owner)]
    pub type IptByOwner<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // owner
            NMapKey<Blake2_128Concat, T::IpsId>,
            NMapKey<Blake2_128Concat, T::IptId>,
        ),
        (),
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub ips: Vec<GenesisIps<T>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig { ips: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            self.ips.iter().for_each(|ipt_class| {
                let ips_id = Pallet::<T>::create_ips(
                    &ipt_class.0,
                    ipt_class.1.to_vec(),
                    ipt_class.2.clone(),
                )
                .expect("Create IPS cannot fail while building genesis");
                for (account_id, ipt_metadata, ipt_data) in &ipt_class.3 {
                    Pallet::<T>::mint(account_id, ips_id, ipt_metadata.to_vec(), ipt_data.clone())
                        .expect("IPT mint cannot fail during genesis");
                }
            })
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// param. [Ipt, who]
        IptStored(u32, T::AccountId),
    }

    /// Errors for IPT pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPS ID
        NoAvailableIpsId,
        /// No available IPT ID
        NoAvailableIptId,
        /// IPT (IpsId, IptId) not found
        IptNotFound,
        /// IPS not found
        IpsNotFound,
        /// The operator is not the owner of the IPT and has no permission
        NoPermission,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    /// Create IP (Intellectual Property) Set (IPS)
    pub fn create_ips(
        // TODO: WIP
        owner: &T::AccountId,
        metadata: Vec<u8>,
        data: T::IpsData,
    ) -> Result<T::IpsId, DispatchError> {
        let bounded_metadata: BoundedVec<u8, T::MaxIpsMetadata> = metadata
            .try_into()
            .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

        let ips_id = NextIpsId::<T>::try_mutate(|id| -> Result<T::IpsId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableIpsId)?;
            Ok(current_id)
        })?;

        let info = IpsInfo {
            metadata: bounded_metadata,
            total_issuance: Default::default(),
            owner: owner.clone(),
            data,
        };
        IpsStorage::<T>::insert(ips_id, info);

        Ok(ips_id)
    }

    /// Mint IPT(Intellectual Property Token) to `owner`
    pub fn mint(
        owner: &T::AccountId,
        ips_id: T::IpsId,
        metadata: Vec<u8>,
        data: T::IptData,
    ) -> Result<T::IptId, DispatchError> {
        NextIptId::<T>::try_mutate(ips_id, |id| -> Result<T::IptId, DispatchError> {
            let bounded_metadata: BoundedVec<u8, T::MaxIptMetadata> = metadata
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

            let ipt_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableIptId)?;

            IpsStorage::<T>::try_mutate(ips_id, |ips_info| -> DispatchResult {
                let info = ips_info.as_mut().ok_or(Error::<T>::IpsNotFound)?;
                info.total_issuance = info
                    .total_issuance
                    .checked_add(&One::one())
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            let ipt_info = IptInfo {
                metadata: bounded_metadata,
                owner: owner.clone(),
                data,
            };
            IptStorage::<T>::insert(ips_id, ipt_id, ipt_info);
            IptByOwner::<T>::insert((owner, ips_id, ipt_id), ());

            Ok(ipt_id)
        })
    }

    /// Burn IPT(Intellectual Property Token) from `owner`
    pub fn burn(owner: &T::AccountId, ipt: (T::IpsId, T::IptId)) -> DispatchResult {
        IptStorage::<T>::try_mutate(ipt.0, ipt.1, |ipt_info| -> DispatchResult {
            let t = ipt_info.take().ok_or(Error::<T>::IptNotFound)?;
            ensure!(t.owner == *owner, Error::<T>::NoPermission);

            IpsStorage::<T>::try_mutate(ipt.0, |ips_info| -> DispatchResult {
                let info = ips_info.as_mut().ok_or(Error::<T>::IpsNotFound)?;
                info.total_issuance = info
                    .total_issuance
                    .checked_sub(&One::one())
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            IptByOwner::<T>::remove((owner, ipt.0, ipt.1));

            Ok(())
        })
    }

    // TODO : WIP
    // - Add `amend` function
}
