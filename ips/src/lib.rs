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
//! - `send` - Transfer IP Set owner account address
//! - `list` - List an IP Set for sale
//! - `buy` - Buy an IP Set
//! - `destroy` - Delete an IP Set and all of its contents

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, Get},
    BoundedVec, Parameter,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedAdd, MaybeSerializeDeserialize, Member, One, Zero},
    DispatchError,
};
use sp_std::{convert::TryInto, vec::Vec};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

/// Import from IPT pallet
use primitives::{IpsInfo, IptInfo};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The IPS ID type
        type IpsId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy; // TODO: WIP
        /// The IPS properties type
        type IpsData: Parameter + Member + MaybeSerializeDeserialize; // TODO: WIP
        /// The IPT ID type
        type IptId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// IPT properties type
        type IptData: Parameter + Member + MaybeSerializeDeserialize; // TODO: WIP
        /// The maximum size of an IPS's metadata
        type MaxIpsMetadata: Get<u32>; // TODO: WIP
        /// The maximum size of an IPT's metadata
        type MaxIptMetadata: Get<u32>;
        /// Currency
        type Currency: Currency<Self::AccountId>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type IpsIndexOf<T> = <T as Config>::IpsId;
    pub type IpsMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpsMetadata>;
    pub type IptMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIptMetadata>;
    pub type IpsInfoOf<T> = IpsInfo<
        <T as Config>::IptId,
        <T as frame_system::Config>::AccountId,
        <T as Config>::IpsData,
        IpsMetadataOf<T>,
    >;
    pub type IptInfoOf<T> =
        IptInfo<<T as frame_system::Config>::AccountId, <T as Config>::IptData, IptMetadataOf<T>>;

    pub type GenesisIptData<T> = (
        <T as frame_system::Config>::AccountId, // IPT owner
        Vec<u8>,                                // IPT metadata
        <T as Config>::IptData,                 // IPT data
    );
    pub type GenesisIps<T> = (
        <T as frame_system::Config>::AccountId, // IPS owner
        Vec<u8>,                                // IPS metadata
        <T as Config>::IpsData,                 // IPS data
        Vec<GenesisIptData<T>>,                 // Vector of IPTs belong to this IPS
    );

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Next available IPS ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ips_id)]
    pub type NextIpsId<T: Config> = StorageValue<_, T::IpsId, ValueQuery>;

    /// Next available IPT ID
    #[pallet::storage]
    #[pallet::getter(fn next_ipt_id)]
    pub type NextIptId<T: Config> = StorageMap<_, Blake2_128Concat, T::IpsId, T::IptId, ValueQuery>;

    /// Store IPS info
    ///
    /// Return `None` if IPS info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ips_storage)]
    pub type IpsStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpsId, IpsInfoOf<T>>;

    /// Store IPT info
    ///
    /// Returns `None` if IPT info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ipt_storage)]
    pub type IptStorage<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::IpsId, Blake2_128Concat, T::IptId, IptInfoOf<T>>;

    /// IPT existence check by owner and IPS ID
    #[pallet::storage]
    #[pallet::getter(fn ipt_by_owner)]
    pub type IptByOwner<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // owner
            NMapKey<Blake2_128Concat, T::IpsId>,
            NMapKey<Blake2_128Concat, T::IptId>,
        ),
        (),
        ValueQuery,
    >;

    /// Get IPS price. None means not for sale.
    #[pallet::storage]
    #[pallet::getter(fn ips_prices)]
    pub type IpsPrices<T: Config> =
        StorageMap<_, Blake2_128Concat, IpsInfoOf<T>, BalanceOf<T>, OptionQuery>;

    /// Errors for IPT pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPS ID
        NoAvailableIpsId,
        /// No available IPT ID
        NoAvailableIptId,
        /// IPT (IpsId, IptId) not found
        IptNotFound,
        /// IPS not found
        IpsNotFound,
        /// The operator is not the owner of the IPT and has no permission
        NoPermission,
        /// The IPS is already owned
        AlreadyOwned,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Buy IPS from their self
        BuyFromSelf,
        /// IPS is not for sale
        NotForSale,
        /// Buy price is too low
        PriceTooLow,
        /// Can not destroy IPS
        CannotDestroyIps,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    /// Create IP (Intellectual Property) Set (IPS)
    pub fn create_ips(
        // TODO: WIP
        owner: &T::AccountId,
        metadata: Vec<u8>,
        data: T::IpsData,
    ) -> Result<T::IpsId, DispatchError> {
        let bounded_metadata: BoundedVec<u8, T::MaxIpsMetadata> = metadata
            .try_into()
            .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

        let ips_id = NextIpsId::<T>::try_mutate(|id| -> Result<T::IpsId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableIpsId)?;
            Ok(current_id)
        })?;

        let info = IpsInfo {
            metadata: bounded_metadata,
            total_issuance: Default::default(),
            owner: owner.clone(),
            data,
        };
        IpsStorage::<T>::insert(ips_id, info);

        Ok(ips_id)
    }

    /// Transfer IP Set owner account address
    pub fn send(
        from: &T::AccountId,
        to: &T::AccountId,
        ipt: (T::IpsId, T::IptId),
    ) -> DispatchResult {
        IptStorage::<T>::try_mutate(ipt.0, ipt.1, |ipt_info| -> DispatchResult {
            let mut info = ipt_info.as_mut().ok_or(Error::<T>::IptNotFound)?;
            ensure!(info.owner == *from, Error::<T>::NoPermission);
            ensure!(*from != *to, Error::<T>::AlreadyOwned);

            info.owner = to.clone();

            IptByOwner::<T>::remove((from, ipt.0, ipt.1));
            IptByOwner::<T>::insert((to, ipt.0, ipt.1), ());

            Ok(())
        })
    }

    /// List a IPS for sale
    /// None to delist the IPS
    pub fn list(
        owner: T::AccountId,
        ips_id: T::IpsId,
        ips_index: IpsInfoOf<T>,
        new_price: Option<BalanceOf<T>>,
    ) -> DispatchResult {
        IpsStorage::<T>::try_mutate(ips_id, |ips_info| -> DispatchResult {
            let info = ips_info.as_mut().ok_or(Error::<T>::IpsNotFound)?;
            ensure!(info.owner == owner, Error::<T>::NoPermission);

            IpsPrices::<T>::mutate_exists(ips_index, |price| *price = new_price);

            Ok(())
        })
    }

    /// Allow a user to buy an IPS
    pub fn buy(
        origin: OriginFor<T>,
        owner: T::AccountId,
        ips_id: T::IpsId,
        ipt_id: T::IptId,
        ips_index: IpsInfoOf<T>,
        max_price: BalanceOf<T>,
    ) -> DispatchResult {
        let sender = ensure_signed(origin)?;

        ensure!(sender != owner, Error::<T>::BuyFromSelf);

        IpsPrices::<T>::try_mutate_exists(ips_index, |price| -> DispatchResult {
            let price = price.take().ok_or(Error::<T>::NotForSale)?;

            ensure!(max_price >= price, Error::<T>::PriceTooLow);

            Pallet::<T>::send(&owner, &sender, (ips_id, ipt_id))?;
            T::Currency::transfer(&sender, &owner, price, ExistenceRequirement::KeepAlive)?;

            Ok(())
        })
    }

    // Delete an IP Set and all of its contents
    pub fn destroy(owner: &T::AccountId, ips_id: T::IpsId) -> DispatchResult {
        IpsStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
            let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;
            ensure!(info.owner == *owner, Error::<T>::NoPermission);
            ensure!(
                info.total_issuance == Zero::zero(),
                Error::<T>::CannotDestroyIps
            );

            NextIptId::<T>::remove(ips_id);

            Ok(())
        })
    }

    pub fn is_owner(account: &T::AccountId, ipt: (T::IpsId, T::IptId)) -> bool {
        IptByOwner::<T>::contains_key((account, ipt.0, ipt.1))
    }
}
