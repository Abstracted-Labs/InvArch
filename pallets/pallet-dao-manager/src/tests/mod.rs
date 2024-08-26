#[allow(unused_imports)]
mod mock;

extern crate alloc;

use crate::{
    multisig::{BoundedCallBytes, MultisigOperation, MAX_SIZE},
    origin::MultisigInternalOrigin,
    voting::{Tally, Vote},
    *,
};
use alloc::collections::BTreeMap;
use codec::Encode;
use frame_support::{assert_err, assert_ok, error::BadOrigin, BoundedBTreeMap};
use frame_system::RawOrigin;
use mock::*;
use primitives::CoreInfo;
use sp_runtime::{
    traits::{Hash, Zero},
    ArithmeticError, Perbill, TokenError,
};
use sp_std::{
    convert::{TryFrom, TryInto},
    vec,
};

#[test]
fn create_core_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Balances::free_balance(ALICE), INITIAL_BALANCE);

        assert_eq!(DaoManager::next_core_id(), 0u32);

        assert_eq!(DaoManager::core_storage(0u32), None);

        assert_ok!(DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::Native
        ));

        assert_eq!(DaoManager::next_core_id(), 1u32);

        assert_eq!(
            DaoManager::core_storage(0u32),
            Some(CoreInfo {
                account: DaoManager::derive_core_account(0u32),
                metadata: vec![].try_into().unwrap(),
                minimum_support: Perbill::from_percent(1),
                required_approval: Perbill::from_percent(1),
                frozen_tokens: true,
            })
        );

        assert_eq!(
            Balances::free_balance(ALICE),
            INITIAL_BALANCE - CoreCreationFee::get()
        );

        // Another attempt

        assert_eq!(DaoManager::next_core_id(), 1u32);

        assert_eq!(DaoManager::core_storage(1u32), None);

        assert_ok!(DaoManager::create_core(
            RawOrigin::Signed(BOB).into(),
            vec![1, 2, 3].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Relay
        ));

        assert_eq!(DaoManager::next_core_id(), 2u32);

        assert_eq!(
            DaoManager::core_storage(1u32),
            Some(CoreInfo {
                account: DaoManager::derive_core_account(1u32),
                metadata: vec![1, 2, 3].try_into().unwrap(),
                minimum_support: Perbill::from_percent(100),
                required_approval: Perbill::from_percent(100),
                frozen_tokens: true,
            })
        );

        assert_eq!(
            Tokens::accounts(BOB, RELAY_ASSET_ID).free,
            INITIAL_BALANCE - RelayCoreCreationFee::get()
        );
    });
}

#[test]
fn create_core_fails() {
    ExtBuilder::default().build().execute_with(|| {
        // Not enough balance for creation fee.

        assert_eq!(Balances::free_balance(DAVE), 0u128);

        assert_eq!(DaoManager::next_core_id(), 0u32);

        assert_eq!(DaoManager::core_storage(0u32), None);

        assert_err!(
            DaoManager::create_core(
                RawOrigin::Signed(DAVE).into(),
                vec![].try_into().unwrap(),
                Perbill::from_percent(1),
                Perbill::from_percent(1),
                FeeAsset::Native
            ),
            pallet_balances::Error::<Test>::InsufficientBalance
        );

        assert_eq!(DaoManager::next_core_id(), 0u32);
        assert_eq!(DaoManager::core_storage(0u32), None);

        // With Relay token.

        assert_eq!(Tokens::accounts(DAVE, RELAY_ASSET_ID).free, 0u128);

        assert_err!(
            DaoManager::create_core(
                RawOrigin::Signed(DAVE).into(),
                vec![].try_into().unwrap(),
                Perbill::from_percent(1),
                Perbill::from_percent(1),
                FeeAsset::Relay
            ),
            TokenError::FundsUnavailable
        );

        assert_eq!(DaoManager::next_core_id(), 0u32);
        assert_eq!(DaoManager::core_storage(0u32), None);
    });
}

