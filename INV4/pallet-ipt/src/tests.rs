//! Unit tests for the IPT pallet.

use codec::Encode;
use frame_support::{assert_noop, assert_ok, dispatch::GetDispatchInfo, traits::WrapperKeepOpaque};
use primitives::{utils::multi_account_id, IptInfo, SubIptInfo};
use sp_core::blake2_256;

use crate::{
    mock::{
        Balances, Call, ExistentialDeposit, ExtBuilder, InvArchLicenses::*, Ipt, Origin, Runtime,
        ALICE, BOB, VADER,
    },
    Balance, Config, Error, Ipt as IptStorage, Multisig, MultisigOperationOf, SubAssets,
};

use sp_std::convert::TryInto;

use primitives::OneOrPercent::*;
use sp_runtime::{DispatchError, Percent};

type IptId = <Runtime as Config>::IptId;

macro_rules! percent {
    ($x:expr) => {
        ZeroPoint(Percent::from_percent($x))
    };
}

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            ALICE,
            0,
            vec![(ALICE, ExistentialDeposit::get())],
            vec![SubIptInfo {
                id: 0,
                metadata: b"test".to_vec().try_into().unwrap(),
            }]
            .try_into()
            .unwrap(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
            })
        );

        assert_ok!(Ipt::mint(Origin::signed(ALICE), (0, None), 1000, ALICE));
        assert_ok!(Ipt::mint(Origin::signed(ALICE), (0, Some(0)), 1000, ALICE));

        let id: (
            <Runtime as Config>::IptId,
            Option<<Runtime as Config>::IptId>,
        ) = (0, None);
        assert_eq!(
            Balance::<Runtime>::get(id, ALICE),
            Some(ExistentialDeposit::get() + 1000)
        );

        assert_eq!(
            Balance::<Runtime>::get((0, Some(0)), ALICE),
            Some(1000u32.into())
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: ExistentialDeposit::get() + 1000,
            })
        );
    });
}

#[test]
fn mint_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            ALICE,
            0,
            vec![(ALICE, ExistentialDeposit::get())],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
            })
        );

        // Case 0: Unknown origin
        assert_noop!(
            Ipt::mint(Origin::none(), (0, None), 1000, ALICE),
            DispatchError::BadOrigin
        );

        assert_ne!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: ExistentialDeposit::get() + 1000,
            })
        );

        // Case 1: Ipt does not exist
        assert_noop!(
            Ipt::mint(Origin::signed(ALICE), (32, None), 1000, ALICE),
            Error::<Runtime>::IptDoesntExist
        );

        // Case 1.5: SubAsset does not exist
        assert_noop!(
            Ipt::mint(Origin::signed(ALICE), (0, Some(0)), 1000, ALICE),
            Error::<Runtime>::SubAssetNotFound
        );

        assert_eq!(IptStorage::<Runtime>::get(32), None);

        // Case 2: Caller has no permission
        assert_noop!(
            Ipt::mint(Origin::signed(BOB), (0, None), 1000, ALICE),
            Error::<Runtime>::NoPermission,
        );

        assert_ne!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: ExistentialDeposit::get() + 1000,
            })
        );
    });
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            ALICE,
            0,
            vec![(ALICE, ExistentialDeposit::get())],
            vec![SubIptInfo {
                id: 0,
                metadata: b"test".to_vec().try_into().unwrap(),
            }]
            .try_into()
            .unwrap(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
            })
        );

        assert_ok!(Ipt::internal_mint((0, Some(0)), ALICE, 1000));

        assert_eq!(
            Balance::<Runtime>::get((0, Some(0)), ALICE),
            Some(1000u32.into())
        );

        assert_ok!(Ipt::burn(Origin::signed(ALICE), (0, None), 500, ALICE));

        let id: (
            <Runtime as Config>::IptId,
            Option<<Runtime as Config>::IptId>,
        ) = (0, None);
        assert_eq!(Balance::<Runtime>::get(id, ALICE), Some(0u32.into()));

        assert_ok!(Ipt::burn(Origin::signed(ALICE), (0, Some(0)), 500, ALICE));

        assert_eq!(
            Balance::<Runtime>::get((0, Some(0)), ALICE),
            Some(500u32.into())
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: 0,
            })
        );
    });
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            ALICE,
            0,
            vec![(ALICE, ExistentialDeposit::get())],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
            })
        );

        // Case 0: Unknown origin
        assert_noop!(
            Ipt::burn(Origin::none(), (0, None), 500, ALICE),
            DispatchError::BadOrigin
        );

        assert_ne!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: 0,
            })
        );

        // Case 1: Ipt does not exist
        assert_noop!(
            Ipt::burn(Origin::signed(ALICE), (32, None), 500, ALICE),
            Error::<Runtime>::IptDoesntExist
        );

        assert_eq!(IptStorage::<Runtime>::get(32), None);

        // Case 1: Sub asset does not exist
        assert_noop!(
            Ipt::burn(Origin::signed(ALICE), (0, Some(0)), 500, ALICE),
            Error::<Runtime>::SubAssetNotFound
        );

        // Case 2: Caller has no permission
        assert_noop!(
            Ipt::burn(Origin::signed(BOB), (0, None), 500, ALICE),
            Error::<Runtime>::NoPermission
        );

        assert_ne!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: 0,
            })
        );
    });
}

