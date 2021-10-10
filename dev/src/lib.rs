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

pub use pallet::*;

// TODO: WIP
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + ips::Config {}

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {}

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

}