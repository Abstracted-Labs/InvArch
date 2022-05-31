use super::pallet::*;
use crate::ipl::LicenseList;
use frame_support::pallet_prelude::*;
use frame_system::ensure_signed;
use frame_system::pallet_prelude::*;
use primitives::utils::multi_account_id;
use primitives::IpInfo;
use primitives::{IpsType, OneOrPercent, Parentage};
use rmrk_traits::Nft;
use sp_arithmetic::traits::{CheckedAdd, One, Zero};
use sp_runtime::traits::StaticLookup;
use sp_std::convert::TryInto;

pub type IpsIndexOf<T> = <T as Config>::IpId;

pub type IpsMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxMetadata>;

impl<T: Config> Pallet<T> {
    pub(crate) fn inner_create_ips(
        owner: OriginFor<T>,
        metadata: Vec<u8>,
        data: Vec<(
            rmrk_traits::primitives::CollectionId,
            rmrk_traits::primitives::NftId,
        )>,
        allow_replica: bool,
        ipl_license: <T as Config>::Licenses,
        ipl_execution_threshold: OneOrPercent,
        ipl_default_asset_weight: OneOrPercent,
        ipl_default_permission: bool,
    ) -> DispatchResultWithPostInfo {
        NextIpId::<T>::try_mutate(|ips_id| -> DispatchResultWithPostInfo {
            let creator = ensure_signed(owner.clone())?;

            let bounded_metadata: BoundedVec<u8, T::MaxMetadata> = metadata
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

            let current_id = *ips_id;
            *ips_id = ips_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableIpId)?;

            for nft_id in &data {
                ensure!(
                    pallet_rmrk_core::Pallet::<T>::lookup_root_owner(nft_id.0, nft_id.1)
                        .map_err(|_| Error::<T>::NoPermission)?
                        == (creator.clone(), *nft_id),
                    Error::<T>::NoPermission
                );
            }

            let ips_account =
                primitives::utils::multi_account_id::<T, <T as Config>::IpId>(current_id, None);

            for ipf in data.clone() {
                pallet_rmrk_core::Pallet::<T>::nft_send(
                    creator.clone(),
                    ipf.0,
                    ipf.1,
                    rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(ips_account.clone()),
                )?;
            }

            pallet_balances::Pallet::<T>::transfer_keep_alive(
                owner.clone(),
                T::Lookup::unlookup(ips_account.clone()),
                <T as pallet_balances::Config>::ExistentialDeposit::get(),
            )?;

            let info = IpInfo {
                parentage: Parentage::Parent(ips_account.clone()),
                metadata: bounded_metadata,
                data: data
                    .into_iter()
                    .map(AnyId::RmrkId)
                    .collect::<Vec<AnyIdOf<T>>>()
                    .try_into()
                    .unwrap(),
                ips_type: IpsType::Normal,
                allow_replica,

                supply: Zero::zero(),

                license: ipl_license.get_hash_and_metadata(),
                execution_threshold: ipl_execution_threshold,
                default_asset_weight: ipl_default_asset_weight,
                default_permission: ipl_default_permission,
            };

            IpStorage::<T>::insert(current_id, info);
            IpsByOwner::<T>::insert(ips_account.clone(), current_id, ());

            Self::deposit_event(Event::Created(ips_account, current_id));

            Ok(().into())
        })
    }

    pub(crate) fn inner_append(
        owner: OriginFor<T>,
        ips_id: T::IpId,
        assets: Vec<AnyIdOf<T>>,
        new_metadata: Option<Vec<u8>>,
    ) -> DispatchResult {
        IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
            let caller_account = ensure_signed(owner.clone())?;
            let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

            let parent_id = ips_id;

            let ips_account = match info.parentage.clone() {
                Parentage::Parent(ips_account) => ips_account,
                Parentage::Child(_, absolute_parent_account) => absolute_parent_account,
            };

            ensure!(
                !assets.is_empty() || new_metadata.is_some(),
                Error::<T>::ValueNotChanged
            );

            for asset in assets.clone() {
                match asset {
                    AnyId::IpsId(ips_id) => {
                        if let Parentage::Parent(acc) = IpStorage::<T>::get(ips_id)
                            .ok_or(Error::<T>::IpsNotFound)?
                            .parentage
                        {
                            ensure!(
                                caller_account
                                    == multi_account_id::<T, T::IpId>(parent_id, Some(acc)),
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
                                    == multi_account_id::<T, T::IpId>(
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
                    AnyId::RmrkId(ipf_id) => {
                        let this_ipf_owner = pallet_rmrk_core::Nfts::<T>::get(ipf_id.0, ipf_id.1)
                            .ok_or(Error::<T>::IpfNotFound)?
                            .owner;
                        ensure!(
                            this_ipf_owner.clone()
                                == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                                    ips_account.clone()
                                )
                                || if let rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(a) =
                                    pallet_rmrk_core::Nfts::<T>::get(ipf_id.0, ipf_id.1)
                                        .ok_or(Error::<T>::IpfNotFound)?
                                        .owner
                                {
                                    caller_account
                                        == multi_account_id::<T, T::IpId>(parent_id, Some(a))
                                } else {
                                    false
                                },
                            Error::<T>::NoPermission
                        );

                        if let rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(acc) =
                            this_ipf_owner
                        {
                            pallet_rmrk_core::Pallet::<T>::nft_send(
                                acc,
                                ipf_id.0,
                                ipf_id.1,
                                rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                                    ips_account.clone(),
                                ),
                            )?;
                        } else {
                            panic!()
                        }
                    }
                }
            }

            for any_id in assets.clone().into_iter() {
                if let AnyId::IpsId(ips_id) = any_id {
                    IpStorage::<T>::try_mutate_exists(ips_id, |ips| -> DispatchResult {
                        let old = ips.take().ok_or(Error::<T>::IpsNotFound)?;

                        let prefix: (<T as Config>::IpId, Option<<T as Config>::IpId>) =
                            (ips_id.into(), None);
                        for (account, amount) in Balance::<T>::iter_prefix(prefix) {
                            let id: (<T as Config>::IpId, Option<<T as Config>::IpId>) =
                                (parent_id.into(), None);
                            Pallet::<T>::internal_mint(id, account.clone(), amount)?;
                            Pallet::<T>::internal_burn(account, prefix, amount)?;
                        }

                        *ips = Some(IpInfo {
                            parentage: Parentage::Child(parent_id, ips_account.clone()),
                            metadata: old.metadata,
                            data: old.data,
                            ips_type: old.ips_type,
                            allow_replica: old.allow_replica,

                            supply: old.supply,

                            license: old.license,
                            execution_threshold: old.execution_threshold,
                            default_asset_weight: old.default_asset_weight,
                            default_permission: old.default_permission,
                        });

                        Ok(())
                    })?;
                }
            }

            *ips_info = Some(IpInfo {
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
                    .collect::<Vec<AnyIdOf<T>>>()
                    .try_into()
                    .unwrap(), // TODO: Remove unwrap.
                ips_type: info.ips_type,
                allow_replica: info.allow_replica,

                supply: info.supply,

                license: info.license,
                execution_threshold: info.execution_threshold,
                default_asset_weight: info.default_asset_weight,
                default_permission: info.default_permission,
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

    pub(crate) fn inner_remove(
        owner: OriginFor<T>,
        ips_id: T::IpId,
        assets: Vec<AnyIdWithNewOwner<T>>,
        new_metadata: Option<Vec<u8>>,
    ) -> DispatchResult {
        IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
            let caller_account = ensure_signed(owner.clone())?;
            let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

            let ips_account = match info.parentage.clone() {
                Parentage::Parent(ips_account) => ips_account,
                Parentage::Child(_, absolute_parent_account) => absolute_parent_account,
            };

            ensure!(ips_account == caller_account, Error::<T>::NoPermission);

            ensure!(
                !assets
                    .clone()
                    .into_iter()
                    .any(|id| { !info.data.contains(&id.0) }),
                Error::<T>::NoPermission
            );

            let mut old_assets = info.data.clone();

            for any_id in assets.clone().into_iter() {
                match any_id {
                    (AnyId::IpsId(this_ips_id), new_owner) => {
                        IpStorage::<T>::try_mutate_exists(this_ips_id, |ips| -> DispatchResult {
                            let id: (<T as Config>::IpId, Option<<T as Config>::IpId>) =
                                (this_ips_id.into(), None);
                            Pallet::<T>::internal_mint(
                                id,
                                new_owner,
                                <T as Config>::ExistentialDeposit::get(),
                            )?;

                            ips.clone().unwrap().parentage =
                                Parentage::Parent(multi_account_id::<T, T::IpId>(
                                    this_ips_id,
                                    None,
                                ));

                            Ok(())
                        })?;
                    }

                    (AnyId::IpfId(this_ipf_id), new_owner) => {
                        ipf::Pallet::<T>::send(ips_account.clone(), this_ipf_id, new_owner)?
                    }

                    (AnyId::RmrkId(this_ipf_id), new_owner) => {
                        pallet_rmrk_core::Pallet::<T>::nft_send(
                            ips_account.clone(),
                            this_ipf_id.0,
                            this_ipf_id.1,
                            rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(new_owner),
                        )?;
                    }
                }
            }

            let just_ids = assets
                .clone()
                .into_iter()
                .map(|(x, _)| x)
                .collect::<Vec<AnyIdOf<T>>>();
            old_assets.retain(|x| !just_ids.clone().contains(x));

            *ips_info = Some(IpInfo {
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

                supply: info.supply,

                license: info.license,
                execution_threshold: info.execution_threshold,
                default_asset_weight: info.default_asset_weight,
                default_permission: info.default_permission,
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

    pub(crate) fn inner_allow_replica(owner: OriginFor<T>, ips_id: T::IpId) -> DispatchResult {
        IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
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

            *ips_info = Some(IpInfo {
                parentage: info.parentage,
                metadata: info.metadata,
                data: info.data,
                ips_type: info.ips_type,
                allow_replica: true,

                supply: info.supply,

                license: info.license,
                execution_threshold: info.execution_threshold,
                default_asset_weight: info.default_asset_weight,
                default_permission: info.default_permission,
            });

            Self::deposit_event(Event::AllowedReplica(ips_id));

            Ok(())
        })
    }

    pub(crate) fn inner_disallow_replica(owner: OriginFor<T>, ips_id: T::IpId) -> DispatchResult {
        IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
            let owner = ensure_signed(owner)?;
            let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

            match info.parentage.clone() {
                Parentage::Parent(ips_account) => {
                    ensure!(ips_account == owner, Error::<T>::NoPermission)
                }
                Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
            }

            ensure!(
                !matches!(info.ips_type, IpsType::Replica(_)),
                Error::<T>::ReplicaCannotAllowReplicas
            );

            ensure!(info.allow_replica, Error::<T>::ValueNotChanged);

            *ips_info = Some(IpInfo {
                parentage: info.parentage,
                metadata: info.metadata,
                data: info.data,
                ips_type: info.ips_type,
                allow_replica: false,

                supply: info.supply,

                license: info.license,
                execution_threshold: info.execution_threshold,
                default_asset_weight: info.default_asset_weight,
                default_permission: info.default_permission,
            });

            Self::deposit_event(Event::DisallowedReplica(ips_id));

            Ok(())
        })
    }

    pub(crate) fn inner_create_replica(
        owner: OriginFor<T>,
        original_ips_id: T::IpId,
        ipl_license: <T as Config>::Licenses,
        ipl_execution_threshold: OneOrPercent,
        ipl_default_asset_weight: OneOrPercent,
        ipl_default_permission: bool,
    ) -> DispatchResultWithPostInfo {
        NextIpId::<T>::try_mutate(|ips_id| -> DispatchResultWithPostInfo {
            let creator = ensure_signed(owner.clone())?;

            let original_ips =
                IpStorage::<T>::get(original_ips_id).ok_or(Error::<T>::IpsNotFound)?;

            ensure!(original_ips.allow_replica, Error::<T>::ReplicaNotAllowed);

            let current_id = *ips_id;
            *ips_id = ips_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableIpId)?;

            let ips_account =
                primitives::utils::multi_account_id::<T, <T as Config>::IpId>(current_id, None);

            pallet_balances::Pallet::<T>::transfer_keep_alive(
                owner.clone(),
                T::Lookup::unlookup(ips_account.clone()),
                <T as pallet_balances::Config>::ExistentialDeposit::get(),
            )?;

            let info = IpInfo {
                parentage: Parentage::Parent(ips_account.clone()),
                metadata: original_ips.metadata,
                data: original_ips.data,
                ips_type: IpsType::Replica(original_ips_id),
                allow_replica: false,

                supply: Zero::zero(),

                license: ipl_license.get_hash_and_metadata(),
                execution_threshold: ipl_execution_threshold,
                default_asset_weight: ipl_default_asset_weight,
                default_permission: ipl_default_permission,
            };

            Pallet::<T>::internal_mint(
                (current_id, None),
                creator,
                <T as Config>::ExistentialDeposit::get(),
            )?;

            IpStorage::<T>::insert(current_id, info);
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
