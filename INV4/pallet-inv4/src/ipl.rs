use super::{
    origin::{ensure_multisig, INV4Origin},
    pallet::{self, *},
    util::derive_ips_account,
};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use primitives::{OneOrPercent, Parentage};

impl<T: Config> Pallet<T>
where
    Result<
        INV4Origin<T, <T as pallet::Config>::IpId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::Origin,
    >: From<<T as frame_system::Config>::Origin>,
{
    /// Set yes/no permission for a sub token to start/vote on a specific multisig call
    pub(crate) fn inner_set_permission(
        origin: OriginFor<T>,
        sub_token_id: T::IpId,
        call_index: [u8; 2],
        permission: bool,
    ) -> DispatchResult {
        let ip_set = ensure_multisig::<T, OriginFor<T>>(origin)?;

        let ip = IpStorage::<T>::get(ip_set.id).ok_or(Error::<T>::IpDoesntExist)?;

        // Only the top-level IP Set can set permissions
        match ip.parentage {
            Parentage::Parent(ips_account) => {
                ensure!(
                    ips_account
                        == derive_ips_account::<
                            T,
                            <T as pallet::Config>::IpId,
                            <T as frame_system::Config>::AccountId,
                        >(ip_set.id, None),
                    Error::<T>::NoPermission
                )
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        Permissions::<T>::insert((ip_set.id, sub_token_id), call_index, permission);

        Self::deposit_event(Event::PermissionSet {
            ips_id: ip_set.id,
            sub_token_id,
            call_index,
            permission,
        });

        Ok(())
    }

    /// Set the voting weight for a sub token
    pub(crate) fn inner_set_sub_token_weight(
        origin: OriginFor<T>,
        sub_token_id: T::IpId,
        voting_weight: OneOrPercent,
    ) -> DispatchResult {
        let ip_set = ensure_multisig::<T, OriginFor<T>>(origin)?;

        let ip = IpStorage::<T>::get(ip_set.id).ok_or(Error::<T>::IpDoesntExist)?;

        // Only the top-level IP Set can set permissions
        match ip.parentage {
            Parentage::Parent(ips_account) => {
                ensure!(
                    ips_account
                        == derive_ips_account::<
                            T,
                            <T as pallet::Config>::IpId,
                            <T as frame_system::Config>::AccountId,
                        >(ip_set.id, None),
                    Error::<T>::NoPermission
                )
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        AssetWeight::<T>::insert(ip_set.id, sub_token_id, voting_weight);

        Self::deposit_event(Event::WeightSet {
            ips_id: ip_set.id,
            sub_token_id,
            voting_weight,
        });

        Ok(())
    }

    /// Return `execution_threshold` setting for sub tokens in a given IP Set
    pub fn execution_threshold(ips_id: T::IpId) -> Option<OneOrPercent> {
        IpStorage::<T>::get(ips_id).map(|ips| ips.execution_threshold)
    }

    /// Get the voting weight for a sub token. If none is found, returns the default voting weight
    pub fn asset_weight(ips_id: T::IpId, sub_token_id: T::IpId) -> Option<OneOrPercent> {
        AssetWeight::<T>::get(ips_id, sub_token_id)
            .or_else(|| IpStorage::<T>::get(ips_id).map(|ips| ips.default_asset_weight))
    }

    /// Check if a sub token has permission to iniate/vote on an extrinsic via the multisig.
    /// `call_metadata`: 1st byte = pallet index, 2nd byte = function index
    pub fn has_permission(
        ips_id: T::IpId,
        sub_token_id: T::IpId,
        call_index: [u8; 2],
    ) -> Result<bool, Error<T>> {
        Ok(
            Permissions::<T>::get((ips_id, sub_token_id), call_index).unwrap_or(
                IpStorage::<T>::get(ips_id)
                    .map(|ips| ips.default_permission)
                    .ok_or(Error::<T>::IpDoesntExist)?,
            ),
        )
    }
}
