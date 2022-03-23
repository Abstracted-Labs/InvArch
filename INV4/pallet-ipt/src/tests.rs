//! Unit tests for the IPT pallet.

use frame_support::{assert_noop, assert_ok};

use crate::{
    mock::{ExistentialDeposit, ExtBuilder, Ipt, Origin, Runtime, ALICE, BOB},
    AssetDetails, Error, Ipt as IptStorage,
};

use sp_runtime::DispatchError;

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(ALICE, 0, vec![(ALICE, ExistentialDeposit::get())]);

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
                deposit: 0,
            })
        );

        assert_ok!(Ipt::mint(Origin::signed(ALICE), 0, 1000, ALICE));

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get() + 1000,
                deposit: 0,
            })
        );
    });
}

#[test]
fn mint_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(ALICE, 0, vec![(ALICE, ExistentialDeposit::get())]);

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
                deposit: 0,
            })
        );

        // Case 0: Unknown origin
        assert_noop!(
            Ipt::mint(Origin::none(), 0, 1000, ALICE),
            DispatchError::BadOrigin
        );

        assert_ne!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get() + 1000,
                deposit: 0,
            })
        );

        // Case 1: Ipt Does not exist
        assert_noop!(
            Ipt::mint(Origin::signed(ALICE), 32, 1000, ALICE),
            Error::<Runtime>::IptDoesntExist
        );

        assert_eq!(IptStorage::<Runtime>::get(32), None);

        // Case 2: Caller has no permission
        assert_noop!(
            Ipt::mint(Origin::signed(BOB), 0, 1000, ALICE),
            Error::<Runtime>::NoPermission,
        );

        assert_ne!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get() + 1000,
                deposit: 0,
            })
        );
    });
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(ALICE, 0, vec![(ALICE, ExistentialDeposit::get())]);

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
                deposit: 0,
            })
        );

        assert_ok!(Ipt::burn(Origin::signed(ALICE), 0, 500, ALICE));

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: 0,
                deposit: 0,
            })
        );
    });
}
#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(ALICE, 0, vec![(ALICE, ExistentialDeposit::get())]);

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
                deposit: 0,
            })
        );

        // Case 0: Unknown origin
        assert_noop!(
            Ipt::burn(Origin::none(), 0, 500, ALICE),
            DispatchError::BadOrigin
        );

        assert_ne!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: 0,
                deposit: 0,
            })
        );

        // Case 1: Ipt does not exist
        assert_noop!(
            Ipt::burn(Origin::signed(ALICE), 32, 500, ALICE),
            Error::<Runtime>::IptDoesntExist
        );

        assert_eq!(IptStorage::<Runtime>::get(32), None);

        // Case 2: Caller has no permission
        assert_noop!(
            Ipt::burn(Origin::signed(BOB), 0, 500, ALICE),
            Error::<Runtime>::NoPermission
        );

        assert_ne!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: 0,
                deposit: 0,
            })
        );
    });
}
