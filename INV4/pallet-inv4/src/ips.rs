use super::pallet::*;
use crate::ipl::LicenseList;
use frame_support::pallet_prelude::*;
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::{utils::multi_account_id, IpInfo, IpsType, OneOrPercent, Parentage};
use rmrk_traits::{Collection, Nft};
use sp_arithmetic::traits::{CheckedAdd, One, Zero};
use sp_runtime::traits::StaticLookup;
use sp_std::{convert::TryInto, vec::Vec};

pub type IpsIndexOf<T> = <T as Config>::IpId;

pub type IpsMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxMetadata>;

impl<T: Config> Pallet<T> {
    /// Create IP Set
    pub(crate) fn inner_create_ips(
        owner: OriginFor<T>,
        metadata: Vec<u8>,
        assets: Vec<AnyIdOf<T>>,
        allow_replica: bool,
        ipl_license: <T as Config>::Licenses,
        ipl_execution_threshold: OneOrPercent,
        ipl_default_asset_weight: OneOrPercent,
        ipl_default_permission: bool,
    ) -> DispatchResult {
        // IPS inside IPS disabled for now. Needs rewrite.
        ensure!(
            !assets
                .clone()
                .into_iter()
                .any(|id| { matches!(id, AnyId::IpsId(_)) }),
            Error::<T>::IpsInsideIpsDisabled
        );

        NextIpId::<T>::try_mutate(|ips_id| -> DispatchResult {
            let creator = ensure_signed(owner.clone())?;

            let bounded_metadata: BoundedVec<u8, T::MaxMetadata> = metadata
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

            // Increment counter
            let current_id = *ips_id;
            *ips_id = ips_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableIpId)?;

            // Verify `creator` has permission to add each item in `assets` to new IP Set
            for asset in assets.clone() {
                match asset {
                    AnyId::IpsId(_) => (),
                    AnyId::IpfId(ipf_id) => {
                        ensure!(
                            ipf::IpfStorage::<T>::get(ipf_id)
                                .ok_or(Error::<T>::IpfNotFound)?
                                .owner
                                == creator,
                            Error::<T>::NoPermission
                        );
                    }
                    AnyId::RmrkNft((collection_id, nft_id)) => {
                        let rmrk_nft = pallet_rmrk_core::Nfts::<T>::get(collection_id, nft_id)
                            .ok_or(Error::<T>::IpfNotFound)?;

                        ensure!(
                            rmrk_nft.owner
                                == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                                    creator.clone()
                                ),
                            Error::<T>::NoPermission
                        );

                        ensure!(rmrk_nft.transferable, Error::<T>::NoPermission);
                    }
                    AnyId::RmrkCollection(collection_id) => {
                        ensure!(
                            pallet_rmrk_core::Collections::<T>::get(collection_id)
                                .ok_or(Error::<T>::IpfNotFound)?
                                .issuer
                                == creator.clone(),
                            Error::<T>::NoPermission
                        );
                    }
                }
            }

            // Generate new `AccountId` to represent new IP Set being created
            let ips_account =
                primitives::utils::multi_account_id::<T, <T as Config>::IpId>(current_id, None);

            // Transfer ownership (issuer for `RmrkCollection`) to `ips_account` for each item in `assets`
            for asset in assets.clone() {
                match asset {
                    AnyId::IpsId(_) => (),
                    AnyId::IpfId(ipf_id) => {
                        ipf::Pallet::<T>::send(creator.clone(), ipf_id, ips_account.clone())?
                    }
                    AnyId::RmrkNft((collection_id, nft_id)) => {
                        pallet_rmrk_core::Pallet::<T>::nft_send(
                            creator.clone(),
                            collection_id,
                            nft_id,
                            rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                                ips_account.clone(),
                            ),
                        )?;
                    }
                    AnyId::RmrkCollection(collection_id) => {
                        pallet_rmrk_core::Pallet::<T>::collection_change_issuer(
                            collection_id,
                            ips_account.clone(),
                        )?;
                    }
                }
            }

            // `ips_account` needs the existential deposit, so we send that
            pallet_balances::Pallet::<T>::transfer_keep_alive(
                owner.clone(),
                T::Lookup::unlookup(ips_account.clone()),
                <T as pallet_balances::Config>::ExistentialDeposit::get(),
            )
            .map_err(|error_with_post_info| error_with_post_info.error)?;

