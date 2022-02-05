//! Unit tests for the IPF pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;
use sp_runtime::DispatchError;

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Ipf::next_ipf_id(), 0);
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_eq!(Ipf::next_ipf_id(), 1);
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY)
        ));
        assert_eq!(Ipf::next_ipf_id(), 2);

        assert_eq!(
            IpfStorage::<Runtime>::get(0),
            Some(IpfInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );

        assert_eq!(
            IpfStorage::<Runtime>::get(1),
            Some(IpfInfoOf::<Runtime> {
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
            Ipf::mint(
                Origin::none(),
                MOCK_METADATA_PAST_MAX.to_vec(),
                H256::from(MOCK_DATA)
            ),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Ipf::mint(
                Origin::signed(BOB),
                MOCK_METADATA_PAST_MAX.to_vec(),
                H256::from(MOCK_DATA)
            ),
            Error::<Runtime>::MaxMetadataExceeded,
        );

        NextIpfId::<Runtime>::mutate(|id| *id = <Runtime as Config>::IpfId::max_value());
        assert_noop!(
            Ipf::mint(
                Origin::signed(BOB),
                MOCK_METADATA.to_vec(),
                H256::from(MOCK_DATA)
            ),
            Error::<Runtime>::NoAvailableIpfId
        );

        assert_eq!(IpfStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_ok!(Ipf::burn(Origin::signed(BOB), IPF_ID));

        assert_eq!(IpfStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_noop!(Ipf::burn(Origin::none(), IPF_ID), DispatchError::BadOrigin);

        assert_noop!(
            Ipf::burn(Origin::signed(BOB), IPF_ID_DOESNT_EXIST),
            Error::<Runtime>::IpfNotFound
        );

        assert_noop!(
            Ipf::burn(Origin::signed(ALICE), IPF_ID),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IpfStorage::<Runtime>::get(0),
            Some(IpfInfoOf::<Runtime> {
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
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(
            IpfStorage::<Runtime>::get(0),
            Some(IpfInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_DATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );

        assert_ok!(Ipf::amend(
            Origin::signed(BOB),
            IPF_ID,
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY)
        ));

        assert_eq!(
            IpfStorage::<Runtime>::get(0),
            Some(IpfInfoOf::<Runtime> {
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
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_noop!(
            Ipf::amend(
                Origin::none(),
                IPF_ID,
                MOCK_METADATA_SECONDARY.to_vec(),
                H256::from(MOCK_DATA_SECONDARY)
            ),
            DispatchError::BadOrigin
        );

        assert_noop!(
            Ipf::amend(
                Origin::signed(BOB),
                IPF_ID_DOESNT_EXIST,
                MOCK_METADATA_SECONDARY.to_vec(),
                H256::from(MOCK_DATA_SECONDARY)
            ),
            Error::<Runtime>::IpfNotFound
        );

        assert_noop!(
            Ipf::amend(
                Origin::signed(ALICE),
                IPF_ID,
                MOCK_METADATA_SECONDARY.to_vec(),
                H256::from(MOCK_DATA_SECONDARY)
            ),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            Ipf::amend(
                Origin::signed(BOB),
                IPF_ID,
                MOCK_METADATA.to_vec(),
                H256::from(MOCK_DATA)
            ),
            Error::<Runtime>::AmendWithoutChanging
        );

        assert_eq!(
            IpfStorage::<Runtime>::get(0),
            Some(IpfInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );
    });
}
