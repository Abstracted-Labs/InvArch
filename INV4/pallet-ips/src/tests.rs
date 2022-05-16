//! Unit tests for the IPS pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use ipf::{IpfInfoOf, IpfStorage};
use ipt::{Ipt as IptStorage, SubAssets};
use mock::InvArchLicenses::*;
use mock::*;
use primitives::{
    utils::multi_account_id,
    AnyId, IpsType, IptInfo,
    OneOrPercent::{One, ZeroPoint},
    Parentage, SubIptInfo,
};
use sp_core::H256;
use sp_runtime::{DispatchError, Percent};

pub type IpsId = <Runtime as Config>::IpsId;

macro_rules! percent {
    ($x:expr) => {
        ZeroPoint(Percent::from_percent($x))
    };
}

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
            vec![0, 1],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false,
        ));

        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            vec![2],
            false,
            Some(vec![SubIptInfo {
                id: 0,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap()
            }]),
            GPLv3,
            percent!(50),
            One,
            false,
        ));

        assert_eq!(Ips::next_ips_id(), 2);

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0), AnyId::IpfId(1)].try_into().unwrap(),
                allow_replica: true,
                ips_type: IpsType::Normal,
            })
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(1),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(1, None)),
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(2)].try_into().unwrap(),
                allow_replica: false,
                ips_type: IpsType::Normal
            })
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0).unwrap().supply,
            ExistentialDeposit::get()
        );

        assert_eq!(SubAssets::<Runtime>::get(0, 0), None);

        assert_eq!(
            IptStorage::<Runtime>::get(1).unwrap().supply,
            ExistentialDeposit::get()
        );

        assert_eq!(
            SubAssets::<Runtime>::get(1, 0),
            Some(SubIptInfo {
                id: 0,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap()
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
            Ips::create_ips(
                Origin::none(),
                MOCK_METADATA.to_vec(),
                vec![0],
                true,
                None,
                GPLv3,
                percent!(50),
                One,
                false
            ),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Ips::create_ips(
                Origin::signed(BOB),
                MOCK_METADATA_PAST_MAX.to_vec(),
                vec![0],
                true,
                None,
                GPLv3,
                percent!(50),
                One,
                false
            ),
            Error::<Runtime>::MaxMetadataExceeded,
        );
        assert_noop!(
            Ips::create_ips(
                Origin::signed(BOB),
                MOCK_METADATA.to_vec(),
                vec![1],
                true,
                None,
                GPLv3,
                percent!(50),
                One,
                false
            ),
            Error::<Runtime>::NoPermission,
        );
        assert_noop!(
            Ips::create_ips(
                Origin::signed(BOB),
                MOCK_METADATA.to_vec(),
                vec![2],
                true,
                None,
                GPLv3,
                percent!(50),
                One,
                false
            ),
            Error::<Runtime>::NoPermission, // BOB doesn't own that IPF because it doesn't exist, so he has no permission to use it
        );

        NextIpsId::<Runtime>::mutate(|id| *id = IpsId::max_value());
        assert_noop!(
            Ips::create_ips(
                Origin::signed(BOB),
                MOCK_METADATA.to_vec(),
                vec![0],
                true,
                None,
                GPLv3,
                percent!(50),
                One,
                false
            ),
            Error::<Runtime>::NoAvailableIpsId
        );

        assert_eq!(IpsStorage::<Runtime>::get(0), None);
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
            vec![0],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                allow_replica: true,
                ips_type: IpsType::Normal
            })
        );

        assert_ok!(Ips::destroy(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
            0
        ));

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
            vec![0],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                allow_replica: true,
                ips_type: IpsType::Normal,
            })
        );

        assert_noop!(Ips::destroy(Origin::none(), 0), DispatchError::BadOrigin);
        assert_noop!(
            Ips::destroy(
                Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
                1
            ),
            Error::<Runtime>::IpsNotFound
        );
        assert_noop!(
            Ips::destroy(Origin::signed(ALICE), 0),
            Error::<Runtime>::NoPermission
        );
        assert_noop!(
            Ips::destroy(
                Origin::signed(multi_account_id::<Runtime, IpsId>(1, None)),
                0
            ),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                allow_replica: true,
                ips_type: IpsType::Normal,
            })
        );
    });
}

