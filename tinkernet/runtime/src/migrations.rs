use frame_support::{dispatch::GetStorageVersion, traits::OnRuntimeUpgrade, weights::Weight};
use log::{info, warn};

pub mod new_core_account_derivation {
    use super::*;
    use crate::{common_types::CommonId, AccountId, Identity, Runtime, RuntimeOrigin, Vec, INV4};
    use pallet_inv4::{
        account_derivation::CoreAccountDerivation, CoreInfoOf, Pallet as INV4Pallet,
    };
    use sp_runtime::MultiAddress;
    use sp_std::boxed::Box;

    fn get_old_accounts() -> Vec<(AccountId, CommonId)> {
        pallet_inv4::CoreByAccount::<Runtime>::iter().collect()
    }

    fn migrate_inv4_storages(old_accounts: Vec<(AccountId, CommonId)>) {
        // let _ = pallet_inv4::CoreByAccount::<Runtime>::clear(1000, None);

        old_accounts.iter().for_each(|(old_acc, core_id)| {
            let new_account =
                <INV4Pallet<Runtime> as CoreAccountDerivation<Runtime>>::derive_core_account(
                    *core_id,
                );

            // pallet_inv4::CoreByAccount::<Runtime>::insert(new_account, core_id);

            pallet_inv4::CoreByAccount::<Runtime>::swap(old_acc, new_account);
        });

        pallet_inv4::CoreStorage::<Runtime>::translate(
            |core_id, core_data: CoreInfoOf<Runtime>| {
                let mut new_core = core_data;

                new_core.account =
                    <INV4Pallet<Runtime> as CoreAccountDerivation<Runtime>>::derive_core_account(
                        core_id,
                    );

                Some(new_core)
            },
        );
    }

    fn migrate_staking_storages(old_accounts: Vec<(AccountId, CommonId)>) {
        old_accounts.iter().for_each(|(old_acc, this_core_id)| {
            let new_account =
                <INV4Pallet<Runtime> as CoreAccountDerivation<Runtime>>::derive_core_account(
                    *this_core_id,
                );

            // let ledger = pallet_ocif_staking::pallet::Ledger::<Runtime>::take(old_acc);
            // pallet_ocif_staking::pallet::Ledger::<Runtime>::insert(new_account.clone(), ledger);

            pallet_ocif_staking::pallet::Ledger::<Runtime>::swap(old_acc, new_account.clone());

            pallet_inv4::CoreStorage::<Runtime>::iter_keys().for_each(|staking_core_id| {
                // let info = pallet_ocif_staking::pallet::GeneralStakerInfo::<Runtime>::take(
                //     staking_core_id,
                //     old_acc,
                // );

                // pallet_ocif_staking::pallet::GeneralStakerInfo::<Runtime>::insert(
                //     staking_core_id,
                //     new_account.clone(),
                //     info,
                // );

                pallet_ocif_staking::pallet::GeneralStakerInfo::<Runtime>::swap(
                    staking_core_id,
                    old_acc,
                    staking_core_id,
                    new_account.clone(),
                );
            });
        });
    }

    fn migrate_balances_storages(old_accounts: Vec<(AccountId, CommonId)>) {
        old_accounts.iter().for_each(|(old_acc, this_core_id)| {
            let new_account =
                <INV4Pallet<Runtime> as CoreAccountDerivation<Runtime>>::derive_core_account(
                    *this_core_id,
                );

            pallet_balances::Account::<Runtime>::swap(old_acc, new_account.clone());
            pallet_balances::Freezes::<Runtime>::swap(old_acc, new_account.clone());
            pallet_balances::Holds::<Runtime>::swap(old_acc, new_account.clone());
            pallet_balances::Locks::<Runtime>::swap(old_acc, new_account.clone());
            pallet_balances::Reserves::<Runtime>::swap(old_acc, new_account.clone());

            frame_system::Account::<Runtime>::swap(old_acc, new_account);
        });
    }

    fn migrate_identity_storages(old_accounts: Vec<(AccountId, CommonId)>) {
        old_accounts.iter().for_each(|(old_acc, this_core_id)| {
            let new_account =
                <INV4Pallet<Runtime> as CoreAccountDerivation<Runtime>>::derive_core_account(
                    *this_core_id,
                );

            let maybe_identity = Identity::identity(old_acc);

            if let Some(identity) = maybe_identity {
                let _ = Identity::kill_identity(
                    RuntimeOrigin::root(),
                    MultiAddress::Id(old_acc.clone()),
                );
                let _ = Identity::set_identity(
                    RuntimeOrigin::signed(new_account),
                    Box::new(identity.info),
                );
            }
        });
    }

    pub struct MigrateToNewDerivation;
    impl OnRuntimeUpgrade for MigrateToNewDerivation {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<sp_std::prelude::Vec<u8>, sp_runtime::DispatchError> {
            frame_support::ensure!(
                INV4::current_storage_version() == 2,
                "Required v2 before migrating core accounts"
            );

            Ok(Default::default())
        }

        fn on_runtime_upgrade() -> Weight {
            let current = INV4::current_storage_version();

            let old_accounts = get_old_accounts();

            if current == 2 {
                migrate_inv4_storages(old_accounts.clone());
                migrate_staking_storages(old_accounts.clone());
                migrate_balances_storages(old_accounts.clone());
                migrate_identity_storages(old_accounts.clone());

                info!("applied successfully");

                <Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, 1)
            } else {
                warn!("Skipping, should be removed");
                <Runtime as frame_system::Config>::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
            Ok(())
        }
    }
}
