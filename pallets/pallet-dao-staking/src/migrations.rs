use super::*;
use frame_support::{
    pallet_prelude::GetStorageVersion,
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};
use log::{info, warn};

pub mod v1 {

    use super::*;

    /// This will check all the info on the ledger and remove the old lock while reapplying the new lock based on the
    /// value of the ledger, so the wrogly locked tokens will be unlocked.
    pub fn migrate_locks_to_freeze<T: Config>() -> Weight {
        let mut weight = Weight::zero();
        let mut count: u32 = 0;

        Ledger::<T>::iter().for_each(|(account, ledger)| {
            if ledger.locked > Zero::zero() {
                <T as pallet::Config>::OldCurrency::remove_lock(LOCK_ID, &account);

                let set_freeze_result =
                    <T as pallet::Config>::Currency::set_freeze(&LOCK_ID, &account, ledger.locked);

                if set_freeze_result.is_err() {
                    warn!("set_freeze_result {:?}", set_freeze_result);
                }

                weight.saturating_accrue(T::DbWeight::get().reads_writes(3, 2));
                count += 1;
            }
        });

        info!("Migrated {} locks", count);
        weight
    }

    /// This will just remove all the holds on the enabling staking on a dao to apply the new one.
    pub fn migrate_holds<T: Config>() -> Weight {
        let mut count: u32 = 0;
        let mut weight = Weight::zero();

        RegisteredCore::<T>::iter().for_each(|(_dao_id, dao_info)| {
            let dao_account = dao_info.account;
            let dao_reserved = <T as Config>::OldCurrency::reserved_balance(&dao_account);

            <T as Config>::OldCurrency::unreserve(&dao_account, dao_reserved);

            let set_on_hold_result = <T as Config>::Currency::set_on_hold(
                &HoldReason::DaoStaking.into(),
                &dao_account,
                dao_reserved,
            );

            if set_on_hold_result.is_err() {
                warn!("set_on_hold_result {:?}", set_on_hold_result);
            }

            count += 1;
            weight.saturating_accrue(T::DbWeight::get().reads_writes(3, 2));
        });

        info!("Migrated {} daos", count);
        weight
    }

    pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<sp_runtime::Vec<u8>, sp_runtime::DispatchError> {
            frame_support::ensure!(
                Pallet::<T>::on_chain_storage_version() <= Pallet::<T>::in_code_storage_version(),
                "Required v0 before upgrading to v1"
            );

            Ok(Default::default())
        }

        fn on_runtime_upgrade() -> Weight {
            let mut weight = Weight::zero();
            let current = Pallet::<T>::in_code_storage_version();

            let chain_version = Pallet::<T>::on_chain_storage_version();

            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 0));

            if current > chain_version {
                weight.saturating_accrue(migrate_locks_to_freeze::<T>());

                weight.saturating_accrue(migrate_holds::<T>());

                current.put::<Pallet<T>>();

                info!("v1 applied successfully");
                T::DbWeight::get().reads_writes(0, 1)
            } else {
                warn!("Skipping v1, should be removed");
                T::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
            frame_support::ensure!(
                Pallet::<T>::on_chain_storage_version() == 1,
                "v1 not applied"
            );

            Ok(())
        }
    }
}
