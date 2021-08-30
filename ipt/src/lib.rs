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
//! `ammend` - Ammend the data stored inside an IP Token

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
	pub type IptStorage<T> = StorageMap<_, Blake2_128Concat, T::IptId, ipt, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// param. [Ipt, who]
		IptStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// No value error
		NoIptFound,
		/// Storage overflow error
		StorageOverflow,
	}

	// Dispatch functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(100_000 + T::DbWeight::get().writes(1))]
		pub fn mint(origin: OriginFor<T>, ipt: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(100_000 + T::DbWeight::get().writes(1))]
		pub fn burn(origin: OriginFor<T>, ipt: u32) -> DispatchResult {
			// The code goes here
		}

		#[pallet::weight(100_000 + T::DbWeight::get().writes(1))]
		pub fn ammend(origin: OriginFor<T>, ipt: u32) -> DispatchResult {
			// The code goes here
		}
	}
}
