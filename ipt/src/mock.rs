//! Mocks for the gradually-update module.

#![cfg(test)]

use frame_support::{construct_runtime, parameter_types};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

use super::*;

use crate as ipt;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

pub type AccountId = u128;
pub type BlockNumber = u64;

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_types! {
    pub const MaxIpsMetadata: u32 = 1;
    pub const MaxIptMetadata: u32 = 1;
}

impl Config for Runtime {
    type IpsId = u64;
    type IptId = u64;
    type IpsData = ();
    type IptData = ();
    type MaxIpsMetadata = MaxIpsMetadata;
    type MaxIptMetadata = MaxIptMetadata;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Ipt: ipt::{Pallet, Storage, Config<T>},
    }
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const IPS_ID: <Runtime as Config>::IpsId = 0;
pub const IPT_ID: <Runtime as Config>::IptId = 0;
pub const IPT_ID_NOT_EXIST: <Runtime as Config>::IptId = 100;

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
