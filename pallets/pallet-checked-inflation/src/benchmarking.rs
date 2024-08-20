#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin as SystemOrigin;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    set_first_year_supply {
    }: _(SystemOrigin::Root)

    halt_unhalt_pallet {
    }: _(SystemOrigin::Root, true)
    verify {
        assert_last_event::<T>(Event::<T>::HaltChanged {
            is_halted: true
        }.into());
    }
}