#[test]
fn set_parameters_works() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::Native,
        )
        .unwrap();

        assert_ok!(DaoManager::set_parameters(
            Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
            Some(vec![1, 2, 3].try_into().unwrap()),
            Some(Perbill::from_percent(100)),
            Some(Perbill::from_percent(100)),
            Some(false)
        ));

        assert_eq!(
            DaoManager::core_storage(0u32),
            Some(CoreInfo {
                account: DaoManager::derive_core_account(0u32),
                metadata: vec![1, 2, 3].try_into().unwrap(),
                minimum_support: Perbill::from_percent(100),
                required_approval: Perbill::from_percent(100),
                frozen_tokens: false,
            })
        );
    });
}

#[test]
fn set_parameters_fails() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::Native,
        )
        .unwrap();

        // Wrong origin.

        assert_err!(
            DaoManager::set_parameters(
                RawOrigin::Signed(ALICE).into(),
                Some(vec![1, 2, 3].try_into().unwrap()),
                Some(Perbill::from_percent(100)),
                Some(Perbill::from_percent(100)),
                Some(false)
            ),
            BadOrigin
        );

        // Core doesn't exist (can't actually happen as core id is taken from origin).

        assert_err!(
            DaoManager::set_parameters(
                Origin::Multisig(MultisigInternalOrigin::new(1u32)).into(),
                Some(vec![1, 2, 3].try_into().unwrap()),
                Some(Perbill::from_percent(100)),
                Some(Perbill::from_percent(100)),
                Some(false)
            ),
            Error::<Test>::CoreNotFound
        );
    });
}

#[test]
fn token_mint_works() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::Native,
        )
        .unwrap();

        assert_eq!(
            CoreAssets::accounts(ALICE, 0u32).free,
            CoreSeedBalance::get()
        );
        assert_eq!(DaoManager::core_members(0u32, ALICE), Some(()));

        assert_eq!(CoreAssets::accounts(BOB, 0u32).free, 0u128);
        assert_eq!(DaoManager::core_members(0u32, BOB), None);

        assert_ok!(DaoManager::token_mint(
            Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
            CoreSeedBalance::get(),
            BOB
        ));

        assert_eq!(CoreAssets::accounts(BOB, 0u32).free, CoreSeedBalance::get());
        assert_eq!(DaoManager::core_members(0u32, BOB), Some(()));
    });
}

#[test]
fn token_mint_fails() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::Native,
        )
        .unwrap();

        // Wrong origin.
        assert_err!(
            DaoManager::token_mint(RawOrigin::Signed(ALICE).into(), CoreSeedBalance::get(), BOB),
            BadOrigin
        );

        // Overflow
        assert_err!(
            DaoManager::token_mint(
                Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
                u128::MAX,
                ALICE
            ),
            ArithmeticError::Overflow
        );
    });
}

#[test]
fn token_burn_works() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::Native,
        )
        .unwrap();

        assert_eq!(
            CoreAssets::accounts(ALICE, 0u32).free,
            CoreSeedBalance::get()
        );
        assert_eq!(DaoManager::core_members(0u32, ALICE), Some(()));

        DaoManager::token_mint(
            Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
            CoreSeedBalance::get(),
            BOB,
        )
        .unwrap();

        assert_eq!(CoreAssets::accounts(BOB, 0u32).free, CoreSeedBalance::get());
        assert_eq!(DaoManager::core_members(0u32, BOB), Some(()));

        // Actual burn test

        assert_ok!(DaoManager::token_burn(
            Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
            CoreSeedBalance::get() / 2,
            ALICE
        ));

        assert_eq!(
            CoreAssets::accounts(ALICE, 0u32).free,
            CoreSeedBalance::get() / 2
        );
        assert_eq!(DaoManager::core_members(0u32, ALICE), Some(()));

        assert_ok!(DaoManager::token_burn(
            Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
            CoreSeedBalance::get(),
            BOB
        ));

        assert_eq!(CoreAssets::accounts(BOB, 0u32).free, 0u128);
        assert_eq!(DaoManager::core_members(0u32, BOB), None);
    });
}

