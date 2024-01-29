#![allow(clippy::type_complexity)]

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{info, warn};

pub mod new_core_account_derivation {
    use super::*;
    use crate::{common_types::CommonId, AccountId, Identity, Runtime, RuntimeOrigin, Vec};
    use frame_support::dispatch::GetDispatchInfo;
    use pallet_identity::IdentityInfo;
    use pallet_inv4::{
        account_derivation::CoreAccountDerivation, CoreInfoOf, Pallet as INV4Pallet,
    };
    use sp_std::boxed::Box;

    fn get_old_accounts() -> Vec<(AccountId, CommonId)> {
        pallet_inv4::CoreByAccount::<Runtime>::iter().collect()
    }

    fn migrate_inv4_storages(old_accounts: Vec<(AccountId, CommonId)>) {
        old_accounts.iter().for_each(|(old_acc, core_id)| {
            let new_account =
                <INV4Pallet<Runtime> as CoreAccountDerivation<Runtime>>::derive_core_account(
                    *core_id,
                );

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

            pallet_ocif_staking::pallet::Ledger::<Runtime>::swap(old_acc, new_account.clone());

            pallet_inv4::CoreStorage::<Runtime>::iter_keys().for_each(|staking_core_id| {
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

    fn clear_identities(
        old_accounts: Vec<(AccountId, CommonId)>,
    ) -> Vec<(
        AccountId,
        Option<
            pallet_identity::Registration<
                crate::Balance,
                crate::MaxRegistrars,
                crate::MaxAdditionalFields,
            >,
        >,
    )> {
        old_accounts
            .iter()
            .map(|(old_acc, this_core_id)| {
                let new_account =
                    <INV4Pallet<Runtime> as CoreAccountDerivation<Runtime>>::derive_core_account(
                        *this_core_id,
                    );

                let maybe_identity = Identity::identity(old_acc);

                if maybe_identity.is_some() {
                    let _ = Identity::clear_identity(RuntimeOrigin::signed(old_acc.clone()));
                }

                (new_account, maybe_identity)
            })
            .collect::<Vec<(
                AccountId,
                Option<
                    pallet_identity::Registration<
                        crate::Balance,
                        crate::MaxRegistrars,
                        crate::MaxAdditionalFields,
                    >,
                >,
            )>>()
    }

    fn set_new_identities(
        new_identities: Vec<(
            AccountId,
            Option<
                pallet_identity::Registration<
                    crate::Balance,
                    crate::MaxRegistrars,
                    crate::MaxAdditionalFields,
                >,
            >,
        )>,
    ) {
        new_identities
            .into_iter()
            .for_each(|(new_account, maybe_identity)| {
                if let Some(identity) = maybe_identity {
                    let _ = Identity::set_identity(
                        RuntimeOrigin::signed(new_account.clone()),
                        Box::new(identity.info),
                    );
                }
            })
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
            let spec = crate::System::runtime_version().spec_version;

            if spec == 21 {
                let old_accounts = get_old_accounts();
                let old_accounts_len = old_accounts.len() as u64;

                migrate_inv4_storages(old_accounts.clone());
                migrate_staking_storages(old_accounts.clone());
                let new_identities = clear_identities(old_accounts.clone());
                migrate_balances_storages(old_accounts.clone());
                set_new_identities(new_identities.clone());

                info!("applied successfully");

                let clear_identities_weight = <Runtime as frame_system::Config>::DbWeight::get()
                    .reads(old_accounts_len)
                    + (pallet_identity::Call::<Runtime>::clear_identity {}
                        .get_dispatch_info()
                        .weight
                        * old_accounts_len);

                let set_new_identities_weight = pallet_identity::Call::<Runtime>::set_identity {
                    info: Box::new(IdentityInfo {
                        additional: Default::default(),
                        display: Default::default(),
                        legal: Default::default(),
                        riot: Default::default(),
                        web: Default::default(),
                        twitter: Default::default(),
                        email: Default::default(),
                        pgp_fingerprint: Default::default(),
                        image: Default::default(),
                    }),
                }
                .get_dispatch_info()
                .weight
                    * (new_identities.clone().len() as u64);

                clear_identities_weight
                    + set_new_identities_weight
                    + <Runtime as frame_system::Config>::DbWeight::get().reads(2)
                    + <Runtime as frame_system::Config>::DbWeight::get()
                        .reads_writes(old_accounts_len, old_accounts_len * 2)
                    + <Runtime as frame_system::Config>::DbWeight::get()
                        .reads_writes(old_accounts_len, old_accounts_len * 2)
                    + <Runtime as frame_system::Config>::DbWeight::get()
                        .writes(old_accounts_len * 6)
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
