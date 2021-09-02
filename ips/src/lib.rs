//! # IPS
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
//! - `change_owner` - Change the owner of an IP Set
//! - `mint` - Mint a new IPT inside ab IP Set
//! - `burn` - Burn IPT(intellectual property token)
//! - `list` - List an IP Set for sale
//! - `buy` - Buy an IP Set
//! - `send` - Transfer IP Set owner account address
//! - `destroy` - Delete an IP Set and all of its contents

#![cfg_attr(not(feature = "std"), no_std)]

/// IpsId
pub type IpsId = u32;

/// IP Set struct

pub struct Ips<Balance> {
    pub name: Vec<u8>,
    pub data: IpsData<Balance>,
    pub description: Vec<u8>,
    /// Metadata from ipfs
    pub metadata: Vec<u8>,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct IpsData<Balance> {
    /// Deposit balance to create each token
    pub deposit: Balance,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub metadata: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
    use crate::IpsId;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[pallet::storage]
    #[pallet::getter(fn get_ips)]
    pub(super) type Ips<T: Config> = StorageMap<_, Blake2_128Concat, IpsId, u64>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// param. [Ips, who]
        IpsCreated(AccountIdOf<T>, u32, u32, Vec<u8>, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No value error
        NoIpsFound,
        /// InvalidQuantity
        InvalidQuantity,
        /// No permission
        NoPermission,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000 + T::DbWeight::get().writes(1))]
        pub fn create(
            owner_id: OriginFor<T>,
            name: u32,
            description: u32,
            metadata: Vec<u8>,
            quantity: u32,
        ) -> DispatchResult {
            let sender = ensure_signed(owner_id)?;

            // TODO : WIP

            ensure!(quantity >= 1, Error::<T>::InvalidQuantity);

            Self::deposit_event(Event::<T>::IpsCreated(
                sender,
                name,
                description,
                metadata,
                quantity,
            ));

            Ok(())
        }
    }
}
