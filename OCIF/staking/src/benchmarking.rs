#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as OcifStaking;
use core::ops::Add;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::{Get, OnFinalize, OnInitialize};
use frame_system::{Pallet as System, RawOrigin};
use pallet_inv4::{
    origin::{INV4Origin, MultisigInternalOrigin},
    util::derive_core_account,
};
use sp_runtime::traits::{Bounded, One};
use sp_std::vec;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn derive_account<T: pallet_inv4::Config>(
    core_id: <T as pallet_inv4::Config>::CoreId,
) -> T::AccountId {
    derive_core_account::<T, <T as pallet_inv4::Config>::CoreId, T::AccountId>(core_id)
}

fn advance_to_era<T: Config>(n: Era) {
    while OcifStaking::<T>::current_era() < n {
        <OcifStaking<T> as OnFinalize<<T as frame_system::Config>::BlockNumber>>::on_finalize(
            System::<T>::block_number(),
        );
        System::<T>::set_block_number(System::<T>::block_number() + One::one());
        <OcifStaking<T> as OnInitialize<<T as frame_system::Config>::BlockNumber>>::on_initialize(
            System::<T>::block_number(),
        );
    }
}

fn mock_register<T: Config>() -> DispatchResultWithPostInfo
where
    Result<
        INV4Origin<T, <T as pallet_inv4::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <T as frame_system::Config>::RuntimeOrigin:
        From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,
{
    <T as Config>::Currency::make_free_balance_be(
        &derive_account::<T>(0u32.into()),
        T::RegisterDeposit::get() + T::RegisterDeposit::get(),
    );

    OcifStaking::<T>::register_core(
        INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())).into(),
        vec![],
        vec![],
        vec![],
    )
}

fn mock_stake<T: Config>() -> DispatchResultWithPostInfo
where
    Result<
        INV4Origin<T, <T as pallet_inv4::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <T as frame_system::Config>::RuntimeOrigin:
        From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,
{
    <T as Config>::Currency::make_free_balance_be(
        &whitelisted_caller(),
        BalanceOf::<T>::max_value(),
    );

    OcifStaking::<T>::stake(
        RawOrigin::Signed(whitelisted_caller()).into(),
        0u32.into(),
        T::StakeThresholdForActiveCore::get() + T::StakeThresholdForActiveCore::get(),
    )
}

fn mock_unstake<T: Config>() -> DispatchResultWithPostInfo
where
    Result<
        INV4Origin<T, <T as pallet_inv4::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <T as frame_system::Config>::RuntimeOrigin:
        From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,
{
    OcifStaking::<T>::unstake(
        RawOrigin::Signed(whitelisted_caller()).into(),
        0u32.into(),
        T::StakeThresholdForActiveCore::get() + T::StakeThresholdForActiveCore::get(),
    )
}

