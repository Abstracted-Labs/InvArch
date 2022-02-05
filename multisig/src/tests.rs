// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Tests for Multisig Pallet

#![cfg(test)]

use super::*;

use crate as pallet_multisig;
use frame_support::{
    assert_noop, assert_ok, parameter_types,
    traits::{Contains, GenesisBuild},
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type OpaqueCall = super::OpaqueCall<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Config<T>, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
    type BaseCallFilter = TestBaseCallFilter;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Call = Call;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}
parameter_types! {
    pub const DepositBase: u64 = 1;
    pub const DepositFactor: u64 = 1;
    pub const MaxSignatories: u16 = 3;
}
pub struct TestBaseCallFilter;
impl Contains<Call> for TestBaseCallFilter {
    fn contains(c: &Call) -> bool {
        match *c {
            Call::Balances(_) => true,
            // Needed for benchmarking
            Call::System(frame_system::Call::remark { .. }) => true,
            _ => false,
        }
    }
}

parameter_types! {
      pub const AssetsStringLimit: u32 = 50;
}

impl pallet_assets::Config for Test {
    type Event = Event;
    type Balance = u64;
    type AssetId = u64;
    type Currency = Balances;
    type ForceOrigin = system::EnsureRoot<u64>;
    type AssetDeposit = ExistentialDeposit;
    type MetadataDepositBase = ExistentialDeposit;
    type MetadataDepositPerByte = ExistentialDeposit;
    type ApprovalDeposit = ExistentialDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Test>;
}

impl Config for Test {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = ();
}

use pallet_balances::{Call as BalancesCall, Error as BalancesError};

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1000), (2, 1000), (3, 1000), (4, 1000), (5, 200)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_assets::GenesisConfig::<Test> {
        assets: vec![
            (
                1,
                Multisig::multi_account_id(1),
                true,
                ExistentialDeposit::get(),
            ),
            (
                2,
                Multisig::multi_account_id(2),
                true,
                ExistentialDeposit::get(),
            ),
            (
                3,
                Multisig::multi_account_id(3),
                true,
                ExistentialDeposit::get(),
            ),
        ],
        metadata: vec![],
        accounts: vec![
            (1, 1, ExistentialDeposit::get()),
            (2, 1, ExistentialDeposit::get()),
            (2, 2, ExistentialDeposit::get()),
            (2, 3, ExistentialDeposit::get()),
            (2, 4, ExistentialDeposit::get()),
            (3, 1, ExistentialDeposit::get()),
            (3, 2, ExistentialDeposit::get()),
            (3, 3, ExistentialDeposit::get() * 2),
            (3, 4, ExistentialDeposit::get() * 2),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn now() -> Timepoint<u64> {
    Multisig::timepoint()
}

fn call_transfer(dest: u64, value: u64) -> Call {
    Call::Balances(BalancesCall::transfer { dest, value })
}

#[test]
fn multisig_deposit_is_taken_and_returned() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 500));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 500));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 500));

        let call = call_transfer(6, 15);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            2,
            None,
            OpaqueCall::from_encoded(data.clone()),
            false,
            0
        ));
        assert_eq!(Balances::free_balance(1), 449);
        assert_eq!(Balances::reserved_balance(1), 51);

        assert_ok!(Multisig::as_multi(
            Origin::signed(2),
            2,
            Some(now()),
            OpaqueCall::from_encoded(data),
            false,
            call_weight
        ));
        assert_eq!(Balances::free_balance(1), 500);
        assert_eq!(Balances::reserved_balance(1), 0);
    });
}

#[test]
fn multisig_deposit_is_taken_and_returned_with_call_storage() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 500));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 500));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 500));

        let call = call_transfer(6, 15);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        let hash = blake2_256(&data);
        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            2,
            None,
            OpaqueCall::from_encoded(data),
            true,
            0
        ));
        assert_eq!(Balances::free_balance(1), 447);
        assert_eq!(Balances::reserved_balance(1), 53);

        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(2),
            2,
            Some(now()),
            hash,
            call_weight
        ));
        assert_eq!(Balances::free_balance(1), 500);
        assert_eq!(Balances::reserved_balance(1), 0);
    });
}