#[test]
fn token_burn_fails() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::Native,
        )
        .unwrap();

        assert_eq!(
            CoreAssets::accounts(ALICE, 0u32).free,
            CoreSeedBalance::get()
        );
        assert_eq!(DaoManager::core_members(0u32, ALICE), Some(()));

        DaoManager::token_mint(
            Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
            CoreSeedBalance::get(),
            BOB,
        )
        .unwrap();

        assert_eq!(CoreAssets::accounts(BOB, 0u32).free, CoreSeedBalance::get());
        assert_eq!(DaoManager::core_members(0u32, BOB), Some(()));

        // Actual burn test

        // Wrong origin.
        assert_err!(
            DaoManager::token_burn(
                RawOrigin::Signed(ALICE).into(),
                CoreSeedBalance::get(),
                ALICE
            ),
            BadOrigin
        );

        // Underflow
        assert_err!(
            DaoManager::token_burn(
                Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
                CoreSeedBalance::get() * 3,
                ALICE
            ),
            ArithmeticError::Underflow
        );

        // Not enough to burn
        assert_err!(
            DaoManager::token_burn(
                Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
                CoreSeedBalance::get() + 1,
                ALICE
            ),
            TokenError::FundsUnavailable
        );
    });
}

#[test]
fn operate_multisig_works() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        // Test with single voter.

        assert_ok!(DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            Some(vec![1, 2, 3].try_into().unwrap()),
            FeeAsset::Native,
            Box::new(call.clone())
        ));

        System::assert_has_event(
            orml_tokens2::Event::Deposited {
                currency_id: 0u32,
                who: BOB,
                amount: CoreSeedBalance::get(),
            }
            .into(),
        );

        System::assert_has_event(
            Event::Minted {
                core_id: 0u32,
                target: BOB,
                amount: CoreSeedBalance::get(),
            }
            .into(),
        );

        System::assert_has_event(
            Event::MultisigExecuted {
                core_id: 0u32,
                executor_account: DaoManager::derive_core_account(0u32),
                voter: ALICE,
                call: call.clone(),
                call_hash: <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call),
                result: Ok(()),
            }
            .into(),
        );

        // Test with 2 voters, call should be stored for voting.

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call)
            ),
            None,
        );

        assert_ok!(DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            Some(vec![1, 2, 3].try_into().unwrap()),
            FeeAsset::Native,
            Box::new(call.clone())
        ));

        System::assert_has_event(
            Event::MultisigVoteStarted {
                core_id: 0u32,
                executor_account: DaoManager::derive_core_account(0u32),
                voter: ALICE,
                votes_added: Vote::Aye(CoreSeedBalance::get()),
                call_hash: <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call),
            }
            .into(),
        );

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: Some(vec![1, 2, 3].try_into().unwrap()),
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    Zero::zero(),
                    BoundedBTreeMap::try_from(BTreeMap::from([(
                        ALICE,
                        Vote::Aye(CoreSeedBalance::get())
                    )]))
                    .unwrap()
                ),
            })
        );
    });
}

#[test]
fn operate_multisig_fails() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        // Using this call now to add a second member to the multisig.
        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call.clone()),
        )
        .unwrap();

        // Not a member of the multisig
        assert_err!(
            DaoManager::operate_multisig(
                RawOrigin::Signed(CHARLIE).into(),
                0u32,
                Some(vec![1, 2, 3].try_into().unwrap()),
                FeeAsset::Native,
                Box::new(call.clone())
            ),
            Error::<Test>::NoPermission
        );

        // Max call length exceeded.
        assert_err!(
            DaoManager::operate_multisig(
                RawOrigin::Signed(ALICE).into(),
                0u32,
                None,
                FeeAsset::Native,
                Box::new(
                    frame_system::pallet::Call::<Test>::remark {
                        remark: vec![0u8; MAX_SIZE as usize]
                    }
                    .into()
                )
            ),
            Error::<Test>::MaxCallLengthExceeded
        );

        // Multisig call already exists in storage.
        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call.clone()),
        )
        .unwrap();
        assert_err!(
            DaoManager::operate_multisig(
                RawOrigin::Signed(ALICE).into(),
                0u32,
                None,
                FeeAsset::Native,
                Box::new(call.clone())
            ),
            Error::<Test>::MultisigCallAlreadyExists
        );
    });
}

