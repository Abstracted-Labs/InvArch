//! Benchmarks for IPL Pallet
#![cfg(feature = "runtime-benchmarks")]

pub use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use primitives::OneOrPercent;

runtime_benchmarks! {
  set_permission {
      let s in 0 .. 100;
      let caller: T::AccountId = whitelisted_caller();
      let sub_asset: T::IplId = Default::default();
  }: _(RawOrigin::Signed(caller), T::IplId::from(s), sub_asset, Default::default(), true)

  set_asset_weight {
      let s in 0 .. 100;
      let caller: T::AccountId = whitelisted_caller();
      let sub_asset: T::IplId = Default::default();

      Pallet::<T>::set_permission(RawOrigin::Signed(caller.clone()).into(),T::IplId::from(s), sub_asset, Default::default(), true)?;
  }: _(RawOrigin::Signed(caller), T::IplId::from(s), sub_asset, OneOrPercent::One)
}

impl_benchmark_test_suite!(Ipl, crate::mock::new_test_ext(), crate::mock::Test,);
