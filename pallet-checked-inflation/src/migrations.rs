use super::*;
use frame_support::traits::OnRuntimeUpgrade;

pub mod first_time {
    use super::*;
    use frame_support::{traits::Currency, weights::Weight};

    pub struct InitializeStorages<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config> OnRuntimeUpgrade for InitializeStorages<T> {
        fn on_runtime_upgrade() -> Weight {
            // As a safety measure, we check if YearStartIssuance is 0.
            if YearStartIssuance::<T>::get() == Zero::zero() {
                let current_issuance =
                    <<T as Config>::Currency as Currency<T::AccountId>>::total_issuance();

                YearStartIssuance::<T>::put(current_issuance);

                T::DbWeight::get().reads_writes(2, 1)
            } else {
                // This migration should be removed from the Runtime if it's not needed anymore.
                T::DbWeight::get().reads(1)
            }
        }
    }
}
