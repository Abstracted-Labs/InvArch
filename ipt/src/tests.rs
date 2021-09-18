//! Unit tests for the IPT pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;

const MOCK_DATA: [u8; 32] = [
    12, 47, 182, 72, 140, 51, 139, 219, 171, 74, 247, 18, 123, 28, 200, 236, 221, 85, 25, 12, 218,
    0, 230, 247, 32, 73, 152, 66, 243, 27, 92, 95,
];

#[test]
fn create_ips_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
    });
}

#[test]
fn create_ips_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        NextIpsId::<Runtime>::mutate(|id| *id = <Runtime as Config>::IpsId::max_value());
        assert_noop!(
            Ipt::create_ips(&ALICE, vec![1], ()),
            Error::<Runtime>::NoAvailableIpsId
        );
    });
}

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let next_ips_id = Ipt::next_ips_id();
        assert_eq!(next_ips_id, IPS_ID);
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_eq!(Ipt::next_ipt_id(IPS_ID), 0);
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], H256::from(MOCK_DATA)));
        assert_eq!(Ipt::next_ipt_id(IPS_ID), 1);
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], H256::from(MOCK_DATA)));
        assert_eq!(Ipt::next_ipt_id(IPS_ID), 2);

        let next_ips_id = Ipt::next_ips_id();
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_eq!(Ipt::next_ipt_id(next_ips_id), 0);
        assert_ok!(Ipt::mint(&BOB, next_ips_id, vec![1], H256::from(MOCK_DATA)));
        assert_eq!(Ipt::next_ipt_id(next_ips_id), 1);

        assert_eq!(Ipt::next_ipt_id(IPS_ID), 2);
    });
}

#[test]
fn mint_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        IpsStorage::<Runtime>::mutate(IPS_ID, |ips_info| {
            ips_info.as_mut().unwrap().total_issuance = <Runtime as Config>::IptId::max_value();
        });
        assert_noop!(
            Ipt::mint(&BOB, IPS_ID, vec![1], H256::from(MOCK_DATA)),
            ArithmeticError::Overflow,
        );

        NextIptId::<Runtime>::mutate(IPS_ID, |id| *id = <Runtime as Config>::IptId::max_value());
        assert_noop!(
            Ipt::mint(&BOB, IPS_ID, vec![1], H256::from(MOCK_DATA)),
            Error::<Runtime>::NoAvailableIptId
        );
    });
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], H256::from(MOCK_DATA)));
        assert_ok!(Ipt::burn(&BOB, (IPS_ID, IPT_ID)));
    });
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], H256::from(MOCK_DATA)));
        assert_noop!(
            Ipt::burn(&BOB, (IPS_ID, IPT_ID_NOT_EXIST)),
            Error::<Runtime>::IptNotFound
        );

        assert_noop!(
            Ipt::burn(&ALICE, (IPS_ID, IPT_ID)),
            Error::<Runtime>::NoPermission
        );
    });

    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], H256::from(MOCK_DATA)));

        IpsStorage::<Runtime>::mutate(IPS_ID, |ips_info| {
            ips_info.as_mut().unwrap().total_issuance = 0;
        });
        assert_noop!(Ipt::burn(&BOB, (IPS_ID, IPT_ID)), ArithmeticError::Overflow,);
    });
}

#[test]
fn exceeding_max_metadata_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Ipt::create_ips(&ALICE, vec![1, 2], ()),
            Error::<Runtime>::MaxMetadataExceeded
        );
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_noop!(
            Ipt::mint(&BOB, IPS_ID, vec![1, 2], H256::from(MOCK_DATA)),
            Error::<Runtime>::MaxMetadataExceeded
        );
    });
}
