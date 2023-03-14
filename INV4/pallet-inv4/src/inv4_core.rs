use super::pallet::*;
use crate::{
    multisig::FreezeAsset,
    origin::{ensure_multisig, INV4Origin},
    util::derive_core_account,
};
use frame_support::{
    pallet_prelude::*,
    traits::fungibles::{Create, Mutate},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::CoreInfo;
use sp_arithmetic::traits::{CheckedAdd, One};
use sp_runtime::Perbill;
use sp_std::{convert::TryInto, vec::Vec};

pub type CoreIndexOf<T> = <T as Config>::CoreId;

pub type CoreMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxMetadata>;

impl<T: Config> Pallet<T>
where
    Result<
        INV4Origin<T, <T as crate::pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
{
    /// Create IP Set
    pub(crate) fn inner_create_core(
        origin: OriginFor<T>,
        metadata: Vec<u8>,
        minimum_support: Perbill,
        required_approval: Perbill,
    ) -> DispatchResult {
        NextCoreId::<T>::try_mutate(|next_id| -> DispatchResult {
            let creator = ensure_signed(origin)?;

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

            T::AssetsProvider::create(current_id, core_account.clone(), true, One::one())?;

            T::AssetsProvider::mint_into(current_id, &creator, seed_balance)?;

            T::AssetFreezer::freeze_asset(current_id)?;

            let info = CoreInfo {
                account: core_account.clone(),
                metadata: bounded_metadata,
                minimum_support,
                required_approval,
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
        origin: OriginFor<T>,
        metadata: Option<Vec<u8>>,
        minimum_support: Option<Perbill>,
        required_approval: Option<Perbill>,
    ) -> DispatchResult {
        let core_origin = ensure_multisig::<T, OriginFor<T>>(origin)?;
        let core_id = core_origin.id;

        CoreStorage::<T>::try_mutate(core_id, |core| {
            let mut c = core.take().ok_or(Error::<T>::CoreNotFound)?;

            if let Some(ms) = minimum_support {
                c.minimum_support = ms;
            }

            if let Some(ra) = required_approval {
                c.required_approval = ra;
            }

            if let Some(m) = metadata {
                c.metadata = m.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?;
            }

            *core = Some(c);

            Ok(())
        })
    }
}
