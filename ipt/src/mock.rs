//! Mocks for the gradually-update module.

use frame_support::{construct_runtime, parameter_types, traits::Contains};
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
    type BaseCallFilter = BaseFilter;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_types! {
    pub const MaxIptMetadata: u32 = 32;
}

impl Config for Runtime {
    type IptId = u64;
    type MaxIptMetadata = MaxIptMetadata;
    type Event = Event;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

use frame_system::Call as SystemCall;
pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
    fn contains(c: &Call) -> bool {
        match *c {
            // Remark is used as a no-op call in the benchmarking
            Call::System(SystemCall::remark(_)) => true,
            Call::System(_) => false,
        }
    }
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Ipt: ipt::{Pallet, Storage, Event<T>},
    }
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const IPT_ID: <Runtime as Config>::IptId = 0;
pub const IPT_ID_DOESNT_EXIST: <Runtime as Config>::IptId = 100;
pub const MOCK_DATA: [u8; 32] = [
    12, 47, 182, 72, 140, 51, 139, 219, 171, 74, 247, 18, 123, 28, 200, 236, 221, 85, 25, 12, 218,
    0, 230, 247, 32, 73, 152, 66, 243, 27, 92, 95,
];
pub const MOCK_METADATA: &'static [u8] = &[
    12, 47, 182, 72, 140, 51, 139, 219, 171, 74, 247, 18, 123, 28, 200, 236, 221, 85, 25, 12, 218,
    0, 230, 247, 32, 73, 152, 66, 243, 27, 92, 95,
];
pub const MOCK_DATA_SECONDARY: [u8; 32] = [
    47, 182, 72, 140, 51, 139, 219, 171, 74, 247, 18, 123, 28, 200, 236, 221, 85, 25, 12, 218, 0,
    230, 247, 32, 73, 152, 66, 243, 27, 92, 95, 12,
];
pub const MOCK_METADATA_SECONDARY: &'static [u8] = &[
    47, 182, 72, 140, 51, 139, 219, 171, 74, 247, 18, 123, 28, 200, 236, 221, 85, 25, 12, 218, 0,
    230, 247, 32, 73, 152, 66, 243, 27, 92, 95, 12,
];
pub const MOCK_METADATA_PAST_MAX: &'static [u8] = &[
    12, 47, 182, 72, 140, 51, 139, 219, 171, 74, 247, 18, 123, 28, 200, 236, 221, 85, 25, 12, 218,
    0, 230, 247, 32, 73, 152, 66, 243, 27, 92, 95, 42,
];

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
