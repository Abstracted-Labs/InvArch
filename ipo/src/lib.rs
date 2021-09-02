//! # IPO
//! IP Ownership
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to create and manage IP Ownership tokens, which reflect ownership over an IP Set and governing weight in a DEV's governance.
//!
//! ### Pallet Functions
//!
//! `issue` - Issues the total supply of a new fungible asset to the account of the caller of the function
//! `transfer` - Transfer some liquid free balance to another account
//! `set_balance` - Set the balances to a given account. The origin of this call mus be root
//! `get_balance` - Get the asset `id` balance of `who`
//! `total_supply` - Get the total supply of an asset `id`
//! `bind` - Bind some `amount` of unit of fungible asset `id` from the ballance of the function caller's account (`origin`) to a specific `IPSet`
//! account to claim some portion of fractionalized ownership of that particular `IPset`
//! `unbind` - Unbind some `amount` of unit of fungible asset `id` from a specific `IPSet` account to unclaim some portion of fractionalized ownership
//! to the ballance of the function caller's account.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
    pub type IpoId = u32;
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type IpoStorage<T> = StorageMap<_, Blake2_128Concat, IpoId, u32>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// param. [Ipo, who]
        IpoStored(u32, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// No value error
        NoIpoFound,
        /// Storage overflow error
        StorageOverflow,
    }

    // Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO: WIP
    }
}
