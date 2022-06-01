use super::pallet::*;
use frame_support::pallet_prelude::*;
use frame_system::ensure_signed;
use frame_system::pallet_prelude::*;
use primitives::{OneOrPercent, Parentage};

pub trait LicenseList<T: Config> {
    fn get_hash_and_metadata(
        &self,
    ) -> (
        BoundedVec<u8, <T as Config>::MaxMetadata>,
        <T as frame_system::Config>::Hash,
    );
}

impl<T: Config> Pallet<T> {
    pub(crate) fn inner_set_permission(
        owner: OriginFor<T>,
        ipl_id: T::IpId,
        sub_asset: T::IpId,
        call_metadata: [u8; 2],
        permission: bool,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let ip = IpStorage::<T>::get(ipl_id).ok_or(Error::<T>::IpDoesntExist)?;

        match ip.parentage.clone() {
            Parentage::Parent(ips_account) => {
                ensure!(ips_account == owner, Error::<T>::NoPermission)
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        Permissions::<T>::insert((ipl_id, sub_asset), call_metadata, permission);

        Self::deposit_event(Event::PermissionSet(
            ipl_id,
            sub_asset,
            call_metadata,
            permission,
        ));

        Ok(())
    }

    pub(crate) fn inner_set_asset_weight(
        owner: OriginFor<T>,
        ipl_id: T::IpId,
        sub_asset: T::IpId,
        asset_weight: OneOrPercent,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let ip = IpStorage::<T>::get(ipl_id).ok_or(Error::<T>::IpDoesntExist)?;

        match ip.parentage.clone() {
            Parentage::Parent(ips_account) => {
                ensure!(ips_account == owner, Error::<T>::NoPermission)
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        AssetWeight::<T>::insert(ipl_id, sub_asset, asset_weight);

        Self::deposit_event(Event::WeightSet(ipl_id, sub_asset, asset_weight));

        Ok(())
    }

    pub fn execution_threshold(ipl_id: T::IpId) -> Option<OneOrPercent> {
        IpStorage::<T>::get(ipl_id).map(|ipl| ipl.execution_threshold)
    }

    pub fn asset_weight(ipl_id: T::IpId, sub_asset: T::IpId) -> Option<OneOrPercent> {
        AssetWeight::<T>::get(ipl_id, sub_asset)
            .or_else(|| IpStorage::<T>::get(ipl_id).map(|ipl| ipl.default_asset_weight))
    }

    pub fn has_permission(
        ipl_id: T::IpId,
        sub_asset: T::IpId,
        call_metadata: [u8; 2],
    ) -> Option<bool> {
        Permissions::<T>::get((ipl_id, sub_asset), call_metadata)
            .or_else(|| IpStorage::<T>::get(ipl_id).map(|ipl| ipl.default_permission))
    }
}
