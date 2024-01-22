use super::pallet::*;
use crate::{
    account_derivation::CoreAccountDerivation,
    fee_handling::{FeeAsset, FeeAssetNegativeImbalance, MultisigFeeHandler},
    origin::{ensure_multisig, INV4Origin},
};
use frame_support::{
    pallet_prelude::*,
    traits::{
        fungibles::{Balanced, Mutate},
        tokens::{Fortitude, Precision, Preservation},
        Currency, ExistenceRequirement, WithdrawReasons,
    },
};
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::CoreInfo;
use sp_arithmetic::traits::{CheckedAdd, One};
use sp_runtime::Perbill;

pub type CoreIndexOf<T> = <T as Config>::CoreId;

pub type CoreMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxMetadata>;

impl<T: Config> Pallet<T>
where
    Result<INV4Origin<T>, <T as frame_system::Config>::RuntimeOrigin>:
        From<<T as frame_system::Config>::RuntimeOrigin>,
    <T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
    /// Inner function for the create_core call.
    pub(crate) fn inner_create_core(
        origin: OriginFor<T>,
        metadata: BoundedVec<u8, T::MaxMetadata>,
        minimum_support: Perbill,
        required_approval: Perbill,
        creation_fee_asset: FeeAsset,
    ) -> DispatchResult {
        NextCoreId::<T>::try_mutate(|next_id| -> DispatchResult {
            let creator = ensure_signed(origin)?;

            // Increment core id counter
            let current_id = *next_id;
            *next_id = next_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableCoreId)?;

            // Derive the account of this core based on the core id
            let core_account = Self::derive_core_account(current_id);

            // Mint base amount of voting token to the caller
            let seed_balance = <T as Config>::CoreSeedBalance::get();
            T::AssetsProvider::mint_into(current_id, &creator, seed_balance)?;

            // Build the structure of the new core
            // Tokens are set to frozen by default
            let info = CoreInfo {
                account: core_account.clone(),
                metadata: metadata.clone(),
                minimum_support,
                required_approval,
                frozen_tokens: true,
            };

            // Charge creation fee from the caller
            T::FeeCharger::handle_creation_fee(match creation_fee_asset {
                FeeAsset::Native => {
                    FeeAssetNegativeImbalance::Native(<T as Config>::Currency::withdraw(
                        &creator,
                        T::CoreCreationFee::get(),
                        WithdrawReasons::TRANSACTION_PAYMENT,
                        ExistenceRequirement::KeepAlive,
                    )?)
                }

                FeeAsset::Relay => {
                    FeeAssetNegativeImbalance::Relay(<T as Config>::Tokens::withdraw(
                        T::RelayAssetId::get(),
                        &creator,
                        T::RelayCoreCreationFee::get(),
                        Precision::Exact,
                        Preservation::Protect,
                        Fortitude::Force,
                    )?)
                }
            });

            // Update core storages
            CoreStorage::<T>::insert(current_id, info);
            CoreByAccount::<T>::insert(core_account.clone(), current_id);

            Self::deposit_event(Event::CoreCreated {
                core_account,
                metadata: metadata.to_vec(),
                core_id: current_id,
                minimum_support,
                required_approval,
            });

            Ok(())
        })
    }

    /// Inner function for the set_parameters call.
    pub(crate) fn inner_set_parameters(
        origin: OriginFor<T>,
        metadata: Option<BoundedVec<u8, T::MaxMetadata>>,
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
                c.metadata = m;
            }

            if let Some(f) = frozen_tokens {
                c.frozen_tokens = f;
            }

            *core = Some(c);

            Self::deposit_event(Event::ParametersSet {
                core_id,
                metadata: metadata.map(|m| m.to_vec()),
                minimum_support,
                required_approval,
                frozen_tokens,
            });

            Ok(())
        })
    }

    /// Checks if the voting asset is frozen.
    pub fn is_asset_frozen(core_id: T::CoreId) -> Option<bool> {
        CoreStorage::<T>::get(core_id).map(|c| c.frozen_tokens)
    }
}
