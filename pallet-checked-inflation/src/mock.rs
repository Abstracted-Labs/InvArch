use core::convert::TryFrom;
use frame_support::traits::{
    ConstU128, ConstU32, ConstU64, Currency, Hooks, Imbalance, OnUnbalanced,
};
use frame_system::limits::BlockWeightsBuilder;
use pallet_balances::AccountData;
use sp_core::H256;
use sp_runtime::{parameter_types, testing::Header, traits::IdentityLookup, Perbill};

use super::*;
use crate::inflation::InflationMethod;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

type AccountId = u32;
type BlockNumber = u64;
type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

const EXISTENTIAL_DEPOSIT: Balance = 1_000_000_000;

const INFLATION_RECEIVER: AccountId = 0;
const ALICE: AccountId = 1;
const BOB: AccountId = 2;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>, Config<T>},
        CheckedInflation: pallet::{Pallet, Call, Storage, Event<T>},
    }
);

impl frame_system::Config for Test {
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
    type BlockHashCount = ConstU64<250>;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const Inflation: InflationMethod<BalanceOf<Test>> = InflationMethod::Rate(Perbill::from_percent(50));
}

pub struct DealWithInflation;
impl OnUnbalanced<NegativeImbalance> for DealWithInflation {
    fn on_unbalanced(amount: NegativeImbalance) {
        Balances::resolve_creating(&INFLATION_RECEIVER, amount)
    }
}

impl pallet::Config for Test {
    type BlocksPerEra = ConstU64<7200>;
    type Currency = Balances;
    type Event = Event;
    type ErasPerYear = ConstU32<365>;
    type Inflation = Inflation;
    type DealWithInflation = DealWithInflation;
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        CheckedInflation::on_initialize(System::block_number());
    }
}

pub fn run_to_next_era() {
    run_to_block(CheckedInflation::next_era_starting_block())
}

pub fn run_to_next_year() {
    run_to_next_era();

    let blocks_per_era = 7200u32;
    let eras_per_year = 365u32;
    let current_era = CheckedInflation::current_era();

    run_to_block(System::block_number() + ((eras_per_year - current_era) * blocks_per_era) as u64);
}