#[test]
fn create_replica_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        // Case 0: Alice replicates her own IPS
        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_replica(
            Origin::signed(ALICE),
            0,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        // Case 1: Bob replicates Alice's IPS
        assert_eq!(Ips::next_ips_id(), 2);
        assert_ok!(Ips::create_replica(
            Origin::signed(BOB),
            0,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        let ips_0 = IpsStorage::<Runtime>::get(0).unwrap();
        let ips_1 = IpsStorage::<Runtime>::get(1).unwrap();

        assert_eq!(
            ips_0,
            IpsInfo {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Normal,
                allow_replica: true,
            }
        );

        assert_eq!(
            ips_1,
            IpsInfo {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(1, None)),
                metadata: ips_0.metadata,
                data: ips_0.data,
                ips_type: IpsType::Replica(0),
                allow_replica: false
            }
        );
    });
}

#[test]
fn create_replica_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            false,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        // Case 0: An unknown origin tries to replicate a non-replicable IPS
        assert_noop!(
            Ips::create_replica(Origin::none(), 0, GPLv3, percent!(50), One, false),
            DispatchError::BadOrigin,
        );

        // Case 1: Alice didn't allow replicas and tried to replicate her own IPS
        assert_eq!(Ips::next_ips_id(), 1);
        assert_noop!(
            Ips::create_replica(Origin::signed(ALICE), 0, GPLv3, percent!(50), One, false),
            Error::<Runtime>::ReplicaNotAllowed
        );

        // Case 2: Bob tried to replicate Alice's IPS
        assert_eq!(Ips::next_ips_id(), 1);
        assert_noop!(
            Ips::create_replica(Origin::signed(BOB), 0, GPLv3, percent!(50), One, false),
            Error::<Runtime>::ReplicaNotAllowed,
        );

        // Case 3: Alice allows replica, then replicates IPS 0. Soon, Bob tries to replicate Alice's replica.
        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::allow_replica(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
            0
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Normal,
                allow_replica: true,
            })
        );

        // Subcase 0: An unknown origin tries to replicate a replicable IPS
        assert_noop!(
            Ips::create_replica(Origin::none(), 0, GPLv3, percent!(50), One, false),
            DispatchError::BadOrigin
        );

        assert_ok!(Ips::create_replica(
            Origin::signed(ALICE),
            0,
            GPLv3,
            percent!(5),
            One,
            false
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(1),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(1, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Replica(0),
                allow_replica: false,
            })
        );

        assert_noop!(
            Ips::create_replica(Origin::signed(BOB), 1, GPLv3, percent!(50), One, false),
            Error::<Runtime>::ReplicaNotAllowed
        );

        assert_eq!(Ips::next_ips_id(), 2);

        // Case 4: Original Ips does not exist
        assert_noop!(
            Ips::create_replica(Origin::signed(BOB), 2, GPLv3, percent!(50), One, false),
            Error::<Runtime>::IpsNotFound
        );

        // Case 5: IpsId Overflow
        NextIpsId::<Runtime>::mutate(|id| *id = IpsId::max_value());
        assert_noop!(
            Ips::create_replica(Origin::signed(BOB), 0, GPLv3, percent!(50), One, false),
            Error::<Runtime>::NoAvailableIpsId
        );
    });
}

#[test]
fn allow_replica_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            false,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::allow_replica(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
            0
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                allow_replica: true,
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Normal
            })
        )
    })
}

