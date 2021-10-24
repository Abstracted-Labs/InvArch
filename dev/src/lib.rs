//! # Pallet DEV
//! Decentralized Entrepreneurial Venture
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates an agreement between 2 or more parties to work together in order to actualize an IP Set.
//!
//! ### Pallet Functions
//!
//! `create` - Create a new DEV agreement
//! `post` - Post a DEV as joinable
//! `add` - Add a user (Account Address) to a DEV
//! `remove` - Remove a user (Account Address) from a DEV
//! `update` - Update a DEV to include a new interaction
//! `freeze` - Freeze a DEV and its metadata

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::no_effect)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, Get},
    BoundedVec, Parameter,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, Member, One};
use sp_std::{convert::TryInto, vec::Vec};

/// Import from primitives pallet
use primitives::DevInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use std::iter::{FromIterator, Sum};

    use ips::IpsByOwner;
    use sp_std::collections::btree_map::BTreeMap;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + ips::Config + ipo::Config {
        /// Overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The DEV ID type
        type DevId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The DEV properties type
        type DevData: Parameter + Member + MaybeSerializeDeserialize;
        /// The maximum size of an DEV's metadata
        type MaxDevMetadata: Get<u32>;
        /// Currency
        type Currency: Currency<Self::AccountId>;
        /// The allocations of IPO tokens for the users
        type Allocation: Default + Copy + AtLeast32BitUnsigned + Parameter + Member + Sum;
        /// The interactions recorded in the DEV
        type Interaction: Parameter + Member;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type DevIndexOf<T> = <T as Config>::DevId;

    pub type DevMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxDevMetadata>;

    pub type DevAllocations<T> =
        BTreeMap<<T as frame_system::Config>::AccountId, <T as Config>::Allocation>;

    pub type DevInteractions<T> = Vec<<T as Config>::Interaction>;

    pub type DevInfoOf<T> = DevInfo<
        <T as frame_system::Config>::AccountId,
        DevMetadataOf<T>,
        <T as ips::Config>::IpsId,
        <T as Config>::DevData,
        DevAllocations<T>,
        <T as Config>::Allocation,
        DevInteractions<T>,
    >;

    pub type GenesisDev<T> = (
        <T as frame_system::Config>::AccountId, // DEV owner
        Vec<u8>,                                // DEV metadata
        <T as Config>::DevData,                 // DEV data
        Vec<ips::GenesisIps<T>>,                // Vector of IPSs belong to this DEV
        Vec<ipo::GenesisIpo<T>>,                // Vector of IPOs belong to this DEV
    );

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_dev_id)]
    pub type NextDevId<T: Config> = StorageValue<_, T::DevId, ValueQuery>;

    /// Store DEV info
    ///
    /// Return `None` if DEV info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn dev_storage)]
    pub type DevStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::DevId, DevInfoOf<T>>;

    /// DEV existence check by owner and DEV ID
    #[pallet::storage]
    #[pallet::getter(fn dev_by_owner)]
    pub type DevByOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // owner
        Blake2_128Concat,
        T::DevId,
        (),
    >;

    /// DEV existence check by IPS ID
    #[pallet::storage]
    #[pallet::getter(fn dev_by_ips_id)]
    pub type DevByIpsId<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::DevId, Blake2_128Concat, T::IpsId, ()>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::DevId = "IpsId")]
    pub enum Event<T: Config> {
        /// Some DEV were issued.
        Created(T::AccountId, T::DevId),
        /// Dev is posted as joinable \[dev_id\]
        DevPosted(T::DevId),
        /// User is added to DEV \[owner, user\]
        UserAdded(T::DevId, T::AccountId, T::Allocation),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No available DEV ID
        NoAvailableDevId,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// The given DEV ID is unknown
        Unknown,
        /// The operator is not the owner of the DEV and has no permission
        NoPermission,
        /// The operator is not the owner of the IPS and has no permission
        NoPermissionForIps,
        /// IPS already has a registered DEV
        IpsAlreadyHasDev,
        /// The allocations sum to more than the total issuance of IPO for the DEV
        AllocationOverflow,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create Decentalized Entrepreneurial Venture (DEV)
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn create_dev(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            ips_id: T::IpsId,
            data: T::DevData,
            ipo_allocations: Vec<(T::AccountId, T::Allocation)>,
            total_issuance: T::Allocation,
            interactions: DevInteractions<T>,
        ) -> DispatchResultWithPostInfo {
            NextDevId::<T>::try_mutate(|dev_id| -> DispatchResultWithPostInfo {
                let creator = ensure_signed(owner)?;

                // Ensuring the signer owns the IPS he's trying to make a DEV for.
                ensure!(
                    IpsByOwner::<T>::get(creator.clone(), ips_id).is_some(),
                    Error::<T>::NoPermissionForIps
                );

                // Ensuring the IPS doesn't already have a DEV.
                ensure!(
                    DevByIpsId::<T>::get(dev_id.clone(), ips_id).is_none(),
                    Error::<T>::IpsAlreadyHasDev
                );

                let ipo_allocations: BTreeMap<
                    <T as frame_system::Config>::AccountId,
                    <T as Config>::Allocation,
                > = BTreeMap::from_iter(ipo_allocations);

                // Ensuring the total allocation isn't above the total issuance.
                ensure!(
                    ipo_allocations
                        .clone()
                        .into_values()
                        .sum::<<T as Config>::Allocation>()
                        <= total_issuance,
                    Error::<T>::AllocationOverflow
                );

                let bounded_metadata: BoundedVec<u8, T::MaxDevMetadata> = metadata
                    .try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

                let current_id = *dev_id;
                *dev_id = dev_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableDevId)?;

                let info = DevInfo {
                    owner: creator.clone(),
                    metadata: bounded_metadata,
                    ips_id,
                    data: data.clone(),
                    interactions,
                    ipo_allocations: BTreeMap::from_iter(ipo_allocations),
                    total_issuance,
                    is_joinable: false,
                };

                DevStorage::<T>::insert(current_id, info);
                DevByOwner::<T>::insert(creator.clone(), current_id, ());
                DevByIpsId::<T>::insert(dev_id, ips_id, ());

                Self::deposit_event(Event::Created(creator, current_id));

                Ok(().into())
            })
        }

        /// Post a DEV as joinable
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn post_dev(owner: OriginFor<T>, dev_id: T::DevId) -> DispatchResult {
            let creator = ensure_signed(owner)?;

            DevStorage::<T>::try_mutate(dev_id, |maybe_details| {
                let d = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(creator == d.owner, Error::<T>::NoPermission);

                d.is_joinable = true;

                Self::deposit_event(Event::<T>::DevPosted(dev_id));

                Ok(())
            })
        }

        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn add_user(
            owner: OriginFor<T>,
            dev_id: T::DevId,
            user: T::AccountId,
            allocation: T::Allocation,
        ) -> DispatchResult {
            let creator = ensure_signed(owner)?;

            DevStorage::<T>::try_mutate(dev_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(creator == details.owner, Error::<T>::NoPermission);

                // Ensuring the new user's allocation doesn't put the total allocation above the total issuance.
                ensure!(
                    details
                        .ipo_allocations
                        .clone()
                        .into_values()
                        .sum::<<T as Config>::Allocation>()
                        + allocation
                        <= details.total_issuance,
                    Error::<T>::AllocationOverflow
                );

                details.ipo_allocations.insert(user.clone(), allocation);

                Self::deposit_event(Event::<T>::UserAdded(dev_id, user, allocation));

                Ok(())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    // TODO: WIP
}
