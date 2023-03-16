#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{
    origin::{INV4Origin, MultisigInternalOrigin},
    util::derive_core_account,
    voting::{Tally, Vote},
    BalanceOf,
};
use codec::Encode;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{
    dispatch::PostDispatchInfo,
    traits::{Currency, Get, WrapperKeepOpaque},
};
use frame_system::RawOrigin as SystemOrigin;
use sp_runtime::{
    traits::{Bounded, Hash, Zero},
    DispatchError, DispatchErrorWithPostInfo, Perbill,
};
use sp_std::{iter::Sum, ops::Div, prelude::*, vec};

use crate::Pallet as INV4;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn perbill_one() -> Perbill {
    Perbill::one()
}

fn derive_account<T: Config>(core_id: T::CoreId) -> T::AccountId {
    derive_core_account::<T, T::CoreId, T::AccountId>(core_id)
}

fn mock_core<T: Config>() -> Result<(), DispatchError>
where
    Result<
        INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance:
        Sum,
{
    T::Currency::make_free_balance_be(
        &whitelisted_caller(),
        T::CoreCreationFee::get() + T::CoreCreationFee::get(),
    );

    INV4::<T>::create_core(
        SystemOrigin::Signed(whitelisted_caller()).into(),
        vec![],
        perbill_one(),
        perbill_one(),
    )
}

fn mock_mint<T: Config>() -> Result<(), DispatchError>
where
    Result<
        INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance:
        Sum,
    <T as frame_system::Config>::RuntimeOrigin:
        From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,
{
    INV4::<T>::token_mint(
        INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())).into(),
        BalanceOf::<T>::max_value().div(4u32.into()),
        account("target", 0, SEED),
    )
}

fn mock_mint_2<T: Config>() -> Result<(), DispatchError>
where
    Result<
        INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance:
        Sum,
    <T as frame_system::Config>::RuntimeOrigin:
        From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,
{
    INV4::<T>::token_mint(
        INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())).into(),
        BalanceOf::<T>::max_value().div(4u32.into()),
        account("target1", 1, SEED + 1),
    )
}

fn mock_call<T: Config>() -> Result<PostDispatchInfo, DispatchErrorWithPostInfo<PostDispatchInfo>>
where
    Result<
        INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance:
        Sum,
    <T as frame_system::Config>::RuntimeOrigin:
        From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,
{
    INV4::<T>::operate_multisig(
        SystemOrigin::Signed(whitelisted_caller()).into(),
        0u32.into(),
        None,
        Box::new(frame_system::Call::<T>::remark { remark: vec![0] }.into()),
    )
}

fn mock_vote<T: Config>() -> Result<PostDispatchInfo, DispatchErrorWithPostInfo<PostDispatchInfo>>
where
    Result<
        INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance:
        Sum,
    <T as frame_system::Config>::RuntimeOrigin:
        From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,
{
    let call: <T as Config>::RuntimeCall =
        frame_system::Call::<T>::remark { remark: vec![0] }.into();
    let call_hash = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&call.clone());

    INV4::<T>::vote_multisig(
        SystemOrigin::Signed(account("target", 0, SEED)).into(),
        0u32.into(),
        call_hash,
        true,
    )
}