#[test]
fn operate_multisig_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        // > total_per_2
        Ipt::create(
            multi_account_id::<Runtime, IptId>(0, None),
            0,
            vec![
                (ALICE, ExistentialDeposit::get()),
                (BOB, ExistentialDeposit::get() * 2 + 1),
            ],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        assert_ok!(Ipt::operate_multisig(
            Origin::signed(BOB),
            false,
            (0, None),
            Box::new(Call::Ipt(crate::Call::mint {
                ipt_id: (0, None),
                amount: 1000,
                target: BOB,
            }))
        ));

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IptId>(0, None),
                supply: ExistentialDeposit::get() * 3 + 1001,
            })
        );

        // < total_per_2
        let call = Call::Ipt(crate::Call::mint {
            ipt_id: (0, None),
            amount: 1000,
            target: ALICE,
        });

        let call_hash = blake2_256(&call.encode());

        assert_ok!(Ipt::operate_multisig(
            Origin::signed(ALICE),
            false,
            (0, None),
            Box::new(call.clone())
        ));

        assert_eq!(
            Multisig::<Runtime>::get((0, call_hash)),
            Some(MultisigOperationOf::<Runtime> {
                signers: vec![(ALICE, None)].try_into().unwrap(),
                include_original_caller: false,
                original_caller: ALICE,
                actual_call: WrapperKeepOpaque::from_encoded(call.encode()),
                call_weight: call.get_dispatch_info().weight,
                call_metadata: call.encode().split_at(2).0.try_into().unwrap()
            })
        )
    });
}

#[test]
fn operate_multisig_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            multi_account_id::<Runtime, IptId>(0, None),
            0,
            vec![
                (ALICE, ExistentialDeposit::get()),
                (BOB, ExistentialDeposit::get() * 2 + 1),
            ],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        let call = Call::Ipt(crate::Call::mint {
            ipt_id: (0, None),
            amount: 1000,
            target: ALICE,
        });

        // Case 0: Unknown origin
        assert_noop!(
            Ipt::operate_multisig(Origin::none(), true, (0, None), Box::new(call.clone())),
            DispatchError::BadOrigin
        );

        // Case 1: Ipt doesn't exist
        assert_noop!(
            Ipt::operate_multisig(
                Origin::signed(ALICE),
                true,
                (32767, None),
                Box::new(call.clone())
            ),
            Error::<Runtime>::IptDoesntExist
        );

        // Case 2: Signer has no permission
        assert_noop!(
            Ipt::operate_multisig(
                Origin::signed(VADER),
                true,
                (0, None),
                Box::new(call.clone())
            ),
            Error::<Runtime>::NoPermission,
        );

        // Case 3: Multisig Operation Already Exists
        assert_ok!(Ipt::operate_multisig(
            Origin::signed(ALICE),
            true,
            (0, None),
            Box::new(call.clone())
        ),);

        assert_noop!(
            Ipt::operate_multisig(
                Origin::signed(ALICE),
                true,
                (0, None),
                Box::new(call.clone())
            ),
            Error::<Runtime>::MultisigOperationAlreadyExists
        );

        assert_eq!(
            Multisig::<Runtime>::get((0, blake2_256(&call.encode()))),
            Some(MultisigOperationOf::<Runtime> {
                signers: vec![(ALICE, None)].try_into().unwrap(),
                include_original_caller: true,
                original_caller: ALICE,
                actual_call: WrapperKeepOpaque::from_encoded(call.encode()),
                call_weight: call.get_dispatch_info().weight,
                call_metadata: call.encode().split_at(2).0.try_into().unwrap()
            })
        );
    });
}

