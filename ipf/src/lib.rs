//! # Pallet IPF
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

use frame_support::{ensure, traits::Get, BoundedVec, Parameter};
use frame_system::ensure_signed;
use frame_system::pallet_prelude::OriginFor;
use primitives::IpfInfo;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, Member, One};
use sp_std::{convert::TryInto, vec::Vec};

//#[cfg(test)]
//mod mock;
//#[cfg(test)]
//mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The IPF Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The IPF ID type
        type IpfId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        /// The maximum size of an IPF's metadata
        type MaxIpfMetadata: Get<u32>;
    }

    pub type IpfMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpfMetadata>;
    pub type IpfInfoOf<T> = IpfInfo<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::Hash, // CID stored as just the hash
        IpfMetadataOf<T>,
    >;

    pub type GenesisIpfData<T> = (
        <T as frame_system::Config>::AccountId, // IPF owner
        Vec<u8>,                                // IPF metadata
        <T as frame_system::Config>::Hash,      // CID stored as just the hash
    );

    /// Next available IPF ID
    #[pallet::storage]
    #[pallet::getter(fn next_ipf_id)]
    pub type NextIpfId<T: Config> = StorageValue<_, T::IpfId, ValueQuery>;

    /// Store IPF info
    ///
    /// Returns `None` if IPF info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ipf_storage)]
    pub type IpfStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpfId, IpfInfoOf<T>>;

    /// IPF existence check by owner and IPF ID
    #[pallet::storage]
    #[pallet::getter(fn ipf_by_owner)]
    pub type IpfByOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // owner
        Blake2_128Concat,
        T::IpfId,
        (),
    >;

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPF ID
        NoAvailableIpfId,
        /// IPF (IpsId, IpfId) not found
        IpfNotFound,
        /// The operator is not the owner of the IPF and has no permission
        NoPermission,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Tried to amend an IPF without any changes
        AmendWithoutChanging,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    //#[pallet::metadata(T::AccountId = "AccountId", T::IpfId = "IpfId", T::Hash = "Hash")]
    pub enum Event<T: Config> {
        Minted(T::AccountId, T::IpfId, T::Hash),
        Amended(T::AccountId, T::IpfId, T::Hash),
        Burned(T::AccountId, T::IpfId),
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint IPF(Intellectual Property Token) to `owner`
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn mint(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            data: T::Hash,
        ) -> DispatchResultWithPostInfo {
            NextIpfId::<T>::try_mutate(|id| -> DispatchResultWithPostInfo {
                let owner = ensure_signed(owner)?;
                let bounded_metadata: BoundedVec<u8, T::MaxIpfMetadata> = metadata
                    .try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

                let ipf_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableIpfId)?;

                let ipf_info = IpfInfo {
                    metadata: bounded_metadata,
                    owner: owner.clone(),
                    data,
                };
                IpfStorage::<T>::insert(ipf_id, ipf_info);
                IpfByOwner::<T>::insert(owner.clone(), ipf_id, ());

                Self::deposit_event(Event::Minted(owner, ipf_id, data));

                Ok(().into())
            })
        }

        /// Burn IPF(Intellectual Property Token) from `owner`
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn burn(owner: OriginFor<T>, ipf_id: T::IpfId) -> DispatchResult {
            IpfStorage::<T>::try_mutate(ipf_id, |ipf_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;
                let t = ipf_info.take().ok_or(Error::<T>::IpfNotFound)?;
                ensure!(t.owner == owner, Error::<T>::NoPermission);

                IpfByOwner::<T>::remove(owner.clone(), ipf_id);

                Self::deposit_event(Event::Burned(owner, ipf_id));

                Ok(())
            })
        }

        /// Amend the data stored inside an IP Token
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn amend(
            owner: OriginFor<T>,
            ipf_id: T::IpfId,
            new_metadata: Vec<u8>,
            data: T::Hash,
        ) -> DispatchResultWithPostInfo {
            IpfStorage::<T>::try_mutate(ipf_id, |ipf_info| -> DispatchResultWithPostInfo {
                let owner = ensure_signed(owner)?;
                let ipf = ipf_info.clone().ok_or(Error::<T>::IpfNotFound)?;
                ensure!(ipf.owner == owner, Error::<T>::NoPermission);
                let bounded_metadata: BoundedVec<u8, T::MaxIpfMetadata> =
                    new_metadata
                        .try_into()
                        .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

                ensure!(
                    ((ipf.metadata != bounded_metadata) || (ipf.data != data)),
                    Error::<T>::AmendWithoutChanging
                );

                ipf_info.replace(IpfInfo {
                    metadata: bounded_metadata,
                    owner: owner.clone(),
                    data,
                });

                Self::deposit_event(Event::Amended(owner, ipf_id, data));

                Ok(().into())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
