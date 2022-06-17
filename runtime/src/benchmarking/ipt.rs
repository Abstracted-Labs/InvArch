//! Benchmarks for IPT Pallet
#![cfg(feature = "runtime-benchmarks")]

pub use super::*;
use frame_benchmarking::{
    account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller, Box,
};
use frame_system::RawOrigin;
use primitives::SubIptInfo;
use sp_io::hashing::blake2_256;

const SEED: u32 = 0;

runtime_benchmarks! {
    mint {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let amount: <T as pallet::Config>::Balance = 300u32.into();
        let target: T::AccountId = account("target", 0, SEED);

        Pallet::<T>::internal_mint((T::IptId::from(s), None), target.clone(), amount.clone())?;
    }: _(RawOrigin::Signed(caller), (T::IptId::from(s), None), amount, target)

    burn {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let amount: <T as pallet::Config>::Balance = 300u32.into();
        let target: T::AccountId = account("target", 0, SEED);

        Pallet::<T>::internal_mint((T::IptId::from(s), None), target.clone(), amount.clone())?;

        Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), (T::IptId::from(s), None), amount, target.clone())?;
    }: _(RawOrigin::Signed(caller), (T::IptId::from(s), None), amount, target)

    operate_multisig {
        let s in 0 .. 100_000;
        let caller: T::AccountId = whitelisted_caller();
        let amount: <T as pallet::Config>::Balance = 300u32.into();
        let target: T::AccountId = account("target", 0, SEED);
        let call: <T as pallet::Config>::Call = frame_system::Call::<T>::remark {
            remark: vec![0; s as usize],
        }.into();

        Pallet::<T>::internal_mint((T::IptId::from(s), None), target.clone(), amount.clone())?;

        Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), (T::IptId::from(s), None), amount, target.clone())?;
    }: _(RawOrigin::Signed(caller), false, (T::IptId::from(s), None), Box::new(call))

    vote_multisig {
        let s in 0 .. 100_000;
        let caller: T::AccountId = whitelisted_caller();
        let amount: <T as pallet::Config>::Balance = 300u32.into();
        let target: T::AccountId = account("target", 0, SEED);
        let call: <T as pallet::Config>::Call = frame_system::Call::<T>::remark {
            remark: vec![0; s as usize],
        }.into();

        Pallet::<T>::internal_mint((T::IptId::from(s), None), target.clone(), amount.clone())?;

        Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), (T::IptId::from(s), None), amount, target.clone())?;
    }: _(RawOrigin::Signed(caller), (T::IptId::from(s), None), blake2_256(&call.encode()))

    withdraw_vote_multisig {
        let s in 0 .. 100_000;
        let caller: T::AccountId = whitelisted_caller();
        let amount: <T as pallet::Config>::Balance = 300u32.into();
        let target: T::AccountId = account("target", 0, SEED);
        let call: <T as pallet::Config>::Call = frame_system::Call::<T>::remark {
            remark: vec![0; s as usize],
        }.into();

        Pallet::<T>::internal_mint((T::IptId::from(s), None), target.clone(), amount.clone())?;

        Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), (T::IptId::from(s), None), amount, target.clone())?;

        Pallet::<T>::vote_multisig(RawOrigin::Signed(caller.clone()).into(), (T::IptId::from(s), None), blake2_256(&call.encode()))?;
    }: _(RawOrigin::Signed(caller), (T::IptId::from(s), None), blake2_256(&call.encode()))

    create_sub_asset {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let sub_assets: SubAssetsWithEndowment<T> = vec![(
            SubIptInfo {id: T::IptId::from(s), metadata: Default::default()}, (account("target", 0, SEED), 500u32.into())
        )];
    }: _(RawOrigin::Signed(caller), T::IptId::from(s), sub_assets)
}

impl_benchmark_test_suite!(Ipt, crate::mock::new_test_ext(), crate::mock::Test,);
