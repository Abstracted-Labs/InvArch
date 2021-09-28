//! Unit tests for the IPO pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;
use sp_runtime::DispatchError;

#[test]
fn issue_ipo_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Ipo::next_ipo_id(), 0);
        assert_ok!(Ipo::issue_ipo(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA),
            MOCK_TOTAL_ISSUANCE.to_vec()
        ));
        assert_eq!(Ipo::next_ipo_id(), 1);
        assert_ok!(Ipo::issue_ipo(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY),
            MOCK_TOTAL_ISSUANCE.to_vec()
        ));
        assert_eq!(Ipo::next_ipo_id(), 2);

        assert_eq!(
            IpoStorage::<T>::get(0),
            Some(IpoInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA),
                total_issuance: MOCK_TOTAL_ISSUANCE.to_vec().try_into().unwrap()
            })
        );

        assert_eq!(
            IpoStorage::<T>::get(1),
            Some(IpoInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA_SECONDARY),
                total_issuance: MOCK_TOTAL_ISSUANCE.to_vec().try_into().unwrap()
            })
        );
    });
}

fn issue_ipo_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Ipo::issue_ipo(
                Origin::none(),
                MOCK_METADATA_PAST_MAX.to_vec(),
                H256::from(MOCK_DATA),
                MOCK_TOTAL_ISSUANCE.to_vec()
            ),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Ipo::issue_ipo(
                Origin::signed(BOB),
                MOCK_METADATA_PAST_MAX.to_vec(),
                H256::from(MOCK_DATA),
                MOCK_TOTAL_ISSUANCE.to_vec()
            ),
            Error::<Runtime>::MaxMetadataExceeded,
        );

        NextIpoId::<Runtime>::mutate(|id| *id = <Runtime as Config>::IpoId::max_value());
        assert_noop!(
            Ipo::issue_ipo(
                Origin::signed(BOB),
                MOCK_METADATA.to_vec(),
                H256::from(MOCK_DATA),
                MOCK_TOTAL_ISSUANCE.to_vec()
            ),
            Error::<Runtime>::NoAvailableIpoId
        );

        assert_eq!(IpoStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipo::transfer(
            Origin::signed(BOB),
            AccountId::get(ALICE),
            MOCK_AMOUNT.to_vec(),
        ));

        assert_ok!(Ipo::transfer(Origin::signed(BOB), IPO_ID));

        assert_eq!(IpoStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn transfer_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipo::transfer(
            Origin::signed(BOB),
            AccountId::get(ALICE),
            MOCK_AMOUNT.to_vec(),
        ));

        assert_noop!(Ipo::transfer(Origin::none(), IPO_ID), DispatchError::BadOrigin);

        assert_noop!(
            Ipo::transfer(Origin::signed(BOB), IPO_ID_DOESNT_EXIST),
            Error::<Runtime>::IpoNotFound
        );

        assert_noop!(
            Ipo::transfer(Origin::signed(ALICE), IPO_ID),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IpoStorage::<Runtime>::get(0),
            Some(IpoInfoOf::<Runtime> {
                owned: BOB,
                AccountId::get(BOB),
                MOCK_AMOUNT.to_vec()
            })
        );
    });
}

#[test]
fn set_balance_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipo::set_balance(
            Origin::root(),
            Balance::new(),
        ));

        assert_ok!(IpoStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn set_balance_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipo::set_balance(
            Origin::root(),
            Balance::new()
        ));

        assert_noop!(Ipo::set_balance(Origin::none(), IPO_ID), DispatchError::BadOrigin);

        assert_noop!(
            Ipo::set_balance(Origin::root(), IPO_ID_DOESNT_EXIST),
            Error::<Runtime>::IpoNotFound
        );

        assert_noop!(
            Ipo::set_balance(Origin::signed(ALICE), IPO_ID),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IpoStorage::<Runtime>::get(0),
            Some(IpoInfoOf::<Runtime> {
                origin: ROOT,
                amount: MOCK_AMOUNT.to_vec().try_into().unwrap()
            })
        );
    });
}

#[test]
fn bind_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipo::bind(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_ok!(Ipo::bind(Origin::signed(BOB), IPO_ID));

        assert_eq!(IpoStorage::<Runtime>::get(0), None);
    });
}

#[test]
fn bind_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipo::bind(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_noop!(Ipo::bind(Origin::none(), IPO_ID), DispatchError::BadOrigin);

        assert_noop!(
            Ipo::bind(Origin::signed(BOB), IPO_ID_DOESNT_EXIST),
            Error::<Runtime>::IpoNotFound
        );

        assert_noop!(
            Ipo::bind(Origin::signed(ALICE), IPO_ID),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IpoStorage::<Runtime>::get(0),
            Some(IpoInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );
    });
}

#[test]
fn unbind_should_work() {}

#[test]
fn unbind_should_fail() {}