//! Unit tests for the IPS pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;
use sp_runtime::DispatchError;

#[test]
fn create_ips_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY)
        ));

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0, 1]
        ));
        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            vec![2]
        ));

        assert_eq!(Ips::next_ips_id(), 2);

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![0, 1]
            })
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(1),
            Some(IpsInfoOf::<Runtime> {
                owner: ALICE,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: vec![2]
            })
        );
    });
}

#[test]
fn create_ips_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY)
        ));

        assert_noop!(
            Ips::create_ips(Origin::none(), MOCK_METADATA.to_vec(), vec![0]),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Ips::create_ips(
                Origin::signed(BOB),
                MOCK_METADATA_PAST_MAX.to_vec(),
                vec![0]
            ),
            Error::<Runtime>::MaxMetadataExceeded,
        );
        assert_noop!(
            Ips::create_ips(Origin::signed(BOB), MOCK_METADATA.to_vec(), vec![1]),
            Error::<Runtime>::NoPermission,
        );
        assert_noop!(
            Ips::create_ips(Origin::signed(BOB), MOCK_METADATA.to_vec(), vec![2]),
            Error::<Runtime>::NoPermission, // BOB doesn't own that IPF because it doesn't exist, so he has no permission to use it
        );

        NextIpsId::<Runtime>::mutate(|id| *id = <Runtime as Config>::IpsId::max_value());
        assert_noop!(
            Ips::create_ips(Origin::signed(BOB), MOCK_METADATA.to_vec(), vec![0]),
            Error::<Runtime>::NoAvailableIpsId
        );

        assert_eq!(IpsStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn send_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));

        assert_ok!(Ips::send(Origin::signed(BOB), ALICE, 0));

        assert_eq!(IpsByOwner::<Runtime>::get(ALICE, 0), Some(()));
        assert_eq!(IpsByOwner::<Runtime>::get(BOB, 0), None);
    });
}

#[test]
fn send_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
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
            Ips::send(Origin::none(), ALICE, 0),
            DispatchError::BadOrigin
        );

        assert_noop!(
            Ips::send(Origin::signed(BOB), ALICE, 1),
            Error::<Runtime>::IpsNotFound
        );

        assert_noop!(
            Ips::send(Origin::signed(ALICE), BOB, 0),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            Ips::send(Origin::signed(BOB), BOB, 0),
            Error::<Runtime>::AlreadyOwned
        );

        assert_eq!(IpsByOwner::<Runtime>::get(BOB, 0), Some(()));
        assert_eq!(IpsByOwner::<Runtime>::get(ALICE, 0), None);
    });
}

#[test]
fn list_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));

        assert_ok!(Ips::list(Origin::signed(BOB), 0, Some(100)));
        assert_eq!(IpsPrices::<Runtime>::get(0), Some(100));

        assert_ok!(Ips::list(Origin::signed(BOB), 0, Some(50)));
        assert_eq!(IpsPrices::<Runtime>::get(0), Some(50));

        assert_ok!(Ips::list(Origin::signed(BOB), 0, None));
        assert_eq!(IpsPrices::<Runtime>::get(0), None);
    });
}

#[test]
fn list_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));
        assert_ok!(Ips::list(Origin::signed(BOB), 0, Some(100)));

        assert_noop!(
            Ips::list(Origin::none(), 0, Some(50)),
            DispatchError::BadOrigin
        );

        assert_noop!(
            Ips::list(Origin::signed(BOB), 1, Some(50)),
            Error::<Runtime>::IpsNotFound
        );

        assert_noop!(
            Ips::list(Origin::signed(ALICE), 0, Some(50)),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            Ips::list(Origin::signed(BOB), 0, Some(100)),
            Error::<Runtime>::SamePrice
        );

        assert_eq!(IpsPrices::<Runtime>::get(0), Some(100));
    });
}

#[test]
fn buy_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));

        assert_ok!(Ips::list(Origin::signed(BOB), 0, Some(100)));
        assert_eq!(IpsByOwner::<Runtime>::get(BOB, 0), Some(()));
        assert_eq!(IpsByOwner::<Runtime>::get(ALICE, 0), None);

        assert_ok!(Ips::buy(Origin::signed(ALICE), 0, 100));
        assert_eq!(IpsByOwner::<Runtime>::get(ALICE, 0), Some(()));
        assert_eq!(ipf::IpfByOwner::<Runtime>::get(ALICE, 0), Some(()));
        assert_eq!(IpsByOwner::<Runtime>::get(BOB, 0), None);
        assert_eq!(ipf::IpfByOwner::<Runtime>::get(BOB, 0), None);

        assert_eq!(IpsPrices::<Runtime>::get(0), None);
    });
}

#[test]
fn buy_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));
        assert_ok!(Ips::list(Origin::signed(BOB), 0, Some(100)));
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            vec![1]
        ));

        assert_noop!(Ips::buy(Origin::none(), 0, 100), DispatchError::BadOrigin);

        assert_noop!(
            Ips::buy(Origin::signed(ALICE), 2, 100),
            Error::<Runtime>::IpsNotFound
        );

        assert_noop!(
            Ips::buy(Origin::signed(BOB), 0, 100),
            Error::<Runtime>::BuyFromSelf
        );

        assert_noop!(
            Ips::buy(Origin::signed(BOB), 1, 100),
            Error::<Runtime>::NotForSale
        );

        assert_noop!(
            Ips::buy(Origin::signed(ALICE), 0, 50),
            Error::<Runtime>::PriceTooLow
        );

        assert_eq!(IpsPrices::<Runtime>::get(0), Some(100));
        assert_eq!(IpsPrices::<Runtime>::get(1), None);
    });
}

#[test]
fn destroy_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![0]
            })
        );

        assert_ok!(Ips::destroy(Origin::signed(BOB), 0));

        assert_eq!(IpsStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn destroy_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            vec![0]
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![0]
            })
        );

        assert_noop!(Ips::destroy(Origin::none(), 0), DispatchError::BadOrigin);
        assert_noop!(
            Ips::destroy(Origin::signed(BOB), 1),
            Error::<Runtime>::IpsNotFound
        );
        assert_noop!(
            Ips::destroy(Origin::signed(ALICE), 0),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![0]
            })
        );
    });
}
