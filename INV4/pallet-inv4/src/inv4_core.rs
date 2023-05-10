use super::pallet::*;
use crate::{
    fee_handling::{FeeAsset, FeeAssetNegativeImbalance, MultisigFeeHandler},
    origin::{ensure_multisig, INV4Origin},
    util::derive_core_account,
};
use frame_support::{
    pallet_prelude::*,
    traits::{
        fungibles::{Balanced, Mutate},
        Currency, ExistenceRequirement, WithdrawReasons,
    },
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
        creation_fee_asset: FeeAsset,
    ) -> DispatchResult {
        NextCoreId::<T>::try_mutate(|next_id| -> DispatchResult {
            let creator = ensure_signed(origin)?;

            let bounded_metadata: BoundedVec<u8, T::MaxMetadata> = metadata
                .clone()
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

            T::AssetsProvider::mint_into(current_id, &creator, seed_balance)?;

            let info = CoreInfo {
                account: core_account.clone(),
                metadata: bounded_metadata,
                minimum_support,
                required_approval,
                frozen_tokens: true,
            };

            T::FeeCharger::handle_creation_fee(match creation_fee_asset {
                FeeAsset::TNKR => {
                    FeeAssetNegativeImbalance::TNKR(<T as Config>::Currency::withdraw(
                        &creator,
                        T::CoreCreationFee::get(),
                        WithdrawReasons::TRANSACTION_PAYMENT,
                        ExistenceRequirement::KeepAlive,
                    )?)
                }

                FeeAsset::KSM => FeeAssetNegativeImbalance::KSM(<T as Config>::Tokens::withdraw(
                    T::KSMAssetId::get(),
                    &creator,
                    T::KSMCoreCreationFee::get(),
                )?),
            });

            // match creation_fee_asset {
            //     FeeAsset::TNKR => {
            //         T::CreationFeeHandler::on_unbalanced(<T as Config>::Currency::withdraw(
            //             &creator,
            //             T::CoreCreationFee::get(),
            //             WithdrawReasons::TRANSACTION_PAYMENT,
            //             ExistenceRequirement::KeepAlive,
            //         )?)
            //     }

            //     FeeAsset::KSM => {
            //         T::KSMCreationFeeHandler::on_unbalanced(<T as Config>::Tokens::withdraw(
            //             &creator,
            //             T::KSMCoreCreationFee::get(),
            //             WithdrawReasons::TRANSACTION_PAYMENT,
            //             ExistenceRequirement::KeepAlive,
            //         )?)
            //     }
            // }

            // Update core storage
            CoreStorage::<T>::insert(current_id, info);
            CoreByAccount::<T>::insert(core_account.clone(), current_id);

            Self::deposit_event(Event::CoreCreated {
                core_account,
                metadata,
                core_id: current_id,
                minimum_support,
                required_approval,
            });

            Ok(())
        })
    }

    pub(crate) fn inner_set_parameters(
        origin: OriginFor<T>,
        metadata: Option<Vec<u8>>,
        minimum_support: Option<Perbill>,
        required_approval: Option<Perbill>,
        frozen_tokens: Option<bool>,
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

            if let Some(m) = metadata.clone() {
                c.metadata = m.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?;
            }

            if let Some(f) = frozen_tokens {
                c.frozen_tokens = f;
            }

            *core = Some(c);

            Self::deposit_event(Event::ParametersSet {
                core_id,
                metadata,
                minimum_support,
                required_approval,
                frozen_tokens,
            });

            Ok(())
        })
    }

    pub fn is_asset_frozen(core_id: T::CoreId) -> Option<bool> {
        CoreStorage::<T>::get(core_id).map(|c| c.frozen_tokens)
    }
}
