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
use pallet_ipl::LicenseList;
use smallvec::smallvec;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

use super::*;

use crate as ipt;
use pallet_ipl as ipl;

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
    pub const MaxLicenseMetadata: u32 = 32;
}

impl pallet_ipl::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Balance = Balance;
    type IplId = u64;
    type Licenses = InvArchLicenses;
    type MaxLicenseMetadata = MaxLicenseMetadata;
}

parameter_types! {
    pub const MaxCallers: u32 = 32;
    pub const MaxIptMetadata: u32 = 32;
}

impl Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Balance = Balance;
    type IptId = u64;
    type MaxCallers = MaxCallers;
    type ExistentialDeposit = ExistentialDeposit;
    type Call = Call;
    type WeightToFeePolynomial = WeightToFee;
    type MaxSubAssets = MaxCallers;
    type MaxIptMetadata = MaxIptMetadata;
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
            _ => true,
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
        Ipt: ipt::{Pallet, Call, Storage, Event<T>},
        Ipl: ipl::{Pallet, Call, Storage, Event<T>},
    }
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const VADER: AccountId = 3;

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

pub type Hash = sp_core::H256;

#[derive(Debug, Clone, Encode, Decode, TypeInfo, Eq, PartialEq)]
pub enum InvArchLicenses {
    Apache2,
    GPLv3,
    Custom(Vec<u8>, Hash),
}

impl LicenseList for InvArchLicenses {
    type IpfsHash = Hash; // License IPFS hash.
    type LicenseMetadata = Vec<u8>; // License name.

    fn get_hash_and_metadata(&self) -> (Self::LicenseMetadata, Self::IpfsHash) {
        match self {
            InvArchLicenses::Apache2 => (
                vec![65, 112, 97, 99, 104, 97, 32, 118, 50, 46, 48],
                [
                    7, 57, 92, 251, 234, 183, 217, 144, 220, 196, 201, 132, 176, 249, 18, 224, 237,
                    201, 2, 113, 146, 78, 111, 152, 92, 71, 16, 228, 87, 39, 81, 142,
                ]
                .into(),
            ),
            InvArchLicenses::GPLv3 => (
                vec![71, 78, 85, 32, 71, 80, 76, 32, 118, 51],
                [
                    72, 7, 169, 24, 30, 7, 200, 69, 232, 27, 10, 138, 130, 253, 91, 158, 210, 95,
                    127, 37, 85, 41, 106, 136, 66, 116, 64, 35, 252, 195, 69, 253,
                ]
                .into(),
            ),
            InvArchLicenses::Custom(metadata, hash) => (metadata.clone(), *hash),
        }
    }
}