benchmarks! {

    where_clause {
      where
        Result<
                INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
            <T as frame_system::Config>::RuntimeOrigin,
            >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: Sum,
    <T as frame_system::Config>::RuntimeOrigin: From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,
}

    create_core {
        let m in 0 .. T::MaxMetadata::get();

        let metadata = vec![u8::MAX; m as usize];
        let caller = whitelisted_caller();
        let minimum_support = perbill_one();
        let required_approval = perbill_one();

        T::Currency::make_free_balance_be(&caller, T::CoreCreationFee::get() + T::CoreCreationFee::get());
    }: _(SystemOrigin::Signed(caller.clone()), metadata.clone(), minimum_support, required_approval)
        verify {
            assert_last_event::<T>(Event::CoreCreated {
                core_account: derive_account::<T>(0u32.into()),
                core_id: 0u32.into(),
                metadata,
                minimum_support,
                required_approval
            }.into());
        }

    set_parameters {
        let m in 0 .. T::MaxMetadata::get();

        mock_core().unwrap();

        let metadata = Some(vec![u8::MAX; m as usize]);
        let minimum_support = Some(perbill_one());
        let required_approval = Some(perbill_one());

    }: _(INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())), metadata.clone(), minimum_support, required_approval)
        verify {
            assert_last_event::<T>(Event::ParametersSet {
                core_id: 0u32.into(),
                metadata,
                minimum_support,
                required_approval
            }.into());
        }

    token_mint {
        mock_core().unwrap();

        let amount = BalanceOf::<T>::max_value().div(2u32.into());
        let target: T::AccountId = account("target", 0, SEED);

    }: _(INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())), amount, target.clone())
        verify {
            assert_last_event::<T>(Event::Minted {
                core_id: 0u32.into(),
                target,
                amount
            }.into());
        }

    token_burn {
        mock_core().unwrap();
        mock_mint().unwrap();

        let amount = BalanceOf::<T>::max_value().div(4u32.into());
        let target: T::AccountId = account("target", 0, SEED);

    }: _(INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())), amount, target.clone())
        verify {
            assert_last_event::<T>(Event::Burned {
                core_id: 0u32.into(),
                target,
                amount
            }.into());
        }

    operate_multisig {
        let m in 0 .. T::MaxMetadata::get();
        let z in 0 .. 10_000;

        mock_core().unwrap();
        mock_mint().unwrap();

        let metadata = vec![u8::MAX; m as usize];
        let caller: T::AccountId = whitelisted_caller();
        let core_id: T::CoreId = 0u32.into();
        let call: <T as Config>::RuntimeCall = frame_system::Call::<T>::remark {
            remark: vec![0; z as usize]
        }.into();
        let call_hash = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&call.clone());

    }: _(SystemOrigin::Signed(caller.clone()), core_id, Some(metadata), Box::new(call.clone()))
        verify {
            assert_last_event::<T>(Event::MultisigVoteStarted {
                core_id,
                executor_account: derive_account::<T>(core_id),
                voter: caller,
                votes_added: Vote::Aye(T::CoreSeedBalance::get()),
                call_hash,
                call: WrapperKeepOpaque::from_encoded(call.encode()),
            }.into());
        }

    vote_multisig {
        mock_core().unwrap();
        mock_mint().unwrap();
        mock_mint_2().unwrap();
        mock_call().unwrap();

        let caller: T::AccountId = account("target", 0, SEED);
        let core_id: T::CoreId = 0u32.into();
        let call: <T as Config>::RuntimeCall = frame_system::Call::<T>::remark {
            remark: vec![0]
        }.into();
        let call_hash = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&call.clone());

    }: _(SystemOrigin::Signed(caller.clone()), core_id, call_hash, true)
        verify {
            assert_last_event::<T>(Event::MultisigVoteAdded {
                core_id,
                executor_account: derive_account::<T>(core_id),
                voter: caller,
                votes_added:  Vote::Aye(BalanceOf::<T>::max_value().div(4u32.into())),
                current_votes: Tally::<T>::from_parts(
                    (BalanceOf::<T>::max_value().div(4u32.into()) + T::CoreSeedBalance::get()).into(), Zero::zero()
                ),
                call_hash,
                call: WrapperKeepOpaque::from_encoded(call.encode()),
            }.into());
        }

    withdraw_vote_multisig {
        mock_core().unwrap();
        mock_mint().unwrap();
        mock_mint_2().unwrap();
        mock_call().unwrap();
        mock_vote().unwrap();

        let caller: T::AccountId = account("target", 0, SEED);
        let core_id: T::CoreId = 0u32.into();
        let call: <T as Config>::RuntimeCall = frame_system::Call::<T>::remark {
            remark: vec![0]
        }.into();
        let call_hash = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&call.clone());

    }: _(SystemOrigin::Signed(caller.clone()), core_id, call_hash)
        verify {
            assert_last_event::<T>(Event::MultisigVoteWithdrawn {
                core_id,
                executor_account: derive_account::<T>(core_id),
                voter: caller,
                votes_removed: Vote::Aye(BalanceOf::<T>::max_value().div(4u32.into())),
                call_hash,
                call: WrapperKeepOpaque::from_encoded(call.encode()),
            }.into());
        }

}
