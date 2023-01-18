use super::pallet::*;
use crate::util::derive_ips_account;
use frame_support::pallet_prelude::*;
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::{CoreInfo, OneOrPercent, Parentage};
use sp_arithmetic::traits::{CheckedAdd, One};
use sp_std::{convert::TryInto, vec::Vec};

pub type CoreIndexOf<T> = <T as Config>::CoreId;

pub type CoreMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxMetadata>;

impl<T: Config> Pallet<T> {
    /// Create IP Set
    pub(crate) fn inner_create_ips(
        owner: OriginFor<T>,
        metadata: Vec<u8>,
        ipl_execution_threshold: OneOrPercent,
        ipl_default_asset_weight: OneOrPercent,
        ipl_default_permission: bool,
    ) -> DispatchResult {
        NextCoreId::<T>::try_mutate(|ips_id| -> DispatchResult {
            let creator = ensure_signed(owner.clone())?;

            let bounded_metadata: BoundedVec<u8, T::MaxMetadata> = metadata
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

            // Increment counter
            let current_id = *ips_id;
            *ips_id = ips_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableCoreId)?;

            // Generate new `AccountId` to represent new IP Set being created
            let ips_account = derive_ips_account::<
                T,
                <T as Config>::CoreId,
                <T as frame_system::Config>::AccountId,
            >(current_id, None);

            // Send IP Set `creator` 1,000,000 "IPT0" tokens
            // Token has 6 decimal places: 1,000,000 / 10^6 = 1 IPTO token
            // This allows for token divisiblity
            Balance::<T>::insert::<
                (<T as Config>::CoreId, Option<<T as Config>::CoreId>),
                T::AccountId,
                <T as Config>::Balance,
            >((current_id, None), creator, 1_000_000u128.into());

            let info = CoreInfo {
                parentage: Parentage::Parent(ips_account.clone()),
                metadata: bounded_metadata,

                supply: 1_000_000u128.into(),

                execution_threshold: ipl_execution_threshold,
                default_asset_weight: ipl_default_asset_weight,
                default_permission: ipl_default_permission,
            };

            // Update core IPS storage
            CoreStorage::<T>::insert(current_id, info);
            CoreByAccount::<T>::insert(ips_account.clone(), current_id);

            Self::deposit_event(Event::IPSCreated {
                ips_account,
                ips_id: current_id,
            });

            Ok(())
        })
    }
}