// This test doesn't include a should_fail, since it's not meant to fail.
#[test]
fn create_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            ALICE,
            0,
            vec![(ALICE, 3_000_000)],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: ALICE,
                supply: 3_000_000,
            })
        );

        let id: (
            <Runtime as Config>::IptId,
            Option<<Runtime as Config>::IptId>,
        ) = (0, None);
        assert_eq!(Balance::<Runtime>::get(id, ALICE), Some(3_000_000));

        Ipt::create(
            BOB,
            32767,
            vec![(ALICE, 300), (BOB, 400_000)],
            vec![SubIptInfo {
                id: 0,
                metadata: b"test".to_vec().try_into().unwrap(),
            }]
            .try_into()
            .unwrap(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        assert_eq!(
            IptStorage::<Runtime>::get(32767),
            Some(IptInfo {
                owner: BOB,
                supply: 400_300,
            })
        );

        assert_eq!(
            SubAssets::<Runtime>::get(32767, 0),
            Some(SubIptInfo {
                id: 0,
                metadata: b"test".to_vec().try_into().unwrap(),
            })
        );

        let id: (
            <Runtime as Config>::IptId,
            Option<<Runtime as Config>::IptId>,
        ) = (32767, None);
        assert_eq!(Balance::<Runtime>::get(id, ALICE), Some(300));
        assert_eq!(Balance::<Runtime>::get(id, BOB), Some(400_000));

        Ipt::create(
            ALICE,
            IptId::max_value(),
            vec![(ALICE, 1), (BOB, 2)],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            true,
        );

        assert_eq!(
            IptStorage::<Runtime>::get(IptId::max_value()),
            Some(IptInfo {
                owner: ALICE,
                supply: 3,
            })
        );

        let id: (
            <Runtime as Config>::IptId,
            Option<<Runtime as Config>::IptId>,
        ) = (IptId::max_value(), None);
        assert_eq!(Balance::<Runtime>::get(id, ALICE), Some(1));
        assert_eq!(Balance::<Runtime>::get(id, BOB), Some(2));
    });
}

#[test]
fn withdraw_vote_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            ALICE,
            0,
            vec![
                (ALICE, ExistentialDeposit::get()),
                (BOB, ExistentialDeposit::get() * 2 + 1),
                (VADER, ExistentialDeposit::get()),
            ],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        let call = Call::Ipt(crate::Call::mint {
            ipt_id: (0, None),
            amount: 1000,
            target: BOB,
        });

        let call_hash = blake2_256(&call.encode());

        assert_ok!(Balances::set_balance(
            Origin::root(),
            multi_account_id::<Runtime, IptId>(0, None),
            ExistentialDeposit::get(),
            0
        ));

        assert_ok!(Ipt::operate_multisig(
            Origin::signed(ALICE),
            false,
            (0, None),
            Box::new(call.clone())
        ));
        assert_ok!(Ipt::vote_multisig(
            Origin::signed(VADER),
            (0, None),
            call_hash
        ));

        assert_eq!(
            Multisig::<Runtime>::get((0, call_hash)),
            Some(MultisigOperationOf::<Runtime> {
                signers: vec![(ALICE, None), (VADER, None)].try_into().unwrap(),
                include_original_caller: false,
                original_caller: ALICE,
                actual_call: WrapperKeepOpaque::from_encoded(call.encode()),
                call_weight: call.get_dispatch_info().weight,
                call_metadata: call.encode().split_at(2).0.try_into().unwrap(),
            })
        );

        assert_ok!(Ipt::withdraw_vote_multisig(
            Origin::signed(VADER),
            (0, None),
            call_hash
        ));

        assert_eq!(
            Multisig::<Runtime>::get((0, call_hash)),
            Some(MultisigOperationOf::<Runtime> {
                signers: vec![(ALICE, None)].try_into().unwrap(),
                include_original_caller: false,
                original_caller: ALICE,
                actual_call: WrapperKeepOpaque::from_encoded(call.encode()),
                call_weight: call.get_dispatch_info().weight,
                call_metadata: call.encode().split_at(2).0.try_into().unwrap(),
            })
        );
    });
}

