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
    traits::{Currency, Get, WithdrawReasons},
    BoundedVec, Parameter,
};

use frame_system::{ensure_signed, pallet_prelude::*};

use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, CheckedAdd, MaybeSerializeDeserialize, Member, One, Saturating, Zero,
    },
    DispatchError,
};

use scale_info::TypeInfo;
use sp_std::{convert::TryInto, ops::BitOr, vec::Vec};

use codec::{Codec, MaxEncodedLen};
use sp_std::{fmt::Debug, prelude::*};

/// Import from primitives pallet
use primitives::DevInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + ips::Config {
        /// Overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The DEV ID type
        type DevId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The maximum size of an DEV's metadata
        type MaxDevMetadata: Get<u32>;
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
    /// Return `None` if DEV info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn dev_storage)]
    pub type DevStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::DevId, DevInfoOf<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Some DEV were issued. \[dev_id, owner, ipo_allocation, interaction\]
        Created(T::DevId, T::AccountId, T::Balance, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No available DEV ID
        NoAvailableDevId,
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
            ipo_allocations: T::Balance,
            interactions: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(owner)?;

            let bounded_metadata: BoundedVec<u8, T::MaxDevMetadata> = metadata
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;
            
            let dev_id = NextDevId::<T>::try_mutate(|id| -> Result<T::DevId, DispatchError> {
                let current_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableDevId)?;
                Ok(current_id)
            })?;

            let info = DevInfo {
                owner: signer.clone(),
                metadata: bounded_metadata,
                data,
            };
            
            let ipo_allocations = ipo_allocations.clone();
            let interactions = ipo_allocations.clone();

            DevStorage::<T>::insert(dev_id, info, ipo_allocations, interactions);
            Self::deposit_event(Event::Created(dev_id, signer, ipo_allocations, interactions));

            Ok(().into())
        }

        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn post_dev(
            owner: OriginFor<T>,
        ) -> {
            
        }

    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    // TODO: WIP
}