use super::pallet::*;
use crate::util::derive_core_account;
use frame_support::pallet_prelude::*;
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::{CoreInfo, OneOrPercent};
use sp_arithmetic::traits::{CheckedAdd, One};
use sp_std::{convert::TryInto, vec::Vec};

pub type CoreIndexOf<T> = <T as Config>::CoreId;

pub type CoreMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxMetadata>;

impl<T: Config> Pallet<T> {
    /// Create IP Set
    pub(crate) fn inner_create_core(
        owner: OriginFor<T>,
        metadata: Vec<u8>,
        execution_threshold: OneOrPercent,
        default_asset_weight: OneOrPercent,
        default_permission: bool,
    ) -> DispatchResult {
        NextCoreId::<T>::try_mutate(|next_id| -> DispatchResult {
            let creator = ensure_signed(owner.clone())?;

            let bounded_metadata: BoundedVec<u8, T::MaxMetadata> = metadata
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

            // Increment counter
            let current_id = *next_id;
            *next_id = next_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableCoreId)?;

            // Generate new `AccountId` to represent new IP Set being created
            let core_account = derive_core_account::<
                T,
                <T as Config>::CoreId,
                <T as frame_system::Config>::AccountId,
            >(current_id);

            let seed_balance = <T as Config>::CoreSeedBalance::get();

            // Send IP Set `creator` 1,000,000 "IPT0" tokens
            // Token has 6 decimal places: 1,000,000 / 10^6 = 1 IPTO token
            // This allows for token divisiblity
            Balances::<T>::insert((current_id, None::<T::CoreId>, creator), seed_balance);

            TotalIssuance::<T>::insert(current_id, None::<T::CoreId>, seed_balance);

            let info = CoreInfo {
                account: core_account.clone(),
                metadata: bounded_metadata,

                execution_threshold,
                default_asset_weight,
                default_permission,
            };

            // Update core IPS storage
            CoreStorage::<T>::insert(current_id, info);
            CoreByAccount::<T>::insert(core_account.clone(), current_id);

            Self::deposit_event(Event::CoreCreated {
                core_account,
                core_id: current_id,
            });

            Ok(())
        })
    }

    pub(crate) fn inner_set_parameters(
        owner: OriginFor<T>,
        core_id: T::CoreId,
        execution_threshold: Option<OneOrPercent>,
        default_asset_weight: Option<OneOrPercent>,
        default_permission: Option<bool>,
    ) -> DispatchResult {
        let signer = ensure_signed(owner)?;

        CoreStorage::<T>::try_mutate(core_id, |core| {
            let mut c = core.take().ok_or(Error::<T>::CoreNotFound)?;

            ensure!(c.account == signer, Error::<T>::NoPermission);

            if let Some(et) = execution_threshold {
                c.execution_threshold = et;
            }

            if let Some(daw) = default_asset_weight {
                c.default_asset_weight = daw;
            }

            if let Some(dp) = default_permission {
                c.default_permission = dp;
            }

            *core = Some(c);

            Ok(())
        })
    }
}