#[test]
fn multisig_deposit_is_taken_and_returned_with_alt_call_storage() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(3);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 500));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 500));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 500));

        let call = call_transfer(6, 15);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        let hash = blake2_256(&data);

        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            3,
            None,
            hash.clone(),
            0
        ));
        assert_eq!(Balances::free_balance(1), 449);
        assert_eq!(Balances::reserved_balance(1), 51);

        assert_ok!(Multisig::as_multi(
            Origin::signed(2),
            3,
            Some(now()),
            OpaqueCall::from_encoded(data),
            true,
            1000000000
        ));
        assert_eq!(Balances::free_balance(2), 498);
        assert_eq!(Balances::reserved_balance(2), 2);
        assert_eq!(Balances::free_balance(1), 449);
        assert_eq!(Balances::reserved_balance(1), 51);

        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(3),
            3,
            Some(now()),
            hash,
            call_weight
        ));
        assert_eq!(Balances::free_balance(1), 500);
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::free_balance(2), 500);
        assert_eq!(Balances::reserved_balance(2), 0);
    });
}

#[test]
fn cancel_multisig_returns_deposit() {
    new_test_ext().execute_with(|| {
        let call = call_transfer(6, 15).encode();
        let hash = blake2_256(&call);
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            3,
            None,
            hash.clone(),
            0
        ));
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(2),
            3,
            Some(now()),
            hash.clone(),
            0
        ));
        assert_eq!(Balances::free_balance(1), 949);
        assert_eq!(Balances::reserved_balance(1), 51);
        assert_ok!(Multisig::cancel_as_multi(
            Origin::signed(1),
            3,
            now(),
            hash.clone()
        ),);
        assert_eq!(Balances::free_balance(1), 1000);
        assert_eq!(Balances::reserved_balance(1), 0);
    });
}

#[test]
fn timepoint_checking_works() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call = call_transfer(6, 15).encode();
        let hash = blake2_256(&call);

        assert_noop!(
            Multisig::approve_as_multi(Origin::signed(2), 2, Some(now()), hash.clone(), 0),
            Error::<Test>::UnexpectedTimepoint,
        );

        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            2,
            None,
            hash,
            0
        ));

        assert_noop!(
            Multisig::as_multi(
                Origin::signed(2),
                2,
                None,
                OpaqueCall::from_encoded(call.clone()),
                false,
                0
            ),
            Error::<Test>::NoTimepoint,
        );
        let later = Timepoint { index: 1, ..now() };
        assert_noop!(
            Multisig::as_multi(
                Origin::signed(2),
                2,
                Some(later),
                OpaqueCall::from_encoded(call),
                false,
                0
            ),
            Error::<Test>::WrongTimepoint,
        );
    });
}

#[test]
fn multisig_2_of_3_works_with_call_storing() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call = call_transfer(6, 15);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        let hash = blake2_256(&data);
        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            2,
            None,
            OpaqueCall::from_encoded(data),
            true,
            0
        ));
        assert_eq!(Balances::free_balance(6), 0);

        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(2),
            2,
            Some(now()),
            hash,
            call_weight
        ));
        assert_eq!(Balances::free_balance(6), 15);
    });
}

#[test]
fn multisig_2_of_3_works() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call = call_transfer(6, 15);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        let hash = blake2_256(&data);
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            2,
            None,
            hash,
            0
        ));
        assert_eq!(Balances::free_balance(6), 0);

        assert_ok!(Multisig::as_multi(
            Origin::signed(2),
            2,
            Some(now()),
            OpaqueCall::from_encoded(data),
            false,
            call_weight
        ));
        assert_eq!(Balances::free_balance(6), 15);
    });
}

#[test]
fn multisig_3_of_3_works() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(3);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call = call_transfer(6, 15);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        let hash = blake2_256(&data);
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            3,
            None,
            hash.clone(),
            0
        ));
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(2),
            3,
            Some(now()),
            hash.clone(),
            0
        ));
        assert_eq!(Balances::free_balance(6), 0);

        assert_ok!(Multisig::as_multi(
            Origin::signed(3),
            3,
            Some(now()),
            OpaqueCall::from_encoded(data),
            false,
            call_weight
        ));
        assert_eq!(Balances::free_balance(6), 15);
    });
}

