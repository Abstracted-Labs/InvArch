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

#[cfg(test)]
mod tests;

use frame_support::{
	codec::{Decode, Encode},
	decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
};
use frame_system::ensure_signed;
use sp_runtime::RuntimeDebug;

#[frame_support::pallet]
pub mode pallet {
	use frame_support::{dispatch::DispatchResult, prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type IpoStorage<T> = StorageMap<_, Blake2_128Concat, T::IpoId, ipo, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
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

		#[pallet::weight(100_000 + T::DbWeight::get().writes(1))]
		pub fn issue(
			origin: OriginFor<T>, 
			ipo: u32
			#[pallet::compact] id: T::AssetId,
			admin: <T::Lookup as StaticLookup>::Source,
			min_balance: T::Balance,
		) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(50_000 + T::DbWeight::get().writes(1))]
		pub fn transfer(
			origin: OriginFor<T>,
			ipo: u32,
			#[pallet::compact] id: T::AssetId,
			target: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(50_000 + T::DbWeight::get().writes(1))]
		pub fn bind(
			origin: OriginFor<T>,
			ipo: u32,
			#[pallet::compact] id: T::AssetId,
			target: <T::Lookup as StaticLookup>::Source<IPSet>,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(50_000 + T::DbWeight::get().writes(1))]
		pub fn unbind(
			origin: OriginFor<T>,
			ipo: u32,
			#[pallet::compact] id: <T::Lookup as StaticLookup>::Source<IPSet>,
			target: T::AssetId,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			// The code goes here
		}

	}
}
