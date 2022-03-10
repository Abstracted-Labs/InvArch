//! # Pallet IPS
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
    traits::{Currency as FSCurrency, Get},
    BoundedVec, Parameter,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, Member, One};
use sp_std::{convert::TryInto, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Import the primitives crate
use primitives::IpsInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use primitives::utils::multi_account_id;
    use primitives::{AnyId, Parentage};
    use scale_info::prelude::fmt::Display;
    use sp_runtime::traits::StaticLookup;
    use sp_std::vec;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + ipf::Config
        + ipt::Config
        + pallet_assets::Config
        + pallet_balances::Config
    {
        /// The IPS Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The IPS ID type
        type IpsId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen
            + IsType<<Self as ipt::Config>::IptId>;
        /// The maximum size of an IPS's metadata
        type MaxIpsMetadata: Get<u32>;
        /// Currency
        type Currency: FSCurrency<Self::AccountId>;

        type IpsData: IntoIterator + Clone;

        #[pallet::constant]
        type ExistentialDeposit: Get<<Self as pallet_assets::Config>::Balance>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as FSCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type IpsIndexOf<T> = <T as Config>::IpsId;

    pub type IpsMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpsMetadata>;

    pub type IpsInfoOf<T> = IpsInfo<
        <T as frame_system::Config>::AccountId,
        BoundedVec<
            AnyId<<T as Config>::IpsId, <T as ipf::Config>::IpfId>,
            <T as Config>::MaxIpsMetadata,
        >,
        IpsMetadataOf<T>,
        <T as Config>::IpsId,
    >;

    pub type GenesisIps<T> = (
        <T as frame_system::Config>::AccountId, // IPS owner
        Vec<u8>,                                // IPS metadata
        BoundedVec<
            AnyId<<T as Config>::IpsId, <T as ipf::Config>::IpfId>,
            <T as Config>::MaxIpsMetadata,
        >, // IPS data
        Vec<ipf::GenesisIpfData<T>>,            // Vector of IPFs belong to this IPS
    );

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Next available IPS ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ips_id)]
    pub type NextIpsId<T: Config> = StorageValue<_, T::IpsId, ValueQuery>;

    /// Store IPS info
    ///
    /// Return `None` if IPS info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ips_storage)]
    pub type IpsStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpsId, IpsInfoOf<T>>;

    /// IPS existence check by owner and IPS ID
    #[pallet::storage]
    #[pallet::getter(fn ips_by_owner)]
    pub type IpsByOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // owner
        Blake2_128Concat,
        T::IpsId,
        (),
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        Created(T::AccountId, T::IpsId),
        Destroyed(T::AccountId, T::IpsId),
        Appended(
            T::AccountId,
            T::IpsId,
            Vec<u8>,
            Vec<AnyId<T::IpsId, T::IpfId>>,
        ),
        Removed(
            T::AccountId,
            T::IpsId,
            Vec<u8>,
            Vec<AnyId<T::IpsId, T::IpfId>>,
        ),
    }

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPS ID
        NoAvailableIpsId,
        /// No available IPF ID
        NoAvailableIpfId,
        /// IPF (IpsId, IpfId) not found
        IpfNotFound,
        /// IPS not found
        IpsNotFound,
        /// The operator is not the owner of the IPF and has no permission
        NoPermission,
        /// The IPS is already owned
        AlreadyOwned,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Can not destroy IPS
        CannotDestroyIps,
        /// IPS is not a parent IPS
        NotParent,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create IP (Intellectual Property) Set (IPS)
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn create_ips(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            data: Vec<<T as ipf::Config>::IpfId>,
        ) -> DispatchResultWithPostInfo {
            NextIpsId::<T>::try_mutate(|ips_id| -> DispatchResultWithPostInfo {
                let creator = ensure_signed(owner.clone())?;

                let bounded_metadata: BoundedVec<u8, T::MaxIpsMetadata> = metadata
                    .try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

                let current_id = *ips_id;
                *ips_id = ips_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableIpsId)?;

                ensure!(
                    !data.clone().into_iter().any(|ipf_id| {
                        ipf::IpfByOwner::<T>::get(creator.clone(), ipf_id).is_none()
                    }),
                    Error::<T>::NoPermission
                );

                let ips_account =
                    primitives::utils::multi_account_id::<T, <T as Config>::IpsId>(current_id);

                pallet_balances::Pallet::<T>::transfer_keep_alive(
                    owner.clone(),
                    T::Lookup::unlookup(ips_account.clone()),
                    <T as pallet_balances::Config>::ExistentialDeposit::get(),
                )?;

                ipt::Pallet::<T>::create(
                    ips_account.clone(),
                    current_id.into(),
                    vec![(creator, <T as ipt::Config>::ExistentialDeposit::get())],
                );

                let info = IpsInfo {
                    parentage: Parentage::Parent(ips_account.clone()),
                    metadata: bounded_metadata,
                    data: data
                        .into_iter()
                        .map(AnyId::IpfId)
                        .collect::<Vec<AnyId<<T as Config>::IpsId, <T as ipf::Config>::IpfId>>>()
                        .try_into()
                        .unwrap(), // TODO: Remove unwrap.
                };

                IpsStorage::<T>::insert(current_id, info);
                IpsByOwner::<T>::insert(ips_account.clone(), current_id, ());

                Self::deposit_event(Event::Created(ips_account, current_id));

                Ok(().into())
            })
        }

        /// Delete an IP Set and all of its contents
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn destroy(owner: OriginFor<T>, ips_id: T::IpsId) -> DispatchResult {
            IpsStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                match info.parentage {
                    Parentage::Parent(ips_account) => {
                        ensure!(ips_account == owner, Error::<T>::NoPermission)
                    }
                    Parentage::Child(parent_id) => {
                        if let Parentage::Parent(ips_account) = IpsStorage::<T>::get(parent_id)
                            .ok_or(Error::<T>::IpsNotFound)?
                            .parentage
                        {
                            ensure!(ips_account == owner, Error::<T>::NoPermission)
                        } else {
                            return Err(Error::<T>::NotParent.into());
                        }
                    }
                }

                IpsByOwner::<T>::remove(owner.clone(), ips_id);

                // TODO: Destroy IPT.

                Self::deposit_event(Event::Destroyed(owner, ips_id));

                Ok(())
            })
        }

        /// Append new assets to an IP Set
        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn append(
            owner: OriginFor<T>,
            ips_id: T::IpsId,
            assets: Vec<AnyId<T::IpsId, T::IpfId>>,
            new_metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            IpsStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let caller_account = ensure_signed(owner.clone())?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                let parent_id = ips_id;

                let ips_account = match info.parentage.clone() {
                    Parentage::Parent(ips_account) => ips_account,
                    Parentage::Child(parent_id) => {
                        if let Parentage::Parent(ips_account) = IpsStorage::<T>::try_get(parent_id)
                            .map_err(|_| Error::<T>::IpsNotFound)?
                            .parentage
                        {
                            ips_account
                        } else {
                            todo!()
                        }
                    }
                };

                ensure!(ips_account == caller_account, Error::<T>::NoPermission);

                ensure!(
                    !assets.clone().into_iter().any(|id| {
                        match id {
                            AnyId::IpsId(ips_id) => {
                                IpsByOwner::<T>::get(ips_account.clone(), ips_id).is_none()
                            }
                            AnyId::IpfId(ipf_id) => {
                                ipf::IpfByOwner::<T>::get(ips_account.clone(), ipf_id).is_none()
                            }
                        }
                    }),
                    Error::<T>::NoPermission
                );

                for any_id in assets.clone().into_iter() {
                    if let AnyId::IpsId(ips_id) = any_id {
                        IpsStorage::<T>::try_mutate_exists(ips_id, |ips| -> DispatchResult {
                            for (account, amount) in ipt::Balance::<T>::iter_prefix(ips_id.into()) {
                                ipt::Pallet::<T>::internal_mint(account, ips_id.into(), amount)?
                            }

                            ips.clone().unwrap().parentage = Parentage::Child(parent_id);

                            Ok(())
                        })?;
                    }
                }

                *ips_info = Some(IpsInfo {
                    parentage: info.parentage,
                    metadata: if let Some(metadata) = new_metadata.clone() {
                        metadata
                            .try_into()
                            .map_err(|_| Error::<T>::MaxMetadataExceeded)?
                    } else {
                        info.metadata.clone()
                    },
                    data: info
                        .data
                        .into_iter()
                        .chain(assets.clone().into_iter())
                        .collect::<Vec<AnyId<<T as Config>::IpsId, <T as ipf::Config>::IpfId>>>()
                        .try_into()
                        .unwrap(), // TODO: Remove unwrap.
                });

                Self::deposit_event(Event::Appended(
                    caller_account,
                    ips_id,
                    if let Some(metadata) = new_metadata {
                        metadata
                    } else {
                        info.metadata.to_vec()
                    },
                    assets,
                ));

                Ok(())
            })
        }

        /// Remove assets from an IP Set
        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn remove(
            owner: OriginFor<T>,
            ips_id: T::IpsId,
            assets: Vec<AnyId<T::IpsId, T::IpfId>>,
            new_metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            IpsStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let caller_account = ensure_signed(owner.clone())?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                let ips_account = match info.parentage.clone() {
                    Parentage::Parent(ips_account) => ips_account,
                    Parentage::Child(parent_id) => {
                        if let Parentage::Parent(ips_account) = IpsStorage::<T>::try_get(parent_id)
                            .map_err(|_| Error::<T>::IpsNotFound)?
                            .parentage
                        {
                            ips_account
                        } else {
                            todo!()
                        }
                    }
                };

                ensure!(ips_account == caller_account, Error::<T>::NoPermission);

                ensure!(
                    !assets.clone().into_iter().any(|id| {
                        match id {
                            AnyId::IpsId(ips_id) => {
                                IpsByOwner::<T>::get(ips_account.clone(), ips_id).is_none()
                            }
                            AnyId::IpfId(ipf_id) => {
                                ipf::IpfByOwner::<T>::get(ips_account.clone(), ipf_id).is_none()
                            }
                        }
                    }),
                    Error::<T>::NoPermission
                );

                let mut old_assets = info.data.clone();

                for any_id in assets.clone().into_iter() {
                    if let AnyId::IpsId(ips_id) = any_id {
                        IpsStorage::<T>::try_mutate_exists(ips_id, |ips| -> DispatchResult {
                            for (account, amount) in ipt::Balance::<T>::iter_prefix(ips_id.into()) {
                                ipt::Pallet::<T>::internal_burn(account, ips_id.into(), amount)?
                            }

                            ips.clone().unwrap().parentage =
                                Parentage::Parent(multi_account_id::<T, T::IpsId>(ips_id));

                            Ok(().into())
                        })?;
                    }
                }

                old_assets.retain(|x| !assets.clone().contains(x));

                *ips_info = Some(IpsInfo {
                    parentage: info.parentage,
                    metadata: if let Some(metadata) = new_metadata.clone() {
                        metadata
                            .try_into()
                            .map_err(|_| Error::<T>::MaxMetadataExceeded)?
                    } else {
                        info.metadata.clone()
                    },
                    data: old_assets,
                });

                Self::deposit_event(Event::Removed(
                    caller_account,
                    ips_id,
                    if let Some(metadata) = new_metadata {
                        metadata
                    } else {
                        info.metadata.to_vec()
                    },
                    assets,
                ));

                Ok(())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
