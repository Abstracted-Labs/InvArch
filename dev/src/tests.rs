//! Unit tests for the IPT pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;

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
