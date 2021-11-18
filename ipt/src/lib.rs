//! # Pallet IPT
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
use primitives::IptInfo;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, Member, One};
use sp_std::{convert::TryInto, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The IPT Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The IPT ID type
        type IptId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The maximum size of an IPT's metadata
        type MaxIptMetadata: Get<u32>;
    }

    pub type IptMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIptMetadata>;
    pub type IptInfoOf<T> = IptInfo<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::Hash, // CID stored as just the hash
        IptMetadataOf<T>,
    >;

    pub type GenesisIptData<T> = (
        <T as frame_system::Config>::AccountId, // IPT owner
        Vec<u8>,                                // IPT metadata
        <T as frame_system::Config>::Hash,      // CID stored as just the hash
    );

    /// Next available IPT ID
    #[pallet::storage]
    #[pallet::getter(fn next_ipt_id)]
    pub type NextIptId<T: Config> = StorageValue<_, T::IptId, ValueQuery>;

    /// Store IPT info
    ///
    /// Returns `None` if IPT info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ipt_storage)]
    pub type IptStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IptId, IptInfoOf<T>>;

    /// IPT existence check by owner and IPT ID
    #[pallet::storage]
    #[pallet::getter(fn ipt_by_owner)]
    pub type IptByOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // owner
        Blake2_128Concat,
        T::IptId,
        (),
    >;

    /// Errors for IPT pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPT ID
        NoAvailableIptId,
        /// IPT (IpsId, IptId) not found
        IptNotFound,
        /// The operator is not the owner of the IPT and has no permission
        NoPermission,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Tried to amend an IPT without any changes
        AmendWithoutChanging,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    //#[pallet::metadata(T::AccountId = "AccountId", T::IptId = "IptId", T::Hash = "Hash")]
    pub enum Event<T: Config> {
        Minted(T::AccountId, T::IptId, T::Hash),
        Amended(T::AccountId, T::IptId, T::Hash),
        Burned(T::AccountId, T::IptId),
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint IPT(Intellectual Property Token) to `owner`
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn mint(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            data: T::Hash,
        ) -> DispatchResultWithPostInfo {
            NextIptId::<T>::try_mutate(|id| -> DispatchResultWithPostInfo {
                let owner = ensure_signed(owner)?;
                let bounded_metadata: BoundedVec<u8, T::MaxIptMetadata> = metadata
                    .try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

                let ipt_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableIptId)?;

                let ipt_info = IptInfo {
                    metadata: bounded_metadata,
                    owner: owner.clone(),
                    data,
                };
                IptStorage::<T>::insert(ipt_id, ipt_info);
                IptByOwner::<T>::insert(owner.clone(), ipt_id, ());

                Self::deposit_event(Event::Minted(owner, ipt_id, data));

                Ok(().into())
            })
        }

        /// Burn IPT(Intellectual Property Token) from `owner`
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn burn(owner: OriginFor<T>, ipt_id: T::IptId) -> DispatchResult {
            IptStorage::<T>::try_mutate(ipt_id, |ipt_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;
                let t = ipt_info.take().ok_or(Error::<T>::IptNotFound)?;
                ensure!(t.owner == owner, Error::<T>::NoPermission);

                IptByOwner::<T>::remove(owner.clone(), ipt_id);

                Self::deposit_event(Event::Burned(owner, ipt_id));

                Ok(())
            })
        }

        /// Amend the data stored inside an IP Token
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn amend(
            owner: OriginFor<T>,
            ipt_id: T::IptId,
            new_metadata: Vec<u8>,
            data: T::Hash,
        ) -> DispatchResultWithPostInfo {
            IptStorage::<T>::try_mutate(ipt_id, |ipt_info| -> DispatchResultWithPostInfo {
                let owner = ensure_signed(owner)?;
                let ipt = ipt_info.clone().ok_or(Error::<T>::IptNotFound)?;
                ensure!(ipt.owner == owner, Error::<T>::NoPermission);
                let bounded_metadata: BoundedVec<u8, T::MaxIptMetadata> =
                    new_metadata
                        .try_into()
                        .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

                ensure!(
                    ((ipt.metadata != bounded_metadata) || (ipt.data != data)),
                    Error::<T>::AmendWithoutChanging
                );

                ipt_info.replace(IptInfo {
                    metadata: bounded_metadata,
                    owner: owner.clone(),
                    data,
                });

                Self::deposit_event(Event::Amended(owner, ipt_id, data));

                Ok(().into())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