#[test]
fn allow_replica_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            false,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![1],
            false,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0,
                Some(multi_account_id::<Runtime, IpsId>(1, None))
            )),
            0,
            vec![AnyId::IpsId(1)],
            None
        ));

        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 2);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![2],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::create_replica(
            Origin::signed(ALICE),
            2,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        // Case 0: Extrinsic called in a non-multisig context:
        assert_noop!(
            Ips::allow_replica(Origin::signed(ALICE), 0),
            Error::<Runtime>::NoPermission
        );

        // Case 1: An unknown origin tries to allow replica on IPS 0:
        assert_noop!(
            Ips::allow_replica(Origin::none(), 0),
            DispatchError::BadOrigin,
        );

        assert_noop!(
            Ips::allow_replica(
                Origin::signed(multi_account_id::<Runtime, IpsId>(4, None)),
                4
            ),
            Error::<Runtime>::IpsNotFound,
        );

        assert_noop!(
            Ips::allow_replica(
                Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
                1
            ),
            Error::<Runtime>::NotParent,
        );

        assert_noop!(
            Ips::allow_replica(
                Origin::signed(multi_account_id::<Runtime, IpsId>(2, None)),
                2
            ),
            Error::<Runtime>::ValueNotChanged,
        );

        assert_noop!(
            Ips::allow_replica(
                Origin::signed(multi_account_id::<Runtime, IpsId>(3, None)),
                3
            ),
            Error::<Runtime>::ReplicaCannotAllowReplicas,
        );

        assert_eq!(IpsStorage::<Runtime>::get(0).unwrap().allow_replica, false);
        assert_eq!(IpsStorage::<Runtime>::get(1).unwrap().allow_replica, false);
        assert_eq!(IpsStorage::<Runtime>::get(2).unwrap().allow_replica, true);
        assert_eq!(IpsStorage::<Runtime>::get(3).unwrap().allow_replica, false);
    })
}

#[test]
fn disallow_replica_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            true,
            None,
            GPLv3,
            percent!(5),
            One,
            false
        ));

        assert_ok!(Ips::disallow_replica(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
            0
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                allow_replica: false,
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Normal
            })
        )
    })
}

#[test]
fn disallow_replica_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![1],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0,
                Some(multi_account_id::<Runtime, IpsId>(1, None))
            )),
            0,
            vec![AnyId::IpsId(1)],
            None
        ));

        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_eq!(Ips::next_ips_id(), 2);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![2],
            false,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::create_replica(
            Origin::signed(ALICE),
            1,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        // Case 0: Extrinsic called in a non-multisig context:
        assert_noop!(
            Ips::disallow_replica(Origin::signed(ALICE), 0),
            Error::<Runtime>::NoPermission
        );

        // Case 1: An unknown origin tries to allow replica on IPS 0:
        assert_noop!(
            Ips::disallow_replica(Origin::none(), 0),
            DispatchError::BadOrigin,
        );

        assert_noop!(
            Ips::disallow_replica(
                Origin::signed(multi_account_id::<Runtime, IpsId>(4, None)),
                4
            ),
            Error::<Runtime>::IpsNotFound,
        );

        assert_noop!(
            Ips::disallow_replica(
                Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
                1
            ),
            Error::<Runtime>::NotParent,
        );

        assert_noop!(
            Ips::disallow_replica(
                Origin::signed(multi_account_id::<Runtime, IpsId>(2, None)),
                2
            ),
            Error::<Runtime>::ValueNotChanged,
        );

        assert_noop!(
            Ips::disallow_replica(
                Origin::signed(multi_account_id::<Runtime, IpsId>(3, None)),
                3
            ),
            Error::<Runtime>::ReplicaCannotAllowReplicas,
        );

        assert_eq!(IpsStorage::<Runtime>::get(0).unwrap().allow_replica, true);
        assert_eq!(IpsStorage::<Runtime>::get(1).unwrap().allow_replica, true);
        assert_eq!(IpsStorage::<Runtime>::get(2).unwrap().allow_replica, false);
        assert_eq!(IpsStorage::<Runtime>::get(3).unwrap().allow_replica, false);
    })
}

