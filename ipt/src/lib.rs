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

use frame_support::{
	codec::{Decode, Encode},
	decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
};
use frame_system::ensure_signed;
use sp_runtime::RuntimeDebug;

#[cfg(test)]
mod tests;

decl_storage! {
	
}

decl_event! (
	
);

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
	}
}
