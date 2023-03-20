use super::*;
use frame_support::{
    dispatch::GetStorageVersion,
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};
use log::{info, warn};

pub mod v1 {

    use super::*;

    pub fn clear_storages<T: Config>() {
        let _ = frame_support::migration::clear_storage_prefix(b"INV4", b"", b"", None, None);
    }

    pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
            frame_support::ensure!(
                Pallet::<T>::current_storage_version() == 0,
                "Required v0 before upgrading to v1"
            );

            Ok(Default::default())
        }

        fn on_runtime_upgrade() -> Weight {
            let current = Pallet::<T>::current_storage_version();

            if current == 1 {
                clear_storages::<T>();

                current.put::<Pallet<T>>();

                info!("v1 applied successfully");
                T::DbWeight::get().reads_writes(0, 1)
            } else {
                warn!("Skipping v1, should be removed");
                T::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
            frame_support::ensure!(
                Pallet::<T>::on_chain_storage_version() == 1,
                "v1 not applied"
            );

            Ok(())
        }
    }
}
