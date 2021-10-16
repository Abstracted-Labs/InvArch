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
        
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type DevIndexOf<T> = <T as Config>::DevId;

    pub type DevMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxDevMetadata>;

    pub type DevInfoOf<T> =
        DevInfo<<T as frame_system::Config>::AccountId, <T as Config>::DevData, DevMetadataOf<T>>;

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
    /// Return `None` if IPS info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn dev_storage)]
    pub type DevStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::DevId, DevInfoOf<T>>;

    /// IPS existence check by owner and IPS ID
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

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::DevId = "IpsId")]
    pub enum Event<T: Config> {
        /// Some DEV were issued.
        Created(T::AccountId, T::DevId),
        /// Dev is posted as joinable \[dev_id\]
        DevPosted(T::DevId),
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
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create Decentalized Entrepreneurial Venture (DEV)
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn create_dev(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            data: T::DevData,
            ipo_allocations: u8,
            interactions: u8,
        ) -> DispatchResultWithPostInfo {
            NextDevId::<T>::try_mutate(|dev_id| -> DispatchResultWithPostInfo {
                let creator = ensure_signed(owner)?;

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
                    data: data.clone(),
                    interactions,
                    ipo_allocations: ipo_allocations.clone(),
                    is_joinable: false
                };

                DevStorage::<T>::insert(current_id, info);
                DevByOwner::<T>::insert(creator.clone(), current_id, ());

                Self::deposit_event(Event::Created(creator, current_id));

                Ok(().into())
            })
        }

        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn post_dev(
            owner: OriginFor<T>,
            dev_id: T::DevId,
        ) -> DispatchResult {
            let origin = ensure_signed(owner)?;

            DevStorage::<T>::try_mutate(dev_id, |maybe_details| {
                let d = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(origin == d.owner, Error::<T>::NoPermission);
                
                d.is_joinable = true;

                Self::deposit_event(Event::<T>::DevPosted(dev_id));
                
                Ok(())
            })     
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    // TODO: WIP
}