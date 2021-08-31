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
pub mod pallet {
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
	pub type IpsStorage<T> = StorageMap<_, Blake2_128Concat, T::IpsId, ips, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// param. [Ips, who]
		IpsStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// No value error
		NoIpsFound,
		/// Storage overflow error
		StorageOverflow,
	}

	// Dispatch functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(100_000 + T::DbWeight::get().writes(1))]
		pub fn create(origin: OriginFor<T>, ips: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(100_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(origin: OriginFor<T>, ips: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(50_000 + T::DbWeight::get().writes(1))]
		pub fn mint(origin: OriginFor<T>, ips: u32, ipt: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn burn(origin: OriginFor<T>, ips: u32, ipt: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(50_000 + T::DbWeight::get().writes(1))]
		pub fn list(origin: OriginFor<T>, ips: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(50_000 + T::DbWeight::get().writes(1))]
		pub fn buy(origin: OriginFor<T>, ips: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(50_000 + T::DbWeight::get().writes(1))]
		pub fn send(origin: OriginFor<T>, ips: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(50_000 + T::DbWeight::get().writes(1))]
		pub fn destroy(origin: OriginFor<T>, ips: u32) -> DispatchResult {
			// The code goes here
		}
	}
}