benchmarks! {
    where_clause {
    where
        Result<
                INV4Origin<
                        T,
                    <T as pallet_inv4::Config>::CoreId,
                    <T as frame_system::Config>::AccountId,
                    >,
            <T as frame_system::Config>::RuntimeOrigin,
            >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <T as frame_system::Config>::RuntimeOrigin: From<INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>>,

}

    register_core {
        let n in 0 .. T::MaxNameLength::get();
        let d in 0 .. T::MaxDescriptionLength::get();
        let i in 0 .. T::MaxImageUrlLength::get();

        let name = vec![u8::MAX; n as usize];
        let description = vec![u8::MAX; d as usize];
        let image = vec![u8::MAX; i as usize];

        <T as Config>::Currency::make_free_balance_be(&derive_account::<T>(0u32.into()), T::RegisterDeposit::get());
    }: _(INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())), name, description, image)
    verify {
        assert_last_event::<T>(Event::<T>::CoreRegistered {
            core: 0u32.into()
        }.into());
    }

    change_core_metadata {
        let n in 0 .. T::MaxNameLength::get();
        let d in 0 .. T::MaxDescriptionLength::get();
        let i in 0 .. T::MaxImageUrlLength::get();

        let name = vec![u8::MAX; n as usize];
        let description = vec![u8::MAX; d as usize];
        let image = vec![u8::MAX; i as usize];

        mock_register().unwrap();

    }: _(INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())), name.clone(), description.clone(), image.clone())
        verify {
            assert_last_event::<T>(Event::<T>::MetadataChanged {
                core: 0u32.into(),
                old_metadata: CoreMetadata {
                    name: vec![],
                    description: vec![],
                    image: vec![]
                },
                new_metadata: CoreMetadata {
                    name,
                    description,
                    image
                }
            }.into());
        }

    unregister_core {
        mock_register().unwrap();

    }: _(INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())))
    verify {
        assert_last_event::<T>(Event::<T>::CoreUnregistered {
            core: 0u32.into()
        }.into());
    }

    stake {
        mock_register().unwrap();

        let staker = whitelisted_caller();
        let amount = T::StakeThresholdForActiveCore::get() + T::StakeThresholdForActiveCore::get();

        <T as Config>::Currency::make_free_balance_be(&staker, BalanceOf::<T>::max_value());
    }: _(RawOrigin::Signed(staker.clone()), 0u32.into(), amount)
    verify {
        assert_last_event::<T>(Event::<T>::Staked {
            staker,
            core: 0u32.into(),
            amount
        }.into());
    }

    unstake {
        mock_register().unwrap();
        mock_stake().unwrap();

        let staker: T::AccountId = whitelisted_caller();
        let amount = T::StakeThresholdForActiveCore::get() + T::StakeThresholdForActiveCore::get();

    }: _(RawOrigin::Signed(staker.clone()), 0u32.into(), amount)
        verify {
            assert_last_event::<T>(Event::<T>::Unstaked {
                staker,
                core: 0u32.into(),
                amount
            }.into());
        }

    withdraw_unstaked {
        mock_register().unwrap();
        mock_stake().unwrap();
        mock_unstake().unwrap();
        advance_to_era::<T>(T::UnbondingPeriod::get().add(1));

        let staker: T::AccountId = whitelisted_caller();
        let amount = T::StakeThresholdForActiveCore::get() + T::StakeThresholdForActiveCore::get();

    }: _(RawOrigin::Signed(staker.clone()))
        verify {
            assert_last_event::<T>(Event::<T>::Withdrawn {
                staker,
                amount
            }.into());
        }

    staker_claim_rewards {
        mock_register().unwrap();
        mock_stake().unwrap();
        advance_to_era::<T>(One::one());

        let staker: T::AccountId = whitelisted_caller();
        let amount = T::StakeThresholdForActiveCore::get() + T::StakeThresholdForActiveCore::get();

        let core_stake_info = OcifStaking::<T>::core_stake_info::<<T as Config>::CoreId, Era>(0u32.into(), 0u32).unwrap();
        let era_info = OcifStaking::<T>::general_era_info::<Era>(0u32).unwrap();

        let (_, reward) = OcifStaking::<T>::core_stakers_split(&core_stake_info, &era_info);

    }: _(RawOrigin::Signed(staker.clone()), 0u32.into())
        verify {
            assert_last_event::<T>(Event::<T>::StakerClaimed {
                staker,
                core: 0u32.into(),
                era: 0u32.into(),
                amount: reward
            }.into());
        }

    core_claim_rewards {
        mock_register().unwrap();
        mock_stake().unwrap();
        advance_to_era::<T>(One::one());

        let staker: T::AccountId = whitelisted_caller();
        let amount = T::StakeThresholdForActiveCore::get() + T::StakeThresholdForActiveCore::get();

        let core_stake_info = OcifStaking::<T>::core_stake_info::<<T as Config>::CoreId, Era>(0u32.into(), 0u32).unwrap();
        let era_info = OcifStaking::<T>::general_era_info::<Era>(0u32).unwrap();

        let (reward, _) = OcifStaking::<T>::core_stakers_split(&core_stake_info, &era_info);

    }: _(INV4Origin::Multisig(MultisigInternalOrigin::new(0u32.into())), 0u32.into(), 0u32.into())
        verify {
            assert_last_event::<T>(Event::<T>::CoreClaimed {
                core: 0u32.into(),
                destination_account: derive_account::<T>(0u32.into()),
                era: 0u32.into(),
                amount: reward
            }.into());
        }

    halt_unhalt_pallet {}: _(RawOrigin::Root, true)
        verify {
            assert_last_event::<T>(Event::<T>::HaltChanged {
                is_halted: true
            }.into());
        }
}
