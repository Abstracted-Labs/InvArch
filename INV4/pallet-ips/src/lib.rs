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

//#[cfg(test)]
//mod mock;
//#[cfg(test)]
//mod tests;

/// Import the primitives crate
use primitives::IpsInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use primitives::utils::multi_account_id;
    use primitives::{AnyId, IpsType, Parentage};
    use scale_info::prelude::fmt::Display;
    use sp_runtime::traits::StaticLookup;
    use sp_std::iter::Sum;
    use sp_std::vec;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + ipf::Config + ipt::Config + pallet_balances::Config
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

        type Balance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo
            + Sum<<Self as pallet::Config>::Balance>;

        #[pallet::constant]
        type ExistentialDeposit: Get<<Self as pallet::Config>::Balance>;
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

    pub type AnyIdWithNewOwner<T> = (
        AnyId<<T as pallet::Config>::IpsId, <T as ipf::Config>::IpfId>,
        <T as frame_system::Config>::AccountId,
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
        Removed(T::AccountId, T::IpsId, Vec<u8>, Vec<AnyIdWithNewOwner<T>>),
        AllowedReplica(T::IpsId),
        DisallowedReplica(T::IpsId),
        ReplicaCreated(T::AccountId, T::IpsId, T::IpsId),
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
        /// Replicas cannot allow themselves to be replicable
        ReplicaCannotAllowReplicas,
        /// Value Not Changed
        ValueNotChanged,
        /// Replicas of this IPS are not allowed
        ReplicaNotAllowed,
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
            allow_replica: bool,
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

                let ips_account = primitives::utils::multi_account_id::<T, <T as Config>::IpsId>(
                    current_id, None,
                );

                for ipf in data.clone() {
                    ipf::Pallet::<T>::send(creator.clone(), ipf, ips_account.clone())?
                }

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
                        .unwrap(),
                    ips_type: IpsType::Normal,
                    allow_replica, // TODO: Remove unwrap.
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
                    Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
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
                    Parentage::Child(_, absolute_parent_account) => absolute_parent_account,
                };

                for asset in assets.clone() {
                    match asset {
                        AnyId::IpsId(ips_id) => {
                            if let Parentage::Parent(acc) = IpsStorage::<T>::get(ips_id)
                                .ok_or(Error::<T>::IpsNotFound)?
                                .parentage
                            {
                                ensure!(
                                    caller_account
                                        == multi_account_id::<T, T::IpsId>(parent_id, Some(acc)),
                                    Error::<T>::NoPermission
                                );
                            } else {
                                return Err(Error::<T>::NotParent.into());
                            }
                        }
                        AnyId::IpfId(ipf_id) => {
                            let this_ipf_owner = ipf::IpfStorage::<T>::get(ipf_id)
                                .ok_or(Error::<T>::IpfNotFound)?
                                .owner;
                            ensure!(
                                this_ipf_owner.clone() == ips_account
                                    || caller_account
                                        == multi_account_id::<T, T::IpsId>(
                                            parent_id,
                                            Some(
                                                ipf::IpfStorage::<T>::get(ipf_id)
                                                    .ok_or(Error::<T>::IpfNotFound)?
                                                    .owner
                                            )
                                        ),
                                Error::<T>::NoPermission
                            );

                            ipf::Pallet::<T>::send(this_ipf_owner, ipf_id, ips_account.clone())?
                        }
                    }
                }

                for any_id in assets.clone().into_iter() {
                    if let AnyId::IpsId(ips_id) = any_id {
                        IpsStorage::<T>::try_mutate_exists(ips_id, |ips| -> DispatchResult {
                            for (account, amount) in ipt::Balance::<T>::iter_prefix(ips_id.into()) {
                                ipt::Pallet::<T>::internal_mint(
                                    account.clone(),
                                    parent_id.into(),
                                    amount,
                                )?;
                                ipt::Pallet::<T>::internal_burn(account, ips_id.into(), amount)?;
                            }

                            ips.clone().unwrap().parentage =
                                Parentage::Child(parent_id, ips_account.clone());

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
                    ips_type: info.ips_type,
                    allow_replica: info.allow_replica,
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
            assets: Vec<AnyIdWithNewOwner<T>>,
            new_metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            IpsStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let caller_account = ensure_signed(owner.clone())?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                let ips_account = match info.parentage.clone() {
                    Parentage::Parent(ips_account) => ips_account,
                    Parentage::Child(_, absolute_parent_account) => absolute_parent_account,
                };

                ensure!(ips_account == caller_account, Error::<T>::NoPermission);

                ensure!(
                    !assets.clone().into_iter().any(|id| {
                        match id.0 {
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
                    match any_id {
                        (AnyId::IpsId(this_ips_id), new_owner) => {
                            IpsStorage::<T>::try_mutate_exists(
                                this_ips_id,
                                |ips| -> DispatchResult {
                                    ipt::Pallet::<T>::internal_mint(
                                        new_owner,
                                        this_ips_id.into(),
                                        <T as ipt::Config>::ExistentialDeposit::get(),
                                    )?;

                                    ips.clone().unwrap().parentage = Parentage::Parent(
                                        multi_account_id::<T, T::IpsId>(this_ips_id, None),
                                    );

                                    Ok(())
                                },
                            )?;
                        }

                        (AnyId::IpfId(this_ipf_id), new_owner) => {
                            ipf::Pallet::<T>::send(ips_account.clone(), this_ipf_id, new_owner)?
                        }
                    }
                }

                let just_ids = assets
                    .clone()
                    .into_iter()
                    .map(|(x, _)| x)
                    .collect::<Vec<AnyId<T::IpsId, T::IpfId>>>();
                old_assets.retain(|x| !just_ids.clone().contains(x));

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
                    ips_type: info.ips_type,
                    allow_replica: info.allow_replica,
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

        /// Allows replicas of this IPS to be made.
        #[pallet::weight(100_000)]
        pub fn allow_replica(owner: OriginFor<T>, ips_id: T::IpsId) -> DispatchResult {
            IpsStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                match info.parentage.clone() {
                    Parentage::Parent(ips_account) => {
                        ensure!(ips_account == owner, Error::<T>::NoPermission)
                    }
                    Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
                }

                ensure!(!info.allow_replica, Error::<T>::ValueNotChanged);

                ensure!(
                    !matches!(info.ips_type, IpsType::Replica(_)),
                    Error::<T>::ReplicaCannotAllowReplicas
                );

                *ips_info = Some(IpsInfo {
                    parentage: info.parentage,
                    metadata: info.metadata,
                    data: info.data,
                    ips_type: info.ips_type,
                    allow_replica: true,
                });

                Self::deposit_event(Event::AllowedReplica(ips_id));

                Ok(())
            })
        }

        /// Disallows replicas of this IPS to be made.
        #[pallet::weight(100_000)]
        pub fn disallow_replica(owner: OriginFor<T>, ips_id: T::IpsId) -> DispatchResult {
            IpsStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;
                let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

                match info.parentage.clone() {
                    Parentage::Parent(ips_account) => {
                        ensure!(ips_account == owner, Error::<T>::NoPermission)
                    }
                    Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
                }

                ensure!(info.allow_replica, Error::<T>::ValueNotChanged);

                ensure!(
                    !matches!(info.ips_type, IpsType::Replica(_)),
                    Error::<T>::ReplicaCannotAllowReplicas
                );

                *ips_info = Some(IpsInfo {
                    parentage: info.parentage,
                    metadata: info.metadata,
                    data: info.data,
                    ips_type: info.ips_type,
                    allow_replica: false,
                });

                Self::deposit_event(Event::DisallowedReplica(ips_id));

                Ok(())
            })
        }

        #[pallet::weight(100_000)]
        pub fn create_replica(
            owner: OriginFor<T>,
            original_ips_id: T::IpsId,
        ) -> DispatchResultWithPostInfo {
            NextIpsId::<T>::try_mutate(|ips_id| -> DispatchResultWithPostInfo {
                let creator = ensure_signed(owner.clone())?;

                let original_ips =
                    IpsStorage::<T>::get(original_ips_id).ok_or(Error::<T>::IpsNotFound)?;

                ensure!(original_ips.allow_replica, Error::<T>::ReplicaNotAllowed);

                let current_id = *ips_id;
                *ips_id = ips_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableIpsId)?;

                let ips_account = primitives::utils::multi_account_id::<T, <T as Config>::IpsId>(
                    current_id, None,
                );

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
                    metadata: original_ips.metadata,
                    data: original_ips.data,
                    ips_type: IpsType::Replica(original_ips_id),
                    allow_replica: false,
                };

                IpsStorage::<T>::insert(current_id, info);
                IpsByOwner::<T>::insert(ips_account.clone(), current_id, ());

                Self::deposit_event(Event::ReplicaCreated(
                    ips_account,
                    current_id,
                    original_ips_id,
                ));

                Ok(().into())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