#[test]
fn append_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));

        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY),
        ));

        assert_eq!(
            IpfStorage::<Runtime>::get(0),
            Some(IpfInfoOf::<Runtime> {
                owner: ALICE,
                author: ALICE,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA),
            })
        );

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_eq!(
            IpfStorage::<Runtime>::get(0),
            Some(IpfInfoOf::<Runtime> {
                owner: multi_account_id::<Runtime, IpsId>(0, None),
                author: ALICE,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA),
            })
        );

        assert_ok!(Ipt::mint(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
            (0, None),
            1000,
            ALICE
        ));

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IpsId>(0, None),
                supply: 1000 + ExistentialDeposit::get(),
            })
        );

        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_replica(
            Origin::signed(ALICE),
            0,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_eq!(
            IptStorage::<Runtime>::get(1),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IpsId>(1, None),
                supply: ExistentialDeposit::get(),
            })
        );

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, Some(ALICE))),
            0,
            vec![AnyId::IpfId(1)],
            None
        ));

        assert_eq!(
            IpfStorage::<Runtime>::get(1),
            Some(IpfInfoOf::<Runtime> {
                owner: multi_account_id::<Runtime, IpsId>(0, None),
                author: ALICE,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA_SECONDARY),
            })
        );

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0,
                Some(multi_account_id::<Runtime, IpsId>(1, None))
            )),
            0,
            vec![AnyId::IpsId(1)],
            None
        ));

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IpsId>(0, None),
                supply: 1000 + 2 * ExistentialDeposit::get(),
            })
        );

        assert_eq!(
            IptStorage::<Runtime>::get(1),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IpsId>(1, None),
                supply: 0,
            })
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                allow_replica: true,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0), AnyId::IpfId(1), AnyId::IpsId(1)]
                    .try_into()
                    .unwrap(),
                ips_type: IpsType::Normal,
            })
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(1),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Child(0, multi_account_id::<Runtime, IpsId>(0, None)),
                allow_replica: false,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Replica(0)
            })
        );
    })
}

#[test]
fn append_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY),
        ));

        assert_ok!(Ipf::mint(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA_SECONDARY),
        ));

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            false,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA_SECONDARY.to_vec(),
            vec![2],
            false,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        // Case 0: Alice tries to append an IPF to an IPS in a non-multisig context
        assert_noop!(
            Ips::append(Origin::signed(ALICE), 0, vec![AnyId::IpfId(1)], None),
            Error::<Runtime>::NoPermission
        );

        // Case 1: Multisig context, but didn't include caller
        assert_noop!(
            Ips::append(
                Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
                0,
                vec![AnyId::IpfId(1)],
                None,
            ),
            Error::<Runtime>::NoPermission
        );

        // Case 2: Multisig context, but wrong IPF
        assert_noop!(
            Ips::append(
                Origin::signed(multi_account_id::<Runtime, IpsId>(0, Some(ALICE))),
                0,
                vec![AnyId::IpfId(2)],
                None,
            ),
            Error::<Runtime>::NoPermission
        );

        // Case 3: Unknown origin
        assert_noop!(
            Ips::append(Origin::none(), 0, vec![AnyId::IpfId(1)], None),
            DispatchError::BadOrigin
        );

        // Case 4: Alice tries to append an IPS to another IPS in a non-multisig context
        assert_noop!(
            Ips::append(Origin::signed(ALICE), 0, vec![AnyId::IpsId(1)], None),
            Error::<Runtime>::NoPermission
        );

        // Case 5: An IPS account tries to append a different IPS to the first one
        assert_noop!(
            Ips::append(
                Origin::signed(multi_account_id::<Runtime, IpsId>(
                    0,
                    Some(multi_account_id::<Runtime, IpsId>(
                        7, /*This IPS does not exist*/
                        None
                    ))
                )),
                0,
                vec![AnyId::IpsId(1)],
                None
            ),
            Error::<Runtime>::NoPermission
        );

        // Case 6: Empty Appending
        assert_noop!(
            Ips::append(
                Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
                0,
                Default::default(), // Empty vec
                None
            ),
            Error::<Runtime>::ValueNotChanged
        );
        // Issue #134: empty append successful when not the owner but empty append list
        assert_noop!(
            Ips::append(
                Origin::signed(ALICE),
                0,
                Default::default(), // Empty vec
                None
            ),
            Error::<Runtime>::ValueNotChanged
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                allow_replica: false,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Normal,
            })
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(1),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(1, None)),
                allow_replica: false,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(2)].try_into().unwrap(),
                ips_type: IpsType::Normal,
            })
        )
    });
}