#[test]
fn withdraw_vote_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            multi_account_id::<Runtime, IptId>(0, None),
            0,
            vec![
                (ALICE, ExistentialDeposit::get()),
                (BOB, ExistentialDeposit::get() * 2 + 1),
                (VADER, ExistentialDeposit::get()),
            ],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        let call = Call::Ipt(crate::Call::mint {
            ipt_id: (0, None),
            amount: 1000,
            target: BOB,
        });

        let call_hash = blake2_256(&call.encode());

        assert_ok!(Balances::set_balance(
            Origin::root(),
            multi_account_id::<Runtime, IptId>(0, None),
            ExistentialDeposit::get(),
            0
        ));

        assert_ok!(Ipt::operate_multisig(
            Origin::signed(ALICE),
            false,
            (0, None),
            Box::new(call.clone())
        ));

        assert_ok!(Ipt::vote_multisig(
            Origin::signed(VADER),
            (0, None),
            call_hash
        ));

        assert_eq!(
            Multisig::<Runtime>::get((0, call_hash)),
            Some(MultisigOperationOf::<Runtime> {
                signers: vec![(ALICE, None), (VADER, None)].try_into().unwrap(),
                include_original_caller: false,
                original_caller: ALICE,
                actual_call: WrapperKeepOpaque::from_encoded(call.encode()),
                call_weight: call.get_dispatch_info().weight,
                call_metadata: call.encode().split_at(2).0.try_into().unwrap(),
            })
        );

        // Case 0: Unknown origin
        assert_noop!(
            Ipt::withdraw_vote_multisig(Origin::none(), (0, None), call_hash),
            DispatchError::BadOrigin
        );

        // Case 1: Ipt does not exist
        assert_noop!(
            Ipt::withdraw_vote_multisig(Origin::signed(VADER), (32767, None), call_hash),
            Error::<Runtime>::IptDoesntExist,
        );

        // Case 2: Multisig operation uninitialized
        let uninitialized_call_hash = blake2_256(
            &Call::Ipt(crate::Call::burn {
                ipt_id: (0, None),
                amount: 1000,
                target: BOB,
            })
            .encode(),
        );

        assert_noop!(
            Ipt::withdraw_vote_multisig(Origin::signed(VADER), (0, None), uninitialized_call_hash),
            Error::<Runtime>::MultisigOperationUninitialized
        );

        // Case 3: Not a voter
        assert_noop!(
            Ipt::withdraw_vote_multisig(Origin::signed(BOB), (0, None), call_hash),
            Error::<Runtime>::NotAVoter,
        );
    });
}

