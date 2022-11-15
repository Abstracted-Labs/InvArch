use super::*;
use frame_support::traits::OnRuntimeUpgrade;

pub mod first_time {
    use super::*;
    use frame_support::{traits::Currency, weights::Weight};

    pub struct InitializeStorages<T, FirstTime>(sp_std::marker::PhantomData<(T, FirstTime)>);
    impl<T: Config, FirstTime: Get<bool>> OnRuntimeUpgrade for InitializeStorages<T, FirstTime> {
        fn on_runtime_upgrade() -> Weight {
            // Has to be hard coded as being the first time the pallet is added to the runtime
            // And as a safety measure in case it is still set to true in subsequent upgrades, we checke if YearStartIssuance is 0.
            if FirstTime::get() && YearStartIssuance::<T>::get() == Zero::zero() {
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
