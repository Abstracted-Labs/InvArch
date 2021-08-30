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
