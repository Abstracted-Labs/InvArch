//! Core's XCM location utilities.
//!
//! ## Overview
//!
//! This module implements the [`StaticLookup`] trait allowing for convenient conversion between a
//! Core's id and it's derived AccountId.
//! This implementation abstracs on top of two lower level functions:
//! - `lookup_core`: Used for accessing the storage and retrieving a core's AccountId.
//! - `lookup_address`: Used for converting from a `MultiAddress::Index` that contains a CoreId to this core's AccountId.

use crate::{Config, CoreByAccount, CoreStorage, Pallet};
use core::marker::PhantomData;
use frame_support::error::LookupError;
use sp_runtime::{traits::StaticLookup, MultiAddress};

impl<T: Config> Pallet<T> {
    /// Queries `CoreStorage` to retrieve the AccountId of a core.
    pub fn lookup_core(core_id: T::CoreId) -> Option<T::AccountId> {
        CoreStorage::<T>::get(core_id).map(|core| core.account)
    }

    /// Matches `MultiAddress` to allow for a `MultiAddress::Index` containing a CoreId to be converted
    /// to it's derived AccountId.
    pub fn lookup_address(a: MultiAddress<T::AccountId, T::CoreId>) -> Option<T::AccountId> {
        match a {
            MultiAddress::Id(i) => Some(i),
            MultiAddress::Index(i) => Self::lookup_core(i),
            _ => None,
        }
    }
}

/// StaticLookup implementor using MultiAddress::Index for looking up cores by id.
pub struct INV4Lookup<T: Config>(PhantomData<T>);

impl<T: Config> StaticLookup for INV4Lookup<T> {
    type Source = MultiAddress<T::AccountId, T::CoreId>;
    type Target = T::AccountId;

    fn lookup(a: Self::Source) -> Result<Self::Target, LookupError> {
        Pallet::<T>::lookup_address(a).ok_or(LookupError)
    }

    fn unlookup(a: Self::Target) -> Self::Source {
        match CoreByAccount::<T>::get(&a) {
            Some(core_id) => MultiAddress::Index(core_id),
            None => MultiAddress::Id(a),
        }
    }
}
