//! Unit tests for the IPT pallet.

use std::iter::FromIterator;

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;
use sp_std::collections::btree_map::BTreeMap;

#[test]
fn create_dev_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));

        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));

        assert_ok!(Dev::create_dev(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            0u64,
            MOCK_DATA.to_vec(),
            vec![(ALICE, 50u32), (BOB, 50u32)],
            100u32,
            vec![H256::from(MOCK_DATA_SECONDARY)]
        ));

        assert_ok!(Dev::create_dev(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            1u64,
            MOCK_DATA.to_vec(),
            vec![(ALICE, 20u32), (BOB, 10u32)],
            100u32,
            vec![H256::from(MOCK_DATA_SECONDARY)]
        ));
    });
}

#[test]
fn create_dev_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));

        assert_noop!(
            Dev::create_dev(
                Origin::none(),
                MOCK_METADATA.to_vec(),
                0u64,
                MOCK_DATA.to_vec(),
                vec![(ALICE, 50u32), (BOB, 50u32)],
                100u32,
                vec![H256::from(MOCK_DATA_SECONDARY)]
            ),
            DispatchError::BadOrigin
        );

        assert_noop!(
            Dev::create_dev(
                Origin::signed(ALICE),
                MOCK_METADATA.to_vec(),
                0u64,
                MOCK_DATA.to_vec(),
                vec![(ALICE, 50u32), (BOB, 50u32)],
                100u32,
                vec![H256::from(MOCK_DATA_SECONDARY)]
            ),
            Error::<Runtime>::NoPermissionForIps
        );

        assert_noop!(
            Dev::create_dev(
                Origin::signed(BOB),
                MOCK_METADATA.to_vec(),
                0u64,
                MOCK_DATA.to_vec(),
                vec![(ALICE, 50u32), (BOB, 51u32)],
                100u32,
                vec![H256::from(MOCK_DATA_SECONDARY)]
            ),
            Error::<Runtime>::AllocationOverflow
        );
    });
}

#[test]
fn post_dev_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));
        assert_ok!(Dev::create_dev(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            0u64,
            MOCK_DATA.to_vec(),
            vec![(ALICE, 50u32), (BOB, 50u32)],
            100u32,
            vec![H256::from(MOCK_DATA_SECONDARY)]
        ));

        assert_ok!(Dev::post_dev(Origin::signed(BOB), 0));

        assert!(DevStorage::<Runtime>::get(0).unwrap().is_joinable);
    })
}

#[test]
fn post_dev_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));
        assert_ok!(Dev::create_dev(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            0u64,
            MOCK_DATA.to_vec(),
            vec![(ALICE, 50u32), (BOB, 50u32)],
            100u32,
            vec![H256::from(MOCK_DATA_SECONDARY)]
        ));
        assert_ok!(Ipt::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![1]
        ));
        assert_ok!(Dev::create_dev(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            1u64,
            MOCK_DATA.to_vec(),
            vec![(ALICE, 50u32), (BOB, 50u32)],
            100u32,
            vec![H256::from(MOCK_DATA_SECONDARY)]
        ));

        assert_noop!(Dev::post_dev(Origin::none(), 0), DispatchError::BadOrigin);

        assert_noop!(
            Dev::post_dev(Origin::signed(ALICE), 0),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            Dev::post_dev(Origin::signed(BOB), 1),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            Dev::post_dev(Origin::signed(BOB), 2),
            Error::<Runtime>::Unknown
        );

        assert!(!DevStorage::<Runtime>::get(0).unwrap().is_joinable);
        assert!(!DevStorage::<Runtime>::get(1).unwrap().is_joinable);
    })
}

#[test]
fn add_user_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));
        assert_ok!(Dev::create_dev(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            0u64,
            MOCK_DATA.to_vec(),
            vec![(BOB, 50u32)],
            100u32,
            vec![H256::from(MOCK_DATA_SECONDARY)]
        ));

        assert_ok!(Dev::post_dev(Origin::signed(BOB), 0u64));

        assert_eq!(
            DevStorage::<Runtime>::get(0u64).unwrap().ipo_allocations,
            BTreeMap::from_iter([(BOB, 50u32)])
        );

        assert_ok!(Dev::add_user(Origin::signed(BOB), 0u64, ALICE, 50u32));

        assert_eq!(
            DevStorage::<Runtime>::get(0u64).unwrap().ipo_allocations,
            BTreeMap::from_iter([(ALICE, 50u32), (BOB, 50u32)])
        );
    })
}

#[test]
fn add_user_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));
        assert_ok!(Dev::create_dev(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            0u64,
            MOCK_DATA.to_vec(),
            vec![(BOB, 50u32)],
            100u32,
            vec![H256::from(MOCK_DATA_SECONDARY)]
        ));

        assert_noop!(
            Dev::add_user(Origin::signed(BOB), 0u64, ALICE, 50u32),
            Error::<Runtime>::DevClosed
        );

        assert_ok!(Dev::post_dev(Origin::signed(BOB), 0u64));

        assert_eq!(
            DevStorage::<Runtime>::get(0u64).unwrap().ipo_allocations,
            BTreeMap::from_iter([(BOB, 50u32)])
        );

        assert_noop!(
            Dev::add_user(Origin::none(), 0u64, ALICE, 50u32),
            DispatchError::BadOrigin
        );

        assert_noop!(
            Dev::add_user(Origin::signed(ALICE), 0u64, ALICE, 50u32),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            Dev::add_user(Origin::signed(BOB), 1u64, ALICE, 50u32),
            Error::<Runtime>::Unknown
        );

        assert_noop!(
            Dev::add_user(Origin::signed(BOB), 0u64, ALICE, 51u32),
            Error::<Runtime>::AllocationOverflow
        );

        assert_eq!(
            DevStorage::<Runtime>::get(0u64).unwrap().ipo_allocations,
            BTreeMap::from_iter([(BOB, 50u32)])
        );
    })
}
