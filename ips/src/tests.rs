//! Unit tests for the IPS pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use primitives::{utils::multi_account_id, AnyId, IpsType, Parentage};
use sp_core::H256;
use sp_runtime::DispatchError;

pub type IpsId = <Runtime as Config>::IpsId;
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
        ));

        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            vec![2],
            false
        ));

        assert_eq!(Ips::next_ips_id(), 2);

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(0, None)
                ),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0), AnyId::IpfId(1)].try_into().unwrap(),
                allow_replica: true,
                ips_type: IpsType::Normal,
            })
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(1),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(1, None)
                ),
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(2)].try_into().unwrap(),
                allow_replica: false,
                ips_type: IpsType::Normal
            })
        );

        assert_eq!(
            ipt::Ipt::<Runtime>::get(0).unwrap().supply,
            ExistentialDeposit::get()
        );
        assert_eq!(
            ipt::Ipt::<Runtime>::get(1).unwrap().supply,
            ExistentialDeposit::get()
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
            Ips::create_ips(Origin::none(), MOCK_METADATA.to_vec(), vec![0], true),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Ips::create_ips(
                Origin::signed(BOB),
                MOCK_METADATA_PAST_MAX.to_vec(),
                vec![0],
                true,
            ),
            Error::<Runtime>::MaxMetadataExceeded,
        );
        assert_noop!(
            Ips::create_ips(Origin::signed(BOB), MOCK_METADATA.to_vec(), vec![1], true),
            Error::<Runtime>::NoPermission,
        );
        assert_noop!(
            Ips::create_ips(Origin::signed(BOB), MOCK_METADATA.to_vec(), vec![2], true),
            Error::<Runtime>::NoPermission, // BOB doesn't own that IPF because it doesn't exist, so he has no permission to use it
        );

        NextIpsId::<Runtime>::mutate(|id| *id = IpsId::max_value());
        assert_noop!(
            Ips::create_ips(Origin::signed(BOB), MOCK_METADATA.to_vec(), vec![0], true),
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
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(0, None)
                ),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                allow_replica: true,
                ips_type: IpsType::Normal
            })
        );

        assert_ok!(Ips::destroy(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0, None
            )),
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
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(0, None)
                ),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                allow_replica: true,
                ips_type: IpsType::Normal,
            })
        );

        assert_noop!(Ips::destroy(Origin::none(), 0), DispatchError::BadOrigin);
        assert_noop!(
            Ips::destroy(
                Origin::signed(multi_account_id::<Runtime, IpsId>(
                    0, None
                )),
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
                Origin::signed(multi_account_id::<Runtime, IpsId>(
                    1, None
                )),
                0
            ),
            Error::<Runtime>::NoPermission
        );

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(0, None)
                ),
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
        ));

        // Case 0: Alice replicates her own IPS
        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_replica(Origin::signed(ALICE), 0));

        // Case 1: Bob replicates Alice's IPS
        assert_eq!(Ips::next_ips_id(), 2);
        assert_ok!(Ips::create_replica(Origin::signed(BOB), 0));

        let ips_0 = IpsStorage::<Runtime>::get(0).unwrap();
        let ips_1 = IpsStorage::<Runtime>::get(1).unwrap();

        assert_eq!(
            ips_1,
            IpsInfo {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(1, None)
                ),
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
        ));

        // Case 0: An unknown origin tries to replicate a non-replicable IPS
        assert_noop!(
            Ips::create_replica(Origin::none(), 0),
            DispatchError::BadOrigin,
        );

        // Case 1: Alice didn't allow replicas and tried to replicate her own IPS
        assert_eq!(Ips::next_ips_id(), 1);
        assert_noop!(
            Ips::create_replica(Origin::signed(ALICE), 0),
            Error::<Runtime>::ReplicaNotAllowed
        );

        // Case 2: Bob tried to replicate Alice's IPS
        assert_eq!(Ips::next_ips_id(), 1);
        assert_noop!(
            Ips::create_replica(Origin::signed(BOB), 0),
            Error::<Runtime>::ReplicaNotAllowed,
        );

        // Case 3: Alice allows replica, then replicates IPS 0. Soon, Bob tries to replicate Alice's replica.
        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::allow_replica(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0, None
            )),
            0
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(0, None)
                ),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Normal,
                allow_replica: true,
            })
        );

        // Subcase 0: An unknown origin tries to replicate a replicable IPS
        assert_noop!(
            Ips::create_replica(Origin::none(), 0),
            DispatchError::BadOrigin
        );

        assert_ok!(Ips::create_replica(Origin::signed(ALICE), 0));

        assert_eq!(
            IpsStorage::<Runtime>::get(1),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(1, None)
                ),
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Replica(0),
                allow_replica: false,
            })
        );

        assert_noop!(
            Ips::create_replica(Origin::signed(BOB), 1),
            Error::<Runtime>::ReplicaNotAllowed
        );

        assert_eq!(Ips::next_ips_id(), 2);
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
        ));

        assert_ok!(Ips::allow_replica(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0, None
            )),
            0
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                allow_replica: true,
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(0, None)
                ),
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

        assert_eq!(IpsStorage::<Runtime>::get(0).unwrap().allow_replica, false);
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

        assert_eq!(Ips::next_ips_id(), 0);
        assert_ok!(Ips::create_ips(
            Origin::signed(ALICE),
            MOCK_METADATA.to_vec(),
            vec![0],
            true,
        ));

        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_replica(Origin::signed(ALICE), 0));

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0,
                Some(ALICE)
            )),
            0,
            vec![AnyId::IpfId(1)],
            None
        ));

        assert_ok!(Ips::append(
            Origin::signed(multi_account_id::<Runtime, IpsId>(
                0,
                Some(multi_account_id::<Runtime, IpsId>(
                    1, None
                ))
            )),
            0,
            vec![AnyId::IpsId(1)],
            None
        ));

        assert_eq!(
            IpsStorage::<Runtime>::get(0),
            Some(IpsInfoOf::<Runtime> {
                parentage: Parentage::Parent(
                    multi_account_id::<Runtime, IpsId>(0, None)
                ),
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
                parentage: Parentage::Parent(multi_account_id::<Runtime, IpsId>(1, None)),
                allow_replica: false,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: vec![AnyId::IpfId(0)].try_into().unwrap(),
                ips_type: IpsType::Replica(0)
            })
        )
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
        ));

        assert_eq!(Ips::next_ips_id(), 1);
        assert_ok!(Ips::create_ips(
            Origin::signed(BOB),
            MOCK_METADATA_SECONDARY.to_vec(),
            vec![2],
            false,
        ));

        // Case 0: Alice tries to append an IPF to an IPS in a non-multisig context
        assert_noop!(
            Ips::append(Origin::signed(ALICE), 0, vec![AnyId::IpfId(1)], None),
            Error::<Runtime>::NoPermission
        );

        // Case 1: Multisig context, but didn't include caller
        assert_noop!(
            Ips::append(
                Origin::signed(multi_account_id::<Runtime, IpsId>(
                    0, None
                )),
                0,
                vec![AnyId::IpfId(1)],
                None,
            ),
            Error::<Runtime>::NoPermission
        );

        // Case 2: Multisig context, but wrong IPF
        assert_noop!(
            Ips::append(
                Origin::signed(multi_account_id::<Runtime, IpsId>(
                    0,
                    Some(ALICE)
                )),
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
