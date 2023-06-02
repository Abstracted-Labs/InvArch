mod mock;

use crate::{origin::MultisigInternalOrigin, *};
use frame_support::{assert_err, assert_ok, error::BadOrigin};
use frame_system::RawOrigin;
use mock::*;
use primitives::CoreInfo;
use sp_runtime::{Perbill, TokenError};
use sp_std::{convert::TryInto, vec};

#[test]
fn create_core_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Balances::free_balance(ALICE), INITIAL_BALANCE);

        assert_eq!(INV4::next_core_id(), 0u32);

        assert_eq!(INV4::core_storage(0u32), None);

        assert_ok!(INV4::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![],
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::TNKR
        ));

        assert_eq!(INV4::next_core_id(), 1u32);

        assert_eq!(
            INV4::core_storage(0u32),
            Some(CoreInfo {
                account: util::derive_core_account::<Test, u32, u32>(0u32),
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

        assert_eq!(INV4::next_core_id(), 1u32);

        assert_eq!(INV4::core_storage(1u32), None);

        assert_ok!(INV4::create_core(
            RawOrigin::Signed(BOB).into(),
            vec![1, 2, 3],
            Perbill::from_percent(100),
            Perbill::from_percent(100),
            FeeAsset::KSM
        ));

        assert_eq!(INV4::next_core_id(), 2u32);

        assert_eq!(
            INV4::core_storage(1u32),
            Some(CoreInfo {
                account: util::derive_core_account::<Test, u32, u32>(1u32),
                metadata: vec![1, 2, 3].try_into().unwrap(),
                minimum_support: Perbill::from_percent(100),
                required_approval: Perbill::from_percent(100),
                frozen_tokens: true,
            })
        );

        assert_eq!(
            Tokens::accounts(BOB, KSM_ASSET_ID).free,
            INITIAL_BALANCE - KSMCoreCreationFee::get()
        );
    });
}

#[test]
fn create_core_fails() {
    ExtBuilder::default().build().execute_with(|| {
        // Not enough balance for creation fee.

        assert_eq!(Balances::free_balance(CHARLIE), 0u128);

        assert_eq!(INV4::next_core_id(), 0u32);

        assert_eq!(INV4::core_storage(0u32), None);

        assert_err!(
            INV4::create_core(
                RawOrigin::Signed(CHARLIE).into(),
                vec![],
                Perbill::from_percent(1),
                Perbill::from_percent(1),
                FeeAsset::TNKR
            ),
            pallet_balances::Error::<Test>::InsufficientBalance
        );

        assert_eq!(INV4::next_core_id(), 0u32);
        assert_eq!(INV4::core_storage(0u32), None);

        // With KSM.

        assert_eq!(Tokens::accounts(CHARLIE, KSM_ASSET_ID).free, 0u128);

        assert_err!(
            INV4::create_core(
                RawOrigin::Signed(CHARLIE).into(),
                vec![],
                Perbill::from_percent(1),
                Perbill::from_percent(1),
                FeeAsset::KSM
            ),
            TokenError::NoFunds
        );

        assert_eq!(INV4::next_core_id(), 0u32);
        assert_eq!(INV4::core_storage(0u32), None);

        // Max metadata exceeded

        assert_err!(
            INV4::create_core(
                RawOrigin::Signed(ALICE).into(),
                vec![0u8; (MaxMetadata::get() + 1) as usize],
                Perbill::from_percent(1),
                Perbill::from_percent(1),
                FeeAsset::TNKR
            ),
            Error::<Test>::MaxMetadataExceeded
        );

        assert_eq!(INV4::next_core_id(), 0u32);
        assert_eq!(INV4::core_storage(0u32), None);
    });
}

#[test]
fn set_parameters_works() {
    ExtBuilder::default().build().execute_with(|| {
        INV4::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![],
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::TNKR,
        )
        .unwrap();

        assert_ok!(INV4::set_parameters(
            Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
            Some(vec![1, 2, 3]),
            Some(Perbill::from_percent(100)),
            Some(Perbill::from_percent(100)),
            Some(false)
        ));

        assert_eq!(
            INV4::core_storage(0u32),
            Some(CoreInfo {
                account: util::derive_core_account::<Test, u32, u32>(0u32),
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
        INV4::create_core(
            RawOrigin::Signed(ALICE).into(),
            vec![],
            Perbill::from_percent(1),
            Perbill::from_percent(1),
            FeeAsset::TNKR,
        )
        .unwrap();

        // Wrong origin.

        assert_err!(
            INV4::set_parameters(
                RawOrigin::Signed(ALICE).into(),
                Some(vec![1, 2, 3]),
                Some(Perbill::from_percent(100)),
                Some(Perbill::from_percent(100)),
                Some(false)
            ),
            BadOrigin
        );

        // Core doesn't exist (can't actually happen as core id is taken from origin).

        assert_err!(
            INV4::set_parameters(
                Origin::Multisig(MultisigInternalOrigin::new(1u32)).into(),
                Some(vec![1, 2, 3]),
                Some(Perbill::from_percent(100)),
                Some(Perbill::from_percent(100)),
                Some(false)
            ),
            Error::<Test>::CoreNotFound
        );

        // Max metadata exceeded.

        assert_err!(
            INV4::set_parameters(
                Origin::Multisig(MultisigInternalOrigin::new(0u32)).into(),
                Some(vec![0u8; (MaxMetadata::get() + 1) as usize],),
                None,
                None,
                None
            ),
            Error::<Test>::MaxMetadataExceeded
        );
    });
}
