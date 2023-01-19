use super::pallet::*;
use frame_support::pallet_prelude::*;
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::OneOrPercent;

impl<T: Config> Pallet<T> {
    /// Set yes/no permission for a sub token to start/vote on a specific multisig call
    pub(crate) fn inner_set_permission(
        owner: OriginFor<T>,
        core_id: T::CoreId,
        sub_token_id: T::CoreId,
        call_index: [u8; 2],
        permission: bool,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let core = CoreStorage::<T>::get(core_id).ok_or(Error::<T>::CoreDoesntExist)?;

        ensure!(core.account == owner, Error::<T>::NoPermission);

        Permissions::<T>::insert((core_id, sub_token_id), call_index, permission);

        Self::deposit_event(Event::PermissionSet {
            core_id,
            sub_token_id,
            call_index,
            permission,
        });

        Ok(())
    }

    /// Set the voting weight for a sub token
    pub(crate) fn inner_set_sub_token_weight(
        owner: OriginFor<T>,
        core_id: T::CoreId,
        sub_token_id: T::CoreId,
        voting_weight: OneOrPercent,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let core = CoreStorage::<T>::get(core_id).ok_or(Error::<T>::CoreDoesntExist)?;

        ensure!(core.account == owner, Error::<T>::NoPermission);

        AssetWeight::<T>::insert(core_id, sub_token_id, voting_weight);

        Self::deposit_event(Event::WeightSet {
            core_id,
            sub_token_id,
            voting_weight,
        });

        Ok(())
    }

    /// Return `execution_threshold` setting for sub tokens in a given IP Set
    pub fn execution_threshold(core_id: T::CoreId) -> Option<OneOrPercent> {
        CoreStorage::<T>::get(core_id).map(|core| core.execution_threshold)
    }

    /// Get the voting weight for a sub token. If none is found, returns the default voting weight
    pub fn asset_weight(core_id: T::CoreId, sub_token_id: T::CoreId) -> Option<OneOrPercent> {
        AssetWeight::<T>::get(core_id, sub_token_id)
            .or_else(|| CoreStorage::<T>::get(core_id).map(|core| core.default_asset_weight))
    }

    /// Check if a sub token has permission to iniate/vote on an extrinsic via the multisig.
    /// `call_metadata`: 1st byte = pallet index, 2nd byte = function index
    pub fn has_permission(
        core_id: T::CoreId,
        sub_token_id: T::CoreId,
        call_index: [u8; 2],
    ) -> Result<bool, Error<T>> {
        Ok(
            Permissions::<T>::get((core_id, sub_token_id), call_index).unwrap_or(
                CoreStorage::<T>::get(core_id)
                    .map(|core| core.default_permission)
                    .ok_or(Error::<T>::CoreDoesntExist)?,
            ),
        )
    }
}