#[test]
fn cancel_multisig_works() {
    new_test_ext().execute_with(|| {
        let call = call_transfer(6, 15).encode();
        let hash = blake2_256(&call);
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            3,
            None,
            hash.clone(),
            0
        ));
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(2),
            3,
            Some(now()),
            hash.clone(),
            0
        ));
        assert_noop!(
            Multisig::cancel_as_multi(Origin::signed(2), 3, now(), hash.clone()),
            Error::<Test>::NotOwner,
        );
        assert_ok!(Multisig::cancel_as_multi(
            Origin::signed(1),
            3,
            now(),
            hash.clone()
        ),);
    });
}

#[test]
fn cancel_multisig_with_call_storage_works() {
    new_test_ext().execute_with(|| {
        let call = call_transfer(6, 15).encode();
        let hash = blake2_256(&call);
        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            3,
            None,
            OpaqueCall::from_encoded(call),
            true,
            0
        ));
        assert_eq!(Balances::free_balance(1), 947);
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(2),
            3,
            Some(now()),
            hash.clone(),
            0
        ));
        assert_noop!(
            Multisig::cancel_as_multi(Origin::signed(2), 3, now(), hash.clone()),
            Error::<Test>::NotOwner,
        );
        assert_ok!(Multisig::cancel_as_multi(
            Origin::signed(1),
            3,
            now(),
            hash.clone()
        ),);
        assert_eq!(Balances::free_balance(1), 1000);
    });
}

#[test]
fn cancel_multisig_with_alt_call_storage_works() {
    new_test_ext().execute_with(|| {
        let call = call_transfer(6, 15).encode();
        let hash = blake2_256(&call);
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            3,
            None,
            hash.clone(),
            0
        ));
        assert_eq!(Balances::free_balance(1), 949);
        assert_ok!(Multisig::as_multi(
            Origin::signed(2),
            3,
            Some(now()),
            OpaqueCall::from_encoded(call),
            true,
            0
        ));
        assert_eq!(Balances::free_balance(2), 998);
        assert_ok!(Multisig::cancel_as_multi(Origin::signed(1), 3, now(), hash));
        assert_eq!(Balances::free_balance(1), 1000);
        assert_eq!(Balances::free_balance(2), 1000);
    });
}

#[test]
fn multisig_2_of_3_as_multi_works() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call = call_transfer(6, 15);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            2,
            None,
            OpaqueCall::from_encoded(data.clone()),
            false,
            0
        ));
        assert_eq!(Balances::free_balance(6), 0);

        assert_ok!(Multisig::as_multi(
            Origin::signed(2),
            2,
            Some(now()),
            OpaqueCall::from_encoded(data),
            false,
            call_weight
        ));
        assert_eq!(Balances::free_balance(6), 15);
    });
}

#[test]
fn multisig_2_of_3_as_multi_with_many_calls_works() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call1 = call_transfer(6, 10);
        let call1_weight = call1.get_dispatch_info().weight;
        let data1 = call1.encode();
        let call2 = call_transfer(7, 5);
        let call2_weight = call2.get_dispatch_info().weight;
        let data2 = call2.encode();

        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            2,
            None,
            OpaqueCall::from_encoded(data1.clone()),
            false,
            0
        ));
        assert_ok!(Multisig::as_multi(
            Origin::signed(2),
            2,
            None,
            OpaqueCall::from_encoded(data2.clone()),
            false,
            0
        ));
        assert_ok!(Multisig::as_multi(
            Origin::signed(3),
            2,
            Some(now()),
            OpaqueCall::from_encoded(data1),
            false,
            call1_weight
        ));
        assert_ok!(Multisig::as_multi(
            Origin::signed(3),
            2,
            Some(now()),
            OpaqueCall::from_encoded(data2),
            false,
            call2_weight
        ));

        assert_eq!(Balances::free_balance(6), 10);
        assert_eq!(Balances::free_balance(7), 5);
    });
}