#[test]
fn vote_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            multi_account_id::<Runtime, IptId>(0, None),
            0,
            vec![
                (ALICE, ExistentialDeposit::get()),
                (BOB, ExistentialDeposit::get() * 2 + 1),
                (VADER, ExistentialDeposit::get()),
            ],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        let call = Call::Ipt(crate::Call::mint {
            ipt_id: (0, None),
            amount: 1000,
            target: BOB,
        });

        let call_hash = blake2_256(&call.encode());

        assert_ok!(Balances::set_balance(
            Origin::root(),
            multi_account_id::<Runtime, IptId>(0, None),
            ExistentialDeposit::get(),
            0
        ));

        assert_ok!(Ipt::operate_multisig(
            Origin::signed(ALICE),
            false,
            (0, None),
            Box::new(call.clone())
        ));

        // Shouldn't execute yet
        assert_ok!(Ipt::vote_multisig(
            Origin::signed(VADER),
            (0, None),
            call_hash
        ));

        assert_eq!(
            Multisig::<Runtime>::get((0, call_hash)),
            Some(MultisigOperationOf::<Runtime> {
                signers: vec![(ALICE, None), (VADER, None)].try_into().unwrap(),
                include_original_caller: false,
                original_caller: ALICE,
                call_weight: call.get_dispatch_info().weight,
                actual_call: WrapperKeepOpaque::from_encoded(call.encode()),
                call_metadata: call.encode().split_at(2).0.try_into().unwrap(),
            })
        );

        // Should execute
        assert_ok!(Ipt::vote_multisig(
            Origin::signed(BOB),
            (0, None),
            call_hash
        ));

        assert_eq!(Multisig::<Runtime>::get((0, call_hash)), None);

        let id: (
            <Runtime as Config>::IptId,
            Option<<Runtime as Config>::IptId>,
        ) = (0, None);
        assert_eq!(
            (
                Balance::<Runtime>::get(id, BOB),
                IptStorage::<Runtime>::get(0)
            ),
            (
                Some(ExistentialDeposit::get() * 2 + 1001),
                Some(IptInfo {
                    owner: multi_account_id::<Runtime, IptId>(0, None),
                    supply: ExistentialDeposit::get() * 4 + 1001,
                })
            )
        );

        // Special case: ipts are minted/burned while a multisig is in storage
        assert_ok!(Ipt::operate_multisig(
            Origin::signed(ALICE),
            false,
            (0, None),
            Box::new(call.clone())
        ));

        assert_ok!(Ipt::vote_multisig(
            Origin::signed(VADER),
            (0, None),
            call_hash
        ));

        // This multisig call now has a bit less than 50% of ipt votes and
        // may get stuck if tokens are burned.
        assert_ok!(Ipt::burn(
            Origin::signed(multi_account_id::<Runtime, IptId>(0, None)),
            (0, None),
            ExistentialDeposit::get() * 2 + 1001, /*Burning BOB's tokens*/
            BOB
        ));

        // Call won't be rechecked until ALICE or VADER tries voting again,
        // this should work even if they are already voters.
        //  assert_ok!(Ipt::vote_multisig(Origin::signed(ALICE), 0, call_hash)); // fails: NotEnoughAmount

        // assert_eq!(Multisig::<Runtime>::get((0, call_hash)), None);
        // assert_eq!(Balance::<Runtime>::get(0, BOB), Some(1000));
    });
}

#[test]
fn vote_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(
            multi_account_id::<Runtime, IptId>(0, None),
            0,
            vec![
                (ALICE, ExistentialDeposit::get()),
                (BOB, ExistentialDeposit::get() * 2 + 1),
                (VADER, ExistentialDeposit::get()),
            ],
            Default::default(),
            GPLv3,
            percent!(50),
            One,
            false,
        );

        let call = Call::Ipt(crate::Call::mint {
            ipt_id: (0, None),
            amount: 1000,
            target: BOB,
        });

        let call_hash = blake2_256(&call.encode());

        assert_ok!(Balances::set_balance(
            Origin::root(),
            multi_account_id::<Runtime, IptId>(0, None),
            ExistentialDeposit::get(),
            0
        ));

        assert_ok!(Ipt::operate_multisig(
            Origin::signed(ALICE),
            false,
            (0, None),
            Box::new(call.clone())
        ));

        // Case 0: Unknown origin
        assert_noop!(
            Ipt::vote_multisig(Origin::none(), (0, None), call_hash),
            DispatchError::BadOrigin
        );

        // Case 1: Ipt doesn't exist
        assert_noop!(
            Ipt::vote_multisig(Origin::signed(BOB), (32767, None), call_hash),
            Error::<Runtime>::IptDoesntExist,
        );

        // Case 2: Multisig operation uninitialized
        let uninitialized_call_hash = blake2_256(
            &Call::Ipt(crate::Call::burn {
                ipt_id: (0, None),
                amount: 1000,
                target: BOB,
            })
            .encode(),
        );
        assert_noop!(
            Ipt::vote_multisig(Origin::signed(BOB), (0, None), uninitialized_call_hash),
            Error::<Runtime>::MultisigOperationUninitialized
        );

        // Case 3: No permission
        assert_noop!(
            Ipt::vote_multisig(Origin::signed(32767), (0, None), call_hash),
            Error::<Runtime>::NoPermission,
        );
    });
}
