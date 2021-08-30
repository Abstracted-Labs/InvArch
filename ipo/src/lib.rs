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
