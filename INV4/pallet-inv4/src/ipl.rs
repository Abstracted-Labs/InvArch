use super::pallet::*;
use frame_support::pallet_prelude::*;
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::{OneOrPercent, Parentage};

/// Trait for getting license information
pub trait LicenseList<T: Config> {
    fn get_hash_and_metadata(
        &self,
    ) -> (
        BoundedVec<u8, <T as Config>::MaxMetadata>,
        <T as frame_system::Config>::Hash,
    );
}

impl<T: Config> Pallet<T> {
    /// Set yes/no permission for a sub token to start/vote on a specific multisig call
    pub(crate) fn inner_set_permission(
        owner: OriginFor<T>,
        ips_id: T::CoreId,
        sub_token_id: T::CoreId,
        call_index: [u8; 2],
        permission: bool,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let ip = CoreStorage::<T>::get(ips_id).ok_or(Error::<T>::IpDoesntExist)?;

        // Only the top-level IP Set can set permissions
        match ip.parentage {
            Parentage::Parent(ips_account) => {
                ensure!(ips_account == owner, Error::<T>::NoPermission)
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        Permissions::<T>::insert((ips_id, sub_token_id), call_index, permission);

        Self::deposit_event(Event::PermissionSet {
            ips_id,
            sub_token_id,
            call_index,
            permission,
        });

        Ok(())
    }

    /// Set the voting weight for a sub token
    pub(crate) fn inner_set_sub_token_weight(
        owner: OriginFor<T>,
        ips_id: T::CoreId,
        sub_token_id: T::CoreId,
        voting_weight: OneOrPercent,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let ip = CoreStorage::<T>::get(ips_id).ok_or(Error::<T>::IpDoesntExist)?;

        // Only the top-level IP Set can set permissions
        match ip.parentage {
            Parentage::Parent(ips_account) => {
                ensure!(ips_account == owner, Error::<T>::NoPermission)
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        AssetWeight::<T>::insert(ips_id, sub_token_id, voting_weight);

        Self::deposit_event(Event::WeightSet {
            ips_id,
            sub_token_id,
            voting_weight,
        });

        Ok(())
    }

    /// Return `execution_threshold` setting for sub tokens in a given IP Set
    pub fn execution_threshold(ips_id: T::CoreId) -> Option<OneOrPercent> {
        CoreStorage::<T>::get(ips_id).map(|ips| ips.execution_threshold)
    }

    /// Get the voting weight for a sub token. If none is found, returns the default voting weight
    pub fn asset_weight(ips_id: T::CoreId, sub_token_id: T::CoreId) -> Option<OneOrPercent> {
        AssetWeight::<T>::get(ips_id, sub_token_id)
            .or_else(|| CoreStorage::<T>::get(ips_id).map(|ips| ips.default_asset_weight))
    }

    /// Check if a sub token has permission to iniate/vote on an extrinsic via the multisig.
    /// `call_metadata`: 1st byte = pallet index, 2nd byte = function index
    pub fn has_permission(
        ips_id: T::CoreId,
        sub_token_id: T::CoreId,
        call_index: [u8; 2],
    ) -> Result<bool, Error<T>> {
        Ok(
            Permissions::<T>::get((ips_id, sub_token_id), call_index).unwrap_or(
                CoreStorage::<T>::get(ips_id)
                    .map(|ips| ips.default_permission)
                    .ok_or(Error::<T>::IpDoesntExist)?,
            ),
        )
    }
}