#[test]
fn cancel_multisig_works() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            Some(vec![1, 2, 3].try_into().unwrap()),
            FeeAsset::Native,
            Box::new(call.clone()),
        )
        .unwrap();

        System::set_block_number(2);

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            Some(vec![1, 2, 3].try_into().unwrap()),
            FeeAsset::Native,
            Box::new(call.clone()),
        )
        .unwrap();

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: Some(vec![1, 2, 3].try_into().unwrap()),
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    Zero::zero(),
                    BoundedBTreeMap::try_from(BTreeMap::from([(
                        ALICE,
                        Vote::Aye(CoreSeedBalance::get())
                    )]))
                    .unwrap()
                ),
            })
        );

        assert_ok!(DaoManager::cancel_multisig_proposal(
            Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
            <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call)
        ));

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call)
            ),
            None
        );
    });
}

#[test]
fn cancel_multisig_fails() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            Some(vec![1, 2, 3].try_into().unwrap()),
            FeeAsset::Native,
            Box::new(call.clone()),
        )
        .unwrap();

        System::set_block_number(2);

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            Some(vec![1, 2, 3].try_into().unwrap()),
            FeeAsset::Native,
            Box::new(call.clone()),
        )
        .unwrap();

        // Wrong origin.
        assert_err!(
            DaoManager::cancel_multisig_proposal(
                RawOrigin::Signed(ALICE).into(),
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call)
            ),
            BadOrigin
        );

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: Some(vec![1, 2, 3].try_into().unwrap()),
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    Zero::zero(),
                    BoundedBTreeMap::try_from(BTreeMap::from([(
                        ALICE,
                        Vote::Aye(CoreSeedBalance::get())
                    )]))
                    .unwrap()
                ),
            })
        );
    });
}

#[test]
fn vote_multisig_works() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call1: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        let call2: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: CHARLIE,
        }
        .into();

        // Adding BOB.

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call1.clone()),
        )
        .unwrap();

        System::set_block_number(2);

        // Adding CHARLIE

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call2.clone()),
        )
        .unwrap();

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call2.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: None,
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    Zero::zero(),
                    BoundedBTreeMap::try_from(BTreeMap::from([(
                        ALICE,
                        Vote::Aye(CoreSeedBalance::get())
                    )]))
                    .unwrap()
                ),
            })
        );

        // BOB votes nay.

        assert_ok!(DaoManager::vote_multisig(
            RawOrigin::Signed(BOB).into(),
            0u32,
            <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            false
        ));

        System::assert_has_event(
            Event::MultisigVoteAdded {
                core_id: 0u32,
                executor_account: DaoManager::derive_core_account(0u32),
                voter: BOB,
                votes_added: Vote::Nay(CoreSeedBalance::get()),
                current_votes: Tally::from_parts(
                    CoreSeedBalance::get(),
                    CoreSeedBalance::get(),
                    BoundedBTreeMap::try_from(BTreeMap::from([
                        (ALICE, Vote::Aye(CoreSeedBalance::get())),
                        (BOB, Vote::Nay(CoreSeedBalance::get())),
                    ]))
                    .unwrap(),
                ),
                call_hash: <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            }
            .into(),
        );

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call2.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: None,
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    CoreSeedBalance::get(),
                    BoundedBTreeMap::try_from(BTreeMap::from([
                        (ALICE, Vote::Aye(CoreSeedBalance::get())),
                        (BOB, Vote::Nay(CoreSeedBalance::get()))
                    ]))
                    .unwrap()
                ),
            })
        );

        // BOB changes vote to aye, executing the call.

        assert_ok!(DaoManager::vote_multisig(
            RawOrigin::Signed(BOB).into(),
            0u32,
            <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            true
        ));

        System::assert_has_event(
            Event::MultisigExecuted {
                core_id: 0u32,
                executor_account: DaoManager::derive_core_account(0u32),
                voter: BOB,
                call: call2.clone(),
                call_hash: <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
                result: Ok(()),
            }
            .into(),
        );

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2)
            ),
            None
        );
    });
}

