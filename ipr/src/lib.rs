//! # Pallet IPR
//! Intellectual Property Replicas
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to produce a noted, tracked, & authorized copy of a IP File or a NFT featuring a standard that is interoperable & composable with the INV4 Protocol.
//!
//! ### Pallet Functions
//!
//! - `replicate` - Create a new IP Replica
//! 

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		// type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// SomethingStored(u32, T::AccountId),
	}


	#[pallet::error]
	pub enum Error<T> {
		// NoneValue,
		// StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO: create replicate function
	}
}