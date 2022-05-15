use sp_std::convert::TryInto;

use frame_support::{assert_noop, assert_ok};
use primitives::{utils::multi_account_id, OneOrPercent};

use crate::{
    mock::{ExtBuilder, InvArchLicenses, Ipl, Origin, Runtime, ALICE, BOB},
    AssetWeight, Config, Error, Ipl as IplStorage, IplInfoOf, LicenseList, Permissions,
};
use sp_runtime::{DispatchError, Percent};

type IplId = <Runtime as Config>::IplId;

macro_rules! percent {
    ($x:expr) => {
        OneOrPercent::ZeroPoint(Percent::from_percent($x))
    };
}

#[test]
fn set_permission_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipl::create(
            0,
            InvArchLicenses::GPLv3,
            percent!(50),
            OneOrPercent::One,
            false,
        );

        assert_ok!(Ipl::set_permission(
            Origin::signed(multi_account_id::<Runtime, IplId>(0, None)),
            0,
            Default::default(),
            [0, 0],
            true,
        ));

        assert_eq!(Permissions::<Runtime>::get((0, 0), [0, 0]), Some(true));
    });
}

#[test]
fn set_permission_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipl::create(
            0,
            InvArchLicenses::GPLv3,
            percent!(50),
            OneOrPercent::One,
            false,
        );

        // Case 0: Unsigned origin
        assert_noop!(
            Ipl::set_permission(Origin::none(), 0, 0, [0, 0], true),
            DispatchError::BadOrigin
        );

        // Case 1: Non-multisig origin
        assert_noop!(
            Ipl::set_permission(Origin::signed(ALICE), 0, 0, [0, 0], true),
            Error::<Runtime>::NoPermission
        );

        // Case 2: Ipl does not exist
        assert_noop!(
            Ipl::set_permission(
                Origin::signed(multi_account_id::<Runtime, IplId>(0, None)),
                32,
                0,
                [0, 0],
                true
            ),
            Error::<Runtime>::IplDoesntExist
        );

        assert_eq!(Permissions::<Runtime>::get((0, 0), [0, 0]), None);
    });
}

#[test]
fn set_asset_weight_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipl::create(
            0,
            InvArchLicenses::GPLv3,
            percent!(50),
            OneOrPercent::One,
            false,
        );

        assert_ok!(Ipl::set_asset_weight(
            Origin::signed(multi_account_id::<Runtime, IplId>(0, None)),
            0,
            0,
            percent!(30),
        ));

        assert_eq!(AssetWeight::<Runtime>::get(0, 0), Some(percent!(30)));
    });
}

#[test]
fn set_asset_weight_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        Ipl::create(
            0,
            InvArchLicenses::GPLv3,
            percent!(50),
            OneOrPercent::One,
            false,
        );

        // Case 0: Unsigned origin
        assert_noop!(
            Ipl::set_asset_weight(Origin::none(), 0, 0, percent!(50)),
            DispatchError::BadOrigin,
        );

        // Case 1: Non-multisig origin
        assert_noop!(
            Ipl::set_asset_weight(Origin::signed(BOB), 0, 0, percent!(50)),
            Error::<Runtime>::NoPermission,
        );

        // Case 2: Ipl does not exist
        assert_noop!(
            Ipl::set_asset_weight(
                Origin::signed(multi_account_id::<Runtime, IplId>(0, None)),
                32767,
                0,
                percent!(50)
            ),
            Error::<Runtime>::IplDoesntExist
        );

        assert_eq!(AssetWeight::<Runtime>::get(0, 0), None);
    });
}

// Test does not include "should_fail" since it's not meant to fail
#[test]
#[ignore]
fn create_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let license = InvArchLicenses::Custom(
            vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            [
                7, 57, 92, 251, 234, 183, 217, 144, 220, 196, 201, 132, 176, 249, 18, 224, 237,
                201, 2, 113, 146, 78, 111, 152, 92, 71, 16, 228, 87, 39, 81, 142,
            ]
            .into(),
        );
        Ipl::create(0, license.clone(), percent!(50), OneOrPercent::One, false);

        assert_eq!(
            IplStorage::<Runtime>::get(0),
            Some(IplInfoOf::<Runtime> {
                owner: multi_account_id::<Runtime, IplId>(0, None),
                id: 0,
                license: {
                    let (metadata, hash) = license.get_hash_and_metadata();
                    (metadata.try_into().unwrap(), hash)
                },
                execution_threshold: percent!(50),
                default_asset_weight: OneOrPercent::One,
                default_permission: false,
            })
        );
    });
}

// Not meant to fail
#[test]
fn execution_threshold_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipl::create(
            0,
            InvArchLicenses::GPLv3,
            percent!(35),
            OneOrPercent::One,
            false,
        );

        assert_eq!(Ipl::execution_threshold(0), Some(percent!(35)));
        assert_eq!(Ipl::execution_threshold(32767), None);
    });
}

// Not meant to fail
#[test]
fn asset_weight_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipl::create(
            0,
            InvArchLicenses::GPLv3,
            percent!(35),
            OneOrPercent::One,
            false,
        );

        assert_eq!(
            Ipl::asset_weight(0, 0),
            Some(OneOrPercent::One) // Default asset weight would be used
        );

        assert_ok!(Ipl::set_asset_weight(
            Origin::signed(multi_account_id::<Runtime, IplId>(0, None)),
            0,
            0,
            percent!(9)
        ));

        assert_eq!(Ipl::asset_weight(0, 0), Some(percent!(9)));
    });
}

#[test]
fn has_permission_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipl::create(
            0,
            InvArchLicenses::GPLv3,
            percent!(35),
            OneOrPercent::One,
            false,
        );

        assert_eq!(
            Ipl::has_permission(0, 0, [0, 0]),
            Some(false) //Default permission would be used
        );

        assert_ok!(Ipl::set_permission(
            Origin::signed(multi_account_id::<Runtime, IplId>(0, None)),
            0,
            0,
            [0, 0],
            true
        ));

        assert_eq!(Ipl::has_permission(0, 0, [0, 0]), Some(true));
    });
}
