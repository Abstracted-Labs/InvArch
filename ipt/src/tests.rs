//! Unit tests for the IPT pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;
use sp_runtime::DispatchError;

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Ipt::next_ipt_id(), 0);
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_eq!(Ipt::next_ipt_id(), 1);
        assert_ok!(Ipt::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY)
        ));
        assert_eq!(Ipt::next_ipt_id(), 2);

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );

        assert_eq!(
            IptStorage::<Runtime>::get(1),
            Some(IptInfoOf::<Runtime> {
                owner: ALICE,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA_SECONDARY)
            })
        );
    });
}

#[test]
fn mint_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Ipt::mint(
                Origin::none(),
                MOCK_METADATA_PAST_MAX.to_vec(),
                H256::from(MOCK_DATA)
            ),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Ipt::mint(
                Origin::signed(BOB),
                MOCK_METADATA_PAST_MAX.to_vec(),
                H256::from(MOCK_DATA)
            ),
            Error::<Runtime>::MaxMetadataExceeded,
        );

        NextIptId::<Runtime>::mutate(|id| *id = <Runtime as Config>::IptId::max_value());
        assert_noop!(
            Ipt::mint(
                Origin::signed(BOB),
                MOCK_METADATA.to_vec(),
                H256::from(MOCK_DATA)
            ),
            Error::<Runtime>::NoAvailableIptId
        );

        assert_eq!(IptStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_ok!(Ipt::burn(Origin::signed(BOB), IPT_ID));

        assert_eq!(IptStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_noop!(Ipt::burn(Origin::none(), IPT_ID), DispatchError::BadOrigin);

        assert_noop!(
            Ipt::burn(Origin::signed(BOB), IPT_ID_DOESNT_EXIST),
            Error::<Runtime>::IptNotFound
        );

        assert_noop!(
            Ipt::burn(Origin::signed(ALICE), IPT_ID),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );
    });
}

#[test]
fn amend_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_DATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );

        assert_ok!(Ipt::amend(
            Origin::signed(BOB),
            IPT_ID,
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY)
        ));

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA_SECONDARY)
            })
        );
    });
}

#[test]
fn amend_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_noop!(
            Ipt::amend(
                Origin::none(),
                IPT_ID,
                MOCK_METADATA_SECONDARY.to_vec(),
                H256::from(MOCK_DATA_SECONDARY)
            ),
            DispatchError::BadOrigin
        );

        assert_noop!(
            Ipt::amend(
                Origin::signed(BOB),
                IPT_ID_DOESNT_EXIST,
                MOCK_METADATA_SECONDARY.to_vec(),
                H256::from(MOCK_DATA_SECONDARY)
            ),
            Error::<Runtime>::IptNotFound
        );

        assert_noop!(
            Ipt::amend(
                Origin::signed(ALICE),
                IPT_ID,
                MOCK_METADATA_SECONDARY.to_vec(),
                H256::from(MOCK_DATA_SECONDARY)
            ),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            Ipt::amend(
                Origin::signed(BOB),
                IPT_ID,
                MOCK_METADATA.to_vec(),
                H256::from(MOCK_DATA)
            ),
            Error::<Runtime>::AmendWithoutChanging
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );
    });
}