            // Send IP Set `creator` 1,000,000 "IPT0" tokens
            // Token has 6 decimal places: 1,000,000 / 10^6 = 1 IPTO token
            // This allows for token divisiblity
            Balance::<T>::insert::<
                (<T as Config>::IpId, Option<<T as Config>::IpId>),
                T::AccountId,
                <T as Config>::Balance,
            >((current_id, None), creator, 1_000_000u128.into());

            let info = IpInfo {
                parentage: Parentage::Parent(ips_account.clone()),
                metadata: bounded_metadata,
                data: assets
                    .try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?,
                ips_type: IpsType::Normal,
                allow_replica,

                supply: 1000000u128.into(),

                license: ipl_license.get_hash_and_metadata(),
                execution_threshold: ipl_execution_threshold,
                default_asset_weight: ipl_default_asset_weight,
                default_permission: ipl_default_permission,
            };

            // Update core IPS storage
            IpStorage::<T>::insert(current_id, info);
            IpsByOwner::<T>::insert(ips_account.clone(), current_id, ());

            Self::deposit_event(Event::Created(ips_account, current_id));

            Ok(())
        })
    }

    /// Append new assets to an IP Set
    pub(crate) fn inner_append(
        owner: OriginFor<T>,
        ips_id: T::IpId,
        assets: Vec<AnyIdOf<T>>,
        new_metadata: Option<Vec<u8>>,
    ) -> DispatchResult {
        IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
            let caller_account = ensure_signed(owner.clone())?;

            // IPS inside IPS disabled for now. Needs rewrite.
            ensure!(
                !assets
                    .clone()
                    .into_iter()
                    .any(|id| { matches!(id, AnyId::IpsId(_)) }),
                Error::<T>::IpsInsideIpsDisabled
            );

            let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

            let parent_id = ips_id;

            // Get highest level IPS `AccountId` in the hierarchy
            let ips_account = match info.parentage.clone() {
                Parentage::Parent(ips_account) => ips_account,
                Parentage::Child(_, absolute_parent_account) => absolute_parent_account,
            };

            ensure!(
                !assets.is_empty() || new_metadata.is_some(),
                Error::<T>::ValueNotChanged
            );

            // Verify valid permission to add each item in `assets` to IP Set
            for asset in assets.clone() {
                match asset {
                    AnyId::IpsId(_) => (),
                    // {
                    //     if let Parentage::Parent(acc) = IpStorage::<T>::get(ips_id)
                    //         .ok_or(Error::<T>::IpsNotFound)?
                    //         .parentage
                    //     {
                    //         ensure!(
                    //             caller_account
                    //                 == multi_account_id::<T, T::IpId>(parent_id, Some(acc)),
                    //             Error::<T>::NoPermission
                    //         );
                    //     } else {
                    //         return Err(Error::<T>::NotParent.into());
                    //     }
                    // }
                    AnyId::IpfId(ipf_id) => {
                        let this_ipf_owner = ipf::IpfStorage::<T>::get(ipf_id)
                            .ok_or(Error::<T>::IpfNotFound)?
                            .owner;

                        // Ensure: either it's the IP Set itself or it's the IP Set with the include_caller option from multisig.
                        // We need that second one so we can allow someone to start a multisig call to include assets
                        // that they own without manually sending to the IPS and then starting a multisig
                        ensure!(
                            this_ipf_owner.clone() == ips_account
                                || caller_account
                                    == multi_account_id::<T, T::IpId>(
                                        parent_id,
                                        Some(this_ipf_owner.clone())
                                    ),
                            Error::<T>::NoPermission
                        );
                    }
                    AnyId::RmrkNft((collection_id, nft_id)) => {
                        let this_rmrk_nft = pallet_rmrk_core::Nfts::<T>::get(collection_id, nft_id)
                            .ok_or(Error::<T>::IpfNotFound)?;
                        let this_rmrk_owner = this_rmrk_nft.owner;
                        ensure!(
                            this_rmrk_owner.clone()
                                == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                                    ips_account.clone()
                                )
                                || if let rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                                    rmrk_owner_account,
                                ) = this_rmrk_owner.clone()
                                {
                                    caller_account
                                        == multi_account_id::<T, T::IpId>(
                                            parent_id,
                                            Some(rmrk_owner_account),
                                        )
                                } else {
                                    false
                                },
                            Error::<T>::NoPermission
                        );

                        ensure!(this_rmrk_nft.transferable, Error::<T>::NoPermission);
                    }
                    AnyId::RmrkCollection(collection_id) => {
                        let this_rmrk_issuer =
                            pallet_rmrk_core::Collections::<T>::get(collection_id)
                                .ok_or(Error::<T>::IpfNotFound)?
                                .issuer;
                        // Ensure IP Set is already owner(issuer) of NFT collection or
                        // initater of multisig call with `include_caller` is the owner(issuer)
                        ensure!(
                            this_rmrk_issuer.clone() == ips_account.clone()
                                || caller_account
                                    == multi_account_id::<T, T::IpId>(
                                        parent_id,
                                        Some(this_rmrk_issuer),
                                    ),
                            Error::<T>::NoPermission
                        );
                    }
                }
            }

            // Permissions have been verified, now send all assets to `ips_account`
            for asset in assets.clone() {
                match asset {
                    AnyId::IpsId(_) => (),
                    AnyId::IpfId(ipf_id) => ipf::Pallet::<T>::send(
                        ipf::IpfStorage::<T>::get(ipf_id)
                            .ok_or(Error::<T>::IpfNotFound)?
                            .owner,
                        ipf_id,
                        ips_account.clone(),
                    )?,
                    AnyId::RmrkNft((collection_id, nft_id)) => {
                        if let rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                            rmrk_owner_account,
                        ) = pallet_rmrk_core::Nfts::<T>::get(collection_id, nft_id)
                            .ok_or(Error::<T>::IpfNotFound)?
                            .owner
                        {
                            pallet_rmrk_core::Pallet::<T>::nft_send(
                                rmrk_owner_account,
                                collection_id,
                                nft_id,
                                rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
                                    ips_account.clone(),
                                ),
                            )?;
                        }
                    }
                    AnyId::RmrkCollection(collection_id) => {
                        pallet_rmrk_core::Pallet::<T>::collection_change_issuer(
                            collection_id,
                            ips_account.clone(),
                        )?;
                    }
                }
            }

            // for any_id in assets.clone().into_iter() {
            //     if let AnyId::IpsId(ips_id) = any_id {
            //         IpStorage::<T>::try_mutate_exists(ips_id, |ips| -> DispatchResult {
            //             let old = ips.take().ok_or(Error::<T>::IpsNotFound)?;

            //             let prefix: (<T as Config>::IpId, Option<<T as Config>::IpId>) =
            //                 (ips_id.into(), None);
            //             for (account, amount) in Balance::<T>::iter_prefix(prefix) {
            //                 let id: (<T as Config>::IpId, Option<<T as Config>::IpId>) =
            //                     (parent_id.into(), None);
            //                 Pallet::<T>::internal_mint(id, account.clone(), amount)?;
            //                 Pallet::<T>::internal_burn(account, prefix, amount)?;
            //             }

            //             *ips = Some(IpInfo {
            //                 parentage: Parentage::Child(parent_id, ips_account.clone()),
            //                 metadata: old.metadata,
            //                 data: old.data,
            //                 ips_type: old.ips_type,
            //                 allow_replica: old.allow_replica,

            //                 supply: old.supply,

            //                 license: old.license,
            //                 execution_threshold: old.execution_threshold,
            //                 default_asset_weight: old.default_asset_weight,
            //                 default_permission: old.default_permission,
            //             });

            //             Ok(())
            //         })?;
            //     }
            // }

            // Update IpInfo struct in storage to hold either new assets, new metadata, or both
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
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?,
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

    /// Remove an asset/assets from an IP Set
    pub(crate) fn inner_remove(
        owner: OriginFor<T>,
        ips_id: T::IpId,
        assets: Vec<AnyIdWithNewOwner<T>>,
        new_metadata: Option<Vec<u8>>,
    ) -> DispatchResult {
        IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
            let caller_account = ensure_signed(owner.clone())?;

            // IPS inside IPS disabled for now. Needs rewrite.
            ensure!(
                !assets
                    .clone()
                    .into_iter()
                    .any(|id| { matches!(id.0, AnyId::IpsId(_)) }),
                Error::<T>::IpsInsideIpsDisabled
            );

            let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

            let ips_account = match info.parentage.clone() {
                Parentage::Parent(ips_account) => ips_account,
                Parentage::Child(_, absolute_parent_account) => absolute_parent_account,
            };

            // Only IP Set can remove assets from itself
            ensure!(ips_account == caller_account, Error::<T>::NoPermission);

            // Are any of the assets requested for removal, not in the IP Set?
            ensure!(
                !assets
                    .clone()
                    .into_iter()
                    .any(|id| { !info.data.contains(&id.0) }),
                Error::<T>::NoPermission
            );

            let mut old_assets = info.data.clone();

            // Checks passed, now send requested assets to new owners
            for any_id in assets.clone().into_iter() {
                match any_id {
                    // Don't do anything. Nested IPS needs rewrite
                    (AnyId::IpsId(_this_ips_id), _new_owner) => (),
                    // {
                    //     IpStorage::<T>::try_mutate_exists(this_ips_id, |ips| -> DispatchResult {
                    //         let id: (<T as Config>::IpId, Option<<T as Config>::IpId>) =
                    //             (this_ips_id.into(), None);
                    //         Pallet::<T>::internal_mint(
                    //             id,
                    //             new_owner,
                    //             <T as Config>::ExistentialDeposit::get(),
                    //         )?;

                    //         ips.clone().unwrap().parentage =
                    //             Parentage::Parent(multi_account_id::<T, T::IpId>(
                    //                 this_ips_id,
                    //                 None,
                    //             ));

                    //         Ok(())
                    //     })?;
                    // }
                    (AnyId::IpfId(this_ipf_id), new_owner) => {
                        ipf::Pallet::<T>::send(ips_account.clone(), this_ipf_id, new_owner)?
                    }

                    (AnyId::RmrkNft((collection_id, nft_id)), new_owner) => {
                        pallet_rmrk_core::Pallet::<T>::nft_send(
                            ips_account.clone(),
                            collection_id,
                            nft_id,
                            rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(new_owner),
                        )?;
                    }
                    (AnyId::RmrkCollection(collection_id), new_owner) => {
                        pallet_rmrk_core::Pallet::<T>::collection_change_issuer(
                            collection_id,
                            new_owner.clone(),
                        )?;
                    }
                }
            }

            // Extract `AnyIdOf`'s from `AnyIdWithNewOwner`'s tuples
            // Then remove all assets from `old_assets` that were transferred out of the IP Set
            let just_ids = assets
                .clone()
                .into_iter()
                .map(|(x, _)| x)
                .collect::<Vec<AnyIdOf<T>>>();
            old_assets.retain(|x| !just_ids.clone().contains(x));

            // Update IP Set info struct in storage
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

    /// Allow replication for the specified IP Set
    pub(crate) fn inner_allow_replica(owner: OriginFor<T>, ips_id: T::IpId) -> DispatchResult {
        IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
            let owner = ensure_signed(owner)?;
            let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

            // Only the top-level IP Set can update the allow replica feature
            match info.parentage.clone() {
                Parentage::Parent(ips_account) => {
                    ensure!(ips_account == owner, Error::<T>::NoPermission)
                }
                Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
            }

            // Can only activate feature if not already activated
            ensure!(!info.allow_replica, Error::<T>::ValueNotChanged);

            // Only `Normal` (original) IP Sets can activate this feature, not `Replica`s
            ensure!(
                !matches!(info.ips_type, IpsType::Replica(_)),
                Error::<T>::ReplicaCannotAllowReplicas
            );

            // Checks passed, now update IP Set info struct in storage
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

    /// Disallow replication for the specified IP Set
    pub(crate) fn inner_disallow_replica(owner: OriginFor<T>, ips_id: T::IpId) -> DispatchResult {
        IpStorage::<T>::try_mutate_exists(ips_id, |ips_info| -> DispatchResult {
            let owner = ensure_signed(owner)?;
            let info = ips_info.take().ok_or(Error::<T>::IpsNotFound)?;

            // Only the top-level IP Set can update the allow replica feature
            match info.parentage.clone() {
                Parentage::Parent(ips_account) => {
                    ensure!(ips_account == owner, Error::<T>::NoPermission)
                }
                Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
            }

            // Only `Normal` (original) IP Sets can deactivate this feature, not `Replica`s
            ensure!(
                !matches!(info.ips_type, IpsType::Replica(_)),
                Error::<T>::ReplicaCannotAllowReplicas
            );

            // Can only deactivate feature if not already deactivated
            ensure!(info.allow_replica, Error::<T>::ValueNotChanged);

            // Checks passed, now update IP Set info struct in storage
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

    /// DISABLED
    pub(crate) fn _inner_create_replica(
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

            // Replication must be allowed
            ensure!(original_ips.allow_replica, Error::<T>::ReplicaNotAllowed);

            let current_id = *ips_id;
            // Increment counter
            *ips_id = ips_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableIpId)?;

            // Generate new `AccountId` to represent new IP Set being created
            let ips_account =
                primitives::utils::multi_account_id::<T, <T as Config>::IpId>(current_id, None);

            // `ips_account` needs the existential deposit, so we send that
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

            // ???
            Pallet::<T>::internal_mint(
                (current_id, None),
                creator,
                <T as Config>::ExistentialDeposit::get(),
            )?;

            // Update core IPS storage
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