#[test]
fn multisig_2_of_3_cannot_reissue_same_call() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call = call_transfer(6, 10);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        let hash = blake2_256(&data);
        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            2,
            None,
            OpaqueCall::from_encoded(data.clone()),
            false,
            0
        ));
        assert_ok!(Multisig::as_multi(
            Origin::signed(2),
            2,
            Some(now()),
            OpaqueCall::from_encoded(data.clone()),
            false,
            call_weight
        ));
        assert_eq!(Balances::free_balance(multi), 5);

        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            2,
            None,
            OpaqueCall::from_encoded(data.clone()),
            false,
            0
        ));
        assert_ok!(Multisig::as_multi(
            Origin::signed(3),
            2,
            Some(now()),
            OpaqueCall::from_encoded(data),
            false,
            call_weight
        ));

        let err = DispatchError::from(BalancesError::<Test, _>::InsufficientBalance).stripped();
        System::assert_last_event(
            pallet_multisig::Event::MultisigExecuted {
                approving: 3,
                timepoint: now(),
                multisig: multi,
                call_hash: hash,
                result: Err(err),
            }
            .into(),
        );
    });
}

#[test]
fn duplicate_approvals_are_ignored() {
    new_test_ext().execute_with(|| {
        let call = call_transfer(6, 15).encode();
        let hash = blake2_256(&call);
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            2,
            None,
            hash.clone(),
            0
        ));
        assert_noop!(
            Multisig::approve_as_multi(Origin::signed(1), 2, Some(now()), hash.clone(), 0),
            Error::<Test>::AlreadyApproved,
        );
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(2),
            2,
            Some(now()),
            hash.clone(),
            0
        ));
        assert_noop!(
            Multisig::approve_as_multi(Origin::signed(3), 2, Some(now()), hash.clone(), 0),
            Error::<Test>::AlreadyApproved,
        );
    });
}

#[test]
fn multisig_1_of_3_works() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(1);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 500));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 500));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 500));

        let boxed_call = Box::new(call_transfer(6, 150));
        assert_ok!(Multisig::as_multi_threshold_1(
            Origin::signed(1),
            1,
            boxed_call
        ));

        assert_eq!(Balances::free_balance(6), 150);
    });
}

#[test]
fn multisig_filters() {
    new_test_ext().execute_with(|| {
        let call = Box::new(Call::System(frame_system::Call::set_code { code: vec![] }));
        assert_noop!(
            Multisig::as_multi_threshold_1(Origin::signed(1), 1, call.clone()),
            DispatchError::from(frame_system::Error::<Test>::CallFiltered),
        );
    });
}

#[test]
fn weight_check_works() {
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(2);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call = call_transfer(6, 15);
        let data = call.encode();
        assert_ok!(Multisig::as_multi(
            Origin::signed(1),
            2,
            None,
            OpaqueCall::from_encoded(data.clone()),
            false,
            0
        ));
        assert_eq!(Balances::free_balance(6), 0);

        assert_noop!(
            Multisig::as_multi(
                Origin::signed(2),
                2,
                Some(now()),
                OpaqueCall::from_encoded(data),
                false,
                0
            ),
            Error::<Test>::MaxWeightTooLow,
        );
    });
}

#[test]
fn multisig_handles_no_preimage_after_all_approve() {
    // This test checks the situation where everyone approves a multi-sig, but no-one provides the
    // call data. In the end, any of the multisig callers can approve again with the call data and
    // the call will go through.
    new_test_ext().execute_with(|| {
        let multi = Multisig::multi_account_id(3);
        assert_ok!(Balances::transfer(Origin::signed(1), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(2), multi, 5));
        assert_ok!(Balances::transfer(Origin::signed(3), multi, 5));

        let call = call_transfer(6, 15);
        let call_weight = call.get_dispatch_info().weight;
        let data = call.encode();
        let hash = blake2_256(&data);
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(1),
            3,
            None,
            hash.clone(),
            0
        ));
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(2),
            3,
            Some(now()),
            hash.clone(),
            0
        ));
        assert_ok!(Multisig::approve_as_multi(
            Origin::signed(3),
            3,
            Some(now()),
            hash.clone(),
            0
        ));
        assert_eq!(Balances::free_balance(6), 0);

        assert_ok!(Multisig::as_multi(
            Origin::signed(3),
            3,
            Some(now()),
            OpaqueCall::from_encoded(data),
            false,
            call_weight
        ));
        assert_eq!(Balances::free_balance(6), 15);
    });
}