#[test]
fn vote_multisig_fails() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call1: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        let call2: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: CHARLIE,
        }
        .into();

        // Adding BOB.

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call1.clone()),
        )
        .unwrap();

        System::set_block_number(2);

        // Adding CHARLIE

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call2.clone()),
        )
        .unwrap();

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call2.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: None,
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    Zero::zero(),
                    BoundedBTreeMap::try_from(BTreeMap::from([(
                        ALICE,
                        Vote::Aye(CoreSeedBalance::get())
                    )]))
                    .unwrap()
                ),
            })
        );

        // Not a member of the multisig.
        assert_err!(
            DaoManager::vote_multisig(
                RawOrigin::Signed(DAVE).into(),
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
                true
            ),
            Error::<Test>::NoPermission
        );

        // Call not found.
        assert_err!(
            DaoManager::vote_multisig(
                RawOrigin::Signed(BOB).into(),
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call1),
                true
            ),
            Error::<Test>::MultisigCallNotFound
        );
    });
}

#[test]
fn withdraw_vote_multisig_works() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call1: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        let call2: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: CHARLIE,
        }
        .into();

        // Adding BOB.

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call1.clone()),
        )
        .unwrap();

        System::set_block_number(2);

        // Adding CHARLIE

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call2.clone()),
        )
        .unwrap();

        // BOB votes nay.

        assert_ok!(DaoManager::vote_multisig(
            RawOrigin::Signed(BOB).into(),
            0u32,
            <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            false
        ));

        System::assert_has_event(
            Event::MultisigVoteAdded {
                core_id: 0u32,
                executor_account: DaoManager::derive_core_account(0u32),
                voter: BOB,
                votes_added: Vote::Nay(CoreSeedBalance::get()),
                current_votes: Tally::from_parts(
                    CoreSeedBalance::get(),
                    CoreSeedBalance::get(),
                    BoundedBTreeMap::try_from(BTreeMap::from([
                        (ALICE, Vote::Aye(CoreSeedBalance::get())),
                        (BOB, Vote::Nay(CoreSeedBalance::get())),
                    ]))
                    .unwrap(),
                ),
                call_hash: <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            }
            .into(),
        );

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call2.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: None,
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    CoreSeedBalance::get(),
                    BoundedBTreeMap::try_from(BTreeMap::from([
                        (ALICE, Vote::Aye(CoreSeedBalance::get())),
                        (BOB, Vote::Nay(CoreSeedBalance::get()))
                    ]))
                    .unwrap()
                ),
            })
        );

        // BOB withdraws his vote.

        assert_ok!(DaoManager::withdraw_vote_multisig(
            RawOrigin::Signed(BOB).into(),
            0u32,
            <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
        ));

        System::assert_has_event(
            Event::MultisigVoteWithdrawn {
                core_id: 0u32,
                executor_account: DaoManager::derive_core_account(0u32),
                voter: BOB,
                votes_removed: Vote::Nay(CoreSeedBalance::get()),
                call_hash: <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            }
            .into(),
        );

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call2.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: None,
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    Zero::zero(),
                    BoundedBTreeMap::try_from(BTreeMap::from([(
                        ALICE,
                        Vote::Aye(CoreSeedBalance::get())
                    )]))
                    .unwrap()
                ),
            })
        );

        // ALICE also withdraws her vote.

        assert_ok!(DaoManager::withdraw_vote_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
        ));

        System::assert_has_event(
            Event::MultisigVoteWithdrawn {
                core_id: 0u32,
                executor_account: DaoManager::derive_core_account(0u32),
                voter: ALICE,
                votes_removed: Vote::Aye(CoreSeedBalance::get()),
                call_hash: <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            }
            .into(),
        );

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call2.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: None,
                tally: Tally::from_parts(Zero::zero(), Zero::zero(), BoundedBTreeMap::new()),
            })
        );
    });
}

