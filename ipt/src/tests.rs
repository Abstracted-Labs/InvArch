//! Unit tests for the IPT pallet.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

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
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], ()));
        assert_eq!(Ipt::next_ipt_id(IPS_ID), 1);
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], ()));
        assert_eq!(Ipt::next_ipt_id(IPS_ID), 2);

        let next_ips_id = Ipt::next_ips_id();
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_eq!(Ipt::next_ipt_id(next_ips_id), 0);
        assert_ok!(Ipt::mint(&BOB, next_ips_id, vec![1], ()));
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
            Ipt::mint(&BOB, IPS_ID, vec![1], ()),
            ArithmeticError::Overflow,
        );

        NextIptId::<Runtime>::mutate(IPS_ID, |id| *id = <Runtime as Config>::IptId::max_value());
        assert_noop!(
            Ipt::mint(&BOB, IPS_ID, vec![1], ()),
            Error::<Runtime>::NoAvailableIptId
        );
    });
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], ()));
        assert_ok!(Ipt::burn(&BOB, (IPS_ID, IPT_ID)));
    });
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipt::create_ips(&ALICE, vec![1], ()));
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], ()));
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
        assert_ok!(Ipt::mint(&BOB, IPS_ID, vec![1], ()));

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
            Ipt::mint(&BOB, IPS_ID, vec![1, 2], ()),
            Error::<Runtime>::MaxMetadataExceeded
        );
    });
}
