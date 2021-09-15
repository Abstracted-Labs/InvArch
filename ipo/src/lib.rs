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
#![allow(clippy::unused_unit)]

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, Get},
    BoundedVec, Parameter,
};

use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedAdd, MaybeSerializeDeserialize, Member, One},
    DispatchError,
};
use sp_std::{convert::TryInto, vec::Vec};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

/// Import from primitives pallet
use primitives::{IpoInfo, IpsInfo};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The IPO ID type
        type IpoId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy; // TODO: WIP
        /// The IPO properties type
        type IpoData: Parameter + Member + MaybeSerializeDeserialize; // TODO: WIP
        /// The IPS ID type
        type IpsId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// IPT properties type
        type IpsData: Parameter + Member + MaybeSerializeDeserialize; // TODO: WIP
        /// The maximum size of an IPS's metadata
        type MaxIpoMetadata: Get<u32>; // TODO: WIP
        /// The maximum size of an IPT's metadata
        type MaxIpsMetadata: Get<u32>;
        /// Currency
        type Currency: Currency<Self::AccountId>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type IpoIndexOf<T> = <T as Config>::IpoId;
    pub type IpoMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpoMetadata>;
    pub type IpsMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpsMetadata>;
    pub type IpoInfoOf<T> = IpoInfo<
        <T as Config>::IpsId,
        <T as frame_system::Config>::AccountId,
        <T as Config>::IpoData,
        IpoMetadataOf<T>,
    >;
    pub type IpsInfoOf<T> = IpsInfo<
        <T as frame_system::Config>::AccountId,
        <T as Config>::IpsData,
        IpoMetadataOf<T>,
        IpsMetadataOf<T>,
    >;

    pub type GenesisIpsData<T> = (
        <T as frame_system::Config>::AccountId, // IPS owner
        Vec<u8>,                                // IPS metadata
        <T as Config>::IpsData,                 // IPS data
    );
    pub type GenesisIpo<T> = (
        <T as frame_system::Config>::AccountId, // IPO owner
        Vec<u8>,                                // IPO metadata
        <T as Config>::IpoData,                 // IPO data
        Vec<GenesisIpsData<T>>,                 // Vector of IPSs belong to this IPO
    );

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Next available IPO ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ipo_id)]
    pub type NextIpoId<T: Config> = StorageValue<_, T::IpoId, ValueQuery>;

    /// Next available IPS ID
    #[pallet::storage]
    #[pallet::getter(fn next_ips_id)]
    pub type NextIpsId<T: Config> = StorageMap<_, Blake2_128Concat, T::IpoId, T::IpsId, ValueQuery>;

    /// Store IPO info
    ///
    /// Return `None` if IPO info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ipo_storage)]
    pub type IpoStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpoId, IpoInfoOf<T>>;

    /// Store IPS info
    ///
    /// Returns `None` if IPS info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ips_storage)]
    pub type IpsStorage<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::IpoId, Blake2_128Concat, T::IpsId, IpsInfoOf<T>>;

    /// IPS existence check by owner and IPO ID
    #[pallet::storage]
    #[pallet::getter(fn ips_by_owner)]
    pub type IpsByOwner<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // owner
            NMapKey<Blake2_128Concat, T::IpoId>,
            NMapKey<Blake2_128Concat, T::IpsId>,
        ),
        (),
        ValueQuery,
    >;

    /// Get IPO price. None means not for sale.
    #[pallet::storage]
    #[pallet::getter(fn ipo_prices)]
    pub type IpoPrices<T: Config> =
        StorageMap<_, Blake2_128Concat, IpoInfoOf<T>, BalanceOf<T>, OptionQuery>;

    /// Errors for IPO pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPO ID
        NoAvailableIpoId,
        /// No available IPS ID
        NoAvailableIpsId,
        /// IPS (IpoId, IpsId) not found
        IpsNotFound,
        /// IPO not found
        IpONotFound,
        /// The operator is not the owner of the IPS and has no permission
        NoPermission,
        /// The IPO is already owned
        AlreadyOwned,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Buy IPO from their self
        BuyFromSelf,
        /// IPO is not for sale
        NotForSale,
        /// Buy price is too low
        PriceTooLow,
        /// Can not destroy IPO
        CannotDestroyIpo,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    /// Create IP (Intellectual Property) Ownership (IPO)
    pub fn issue_ipo(
        // TODO: WIP
        owner: &T::AccountId,
        metadata: Vec<u8>,
        data: T::IpoData,
    ) -> Result<T::IpoId, DispatchError> {
        let bounded_metadata: BoundedVec<u8, T::MaxIpoMetadata> = metadata
            .try_into()
            .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

        let ipo_id = NextIpoId::<T>::try_mutate(|id| -> Result<T::IpoId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableIpoId)?;
            Ok(current_id)
        })?;

        let info = IpoInfo {
            metadata: bounded_metadata,
            total_issuance: Default::default(),
            owner: owner.clone(),
            data,
        };
        IpoStorage::<T>::insert(ipo_id, info);

        Ok(ipo_id)
    }

    // TODO: WIP
    // - Add transfer function
    // - Add set_balance function
    // - Add get_balance function
    // - Add total_supply function
    // - Add bind function
    // - Add unbind function
}
