//! Mocks for the gradually-update module.

use frame_support::{
    construct_runtime, parameter_types,
    traits::Contains,
    weights::{
        constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    },
};
use pallet_balances::AccountData;
use smallvec::smallvec;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

use super::*;

use crate as ips;
use ipf;

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
    type AccountData = AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = BaseFilter;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
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
    pub const MaxIpfMetadata: u32 = 32;
}

impl ipf::Config for Runtime {
    type IpfId = u64;
    type MaxIpfMetadata = MaxIpfMetadata;
    type Event = Event;
}

parameter_types! {
    pub const MaxCallers: u32 = 32;
}

impl ipt::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Balance = Balance;
    type IptId = u64;
    type MaxCallers = MaxCallers;
    type ExistentialDeposit = ExistentialDeposit;
    type Call = Call;
    type WeightToFeePolynomial = WeightToFee;
    type MaxSubAssets = MaxCallers;
    type MaxIptMetadata = MaxIpfMetadata;
}

parameter_types! {
    pub const MaxIpsMetadata: u32 = 32;
}

impl Config for Runtime {
    type Event = Event;
    type IpsId = u64;
    type MaxIpsMetadata = MaxIpsMetadata;
    type Currency = Balances;
    type IpsData = Vec<<Runtime as ipf::Config>::IpfId>;
    type ExistentialDeposit = ExistentialDeposit;
    type Balance = Balance;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

use frame_system::Call as SystemCall;
use sp_runtime::BuildStorage;
pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
    fn contains(c: &Call) -> bool {
        match *c {
            // Remark is used as a no-op call in the benchmarking
            Call::System(SystemCall::remark { .. }) => true,
            Call::System(_) => false,
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
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>, Config<T>},
        Ipf: ipf::{Pallet, Storage, Event<T>},
        Ips: ips::{Pallet, Storage, Event<T>},
        Ipt: ipt::{Pallet, Call, Storage, Event<T>},
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
    // pub fn build(self) -> sp_io::TestExternalities {
    //     let t = frame_system::GenesisConfig::default()
    //         .build_storage::<Runtime>()
    //         .unwrap();

    //     let mut ext = sp_io::TestExternalities::new(t);
    //     ext.execute_with(|| System::set_block_number(1));
    //     ext
    // }

    pub fn build(self) -> sp_io::TestExternalities {
        GenesisConfig {
            system: Default::default(),
            balances: pallet_balances::GenesisConfig::<Runtime> {
                balances: vec![(ALICE, 100000), (BOB, 100000)],
            },
        }
        .build_storage()
        .unwrap()
        .into()
    }
}

pub const MILLIUNIT: Balance = 1_000_000_000;

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
        // in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
        let p = MILLIUNIT / 10;
        let q = 100 * Balance::from(ExtrinsicBaseWeight::get());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}
