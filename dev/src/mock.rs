//! Mocks for pallet-dev.

use frame_support::{construct_runtime, parameter_types, traits::Contains};
use pallet_balances::AccountData;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

use super::*;
use ipo;
use ips;
use ipt;

use crate as dev;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

pub type Balance = u128;
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
    type AccountData = AccountData<AccountId>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = BaseFilter;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const MaxIptMetadata: u32 = 32;
}

impl ipt::Config for Runtime {
    type IptId = u64;
    type MaxIptMetadata = MaxIptMetadata;
    type Event = Event;
}

parameter_types! {
    pub const MaxIpsMetadata: u32 = 32;
}

impl ips::Config for Runtime {
    type Event = Event;
    type IpsId = u64;
    type MaxIpsMetadata = MaxIpsMetadata;
    type Currency = Balances;
    type IpsData = Vec<<Runtime as ipt::Config>::IptId>;
}

parameter_types! {
    pub const MaxIpoMetadata: u32 = 32;
}

impl ipo::Config for Runtime {
    type IpoId = u64;
    type MaxIpoMetadata = MaxIpoMetadata;
    type Event = Event;
    type IpoData = ();
    type Currency = Balances;
    type Balance = Balance;
    type ExistentialDeposit = ExistentialDeposit;
}

parameter_types! {
    pub const MaxDevMetadata: u32 = 32;
}

impl Config for Runtime {
    type Event = Event;
    type DevId = u64;
    type DevData = Vec<u8>;
    type MaxDevMetadata = MaxDevMetadata;
    type Currency = Balances;
    type Allocation = u32;
    type Interaction = <Runtime as frame_system::Config>::Hash;
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
            _ => false,
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
        Dev: dev::{Pallet, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>, Config<T>},
        Ipt: ipt::{Pallet, Storage, Event<T>},
        Ips: ips::{Pallet, Storage, Event<T>},
        Ipo: ipo::{Pallet, Storage, Event<T>},
    }
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

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
pub const _MOCK_METADATA_SECONDARY: &'static [u8] = &[
    47, 182, 72, 140, 51, 139, 219, 171, 74, 247, 18, 123, 28, 200, 236, 221, 85, 25, 12, 218, 0,
    230, 247, 32, 73, 152, 66, 243, 27, 92, 95, 12,
];
pub const _MOCK_METADATA_PAST_MAX: &'static [u8] = &[
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