#[test]
fn withdraw_vote_multisig_fails() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call1: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        let call2: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: CHARLIE,
        }
        .into();

        // Adding BOB.

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call1.clone()),
        )
        .unwrap();

        System::set_block_number(2);

        // Adding CHARLIE

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call2.clone()),
        )
        .unwrap();

        // BOB votes nay.

        assert_ok!(DaoManager::vote_multisig(
            RawOrigin::Signed(BOB).into(),
            0u32,
            <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            false
        ));

        // Multisig call not found.
        assert_err!(
            DaoManager::withdraw_vote_multisig(
                RawOrigin::Signed(BOB).into(),
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call1),
            ),
            Error::<Test>::MultisigCallNotFound
        );

        // Not a voter in this proposal.
        assert_err!(
            DaoManager::withdraw_vote_multisig(
                RawOrigin::Signed(CHARLIE).into(),
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2),
            ),
            Error::<Test>::NotAVoter
        );

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&call2)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(call2.clone().encode()).unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: None,
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    CoreSeedBalance::get(),
                    BoundedBTreeMap::try_from(BTreeMap::from([
                        (ALICE, Vote::Aye(CoreSeedBalance::get())),
                        (BOB, Vote::Nay(CoreSeedBalance::get()))
                    ]))
                    .unwrap()
                ),
            })
        );
    });
}

#[test]
fn core_address_matches() {
    const ACCOUNT_IN_ASSET_HUB: [u8; 32] = [
        147, 83, 7, 98, 71, 245, 98, 15, 146, 176, 22, 221, 20, 216, 188, 203, 166, 234, 117, 86,
        56, 214, 204, 37, 238, 26, 161, 82, 2, 174, 180, 74,
    ];

    let core_account = <Pallet<Test> as CoreAccountDerivation<Test>>::derive_core_account(0);

    let core_account_bytes: [u8; 32] = core_account.into();

    assert_eq!(core_account_bytes, ACCOUNT_IN_ASSET_HUB);
}

// SRLabs tests.
#[test]
fn vote_multisig_stack_overflow() {
    ExtBuilder::default().build().execute_with(|| {
        DaoManager::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![].try_into().unwrap(),
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::Native,
        )
        .unwrap();

        System::set_block_number(1);

        let call1: RuntimeCall = pallet::Call::token_mint {
            amount: CoreSeedBalance::get(),
            target: BOB,
        }
        .into();

        let mut nested_call: RuntimeCall = pallet::Call::operate_multisig {
            core_id: 0u32,
            metadata: None,
            fee_asset: FeeAsset::Native,
            call: Box::new(call1.clone()),
        }
        .into();

        for _ in 0..(sp_api::MAX_EXTRINSIC_DEPTH / 4) + 1 {
            nested_call = pallet::Call::operate_multisig {
                core_id: 0u32,
                metadata: None,
                fee_asset: FeeAsset::Native,
                call: Box::new(nested_call.clone()),
            }
            .into();
        }

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(call1.clone()),
        )
        .unwrap();

        System::set_block_number(2);

        DaoManager::operate_multisig(
            RawOrigin::Signed(ALICE).into(),
            0u32,
            None,
            FeeAsset::Native,
            Box::new(nested_call.clone()),
        )
        .unwrap();

        assert_eq!(
            DaoManager::multisig(
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&nested_call)
            ),
            Some(MultisigOperation {
                actual_call: BoundedCallBytes::<Test>::try_from(nested_call.clone().encode())
                    .unwrap(),
                fee_asset: FeeAsset::Native,
                original_caller: ALICE,
                metadata: None,
                tally: Tally::from_parts(
                    CoreSeedBalance::get(),
                    Zero::zero(),
                    BoundedBTreeMap::try_from(BTreeMap::from([(
                        ALICE,
                        Vote::Aye(CoreSeedBalance::get())
                    )]))
                    .unwrap()
                ),
            })
        );

        assert_err!(
            DaoManager::vote_multisig(
                RawOrigin::Signed(BOB).into(),
                0u32,
                <<Test as frame_system::Config>::Hashing as Hash>::hash_of(&nested_call),
                true
            ),
            Error::<Test>::FailedDecodingCall
        );
    });
}