#[test]
fn remove_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY),
        ));

        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::create_replica(
            Origin::signed(ALICE),
            0,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, Some(ALICE))),
            0,
            vec![AnyId::IpfId(1)],
            None
        ));

        assert_eq!(
            IpfStorage::<Runtime>::get(1),
            Some(IpfInfoOf::<Runtime> {
                owner: multi_account_id::<Runtime, IpsId>(0, None),
                author: ALICE,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA_SECONDARY),
            })
        );

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0,
                Some(multi_account_id::<Runtime, IpsId>(1, None))
            )),
            0,
            vec![AnyId::IpsId(1)],
            None
        ));

        assert_ok!(Ips::remove(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
            0,
            vec![(AnyId::IpsId(1), ALICE)],
            None
        ));

        assert_ok!(Ips::remove(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
            0,
            vec![(AnyId::IpfId(1), ALICE)],
            None,
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                allow_replica: true,
                ips_type: IpsType::Normal,
            })
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IpsId>(0, None),
                supply: ExistentialDeposit::get() + ExistentialDeposit::get(), // We appended a child IPS to this one and then removed, that process doe not burn the migrated IPTs
            })
        );

        assert_eq!(
            IptStorage::<Runtime>::get(1),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IpsId>(1, None),
                supply: ExistentialDeposit::get(),
            })
        )
    });
}

#[test]
fn remove_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_ok!(Ipf::mint(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY),
        ));

        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            true,
            None,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::create_replica(
            Origin::signed(ALICE),
            0,
            GPLv3,
            percent!(50),
            One,
            false
        ));

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(0, Some(ALICE))),
            0,
            vec![AnyId::IpfId(1)],
            None
        ));
        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0,
                Some(multi_account_id::<Runtime, IpsId>(1, None))
            )),
            0,
            vec![AnyId::IpsId(1)],
            None
        ));

        // Case 1: Unknown origin
        assert_noop!(
            Ips::remove(Origin::none(), 0, vec![(AnyId::IpfId(1), ALICE)], None),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Ips::remove(Origin::none(), 0, vec![(AnyId::IpsId(1), BOB)], None),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Ips::remove(
                Origin::none(),
                0,
                vec![(AnyId::IpfId(1), ALICE), (AnyId::IpsId(1), BOB)],
                None
            ),
            DispatchError::BadOrigin
        );

        // Case 2: Non-multisig operation
        assert_noop!(
            Ips::remove(
                Origin::signed(ALICE),
                0,
                vec![(AnyId::IpfId(1), ALICE)],
                None
            ),
            Error::<Runtime>::NoPermission
        );
        assert_noop!(
            Ips::remove(Origin::signed(BOB), 0, vec![(AnyId::IpfId(1), BOB)], None),
            Error::<Runtime>::NoPermission
        );

        // Case 3: Asset does not exist
        assert_noop!(
            Ips::remove(
                Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
                0,
                vec![(AnyId::IpfId(32767), ALICE)],
                None
            ),
            Error::<Runtime>::NoPermission
        );
        assert_noop!(
            Ips::remove(
                Origin::signed(multi_account_id::<Runtime, IpsId>(0, None)),
                0,
                vec![(AnyId::IpsId(65535), ALICE)],
                None
            ),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(0, None)),
                allow_replica: true,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0), AnyId::IpfId(1), AnyId::IpsId(1)]
                    .try_into()
                    .unwrap(),
                ips_type: IpsType::Normal,
            })
        );
        assert_eq!(
            IpsStorage::<Runtime>::get(1),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Child(0, multi_account_id::<Runtime, IpsId>(0, None)),
                allow_replica: false,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Replica(0),
            })
        );

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IpsId>(0, None),
                supply: 2 * ExistentialDeposit::get(),
            })
        );

        assert_eq!(
            IptStorage::<Runtime>::get(1),
            Some(IptInfo {
                owner: multi_account_id::<Runtime, IpsId>(1, None),
                supply: 0,
            })
        );
    });
}
