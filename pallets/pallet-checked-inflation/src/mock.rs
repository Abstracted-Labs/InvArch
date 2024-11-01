use super::*;
use crate::inflation::InflationMethod;
use core::convert::TryFrom;
use frame_support::{
    derive_impl, parameter_types,
    traits::{fungible, ConstU128, ConstU32, ConstU64, Hooks, OnUnbalanced},
};
use pallet_balances::AccountData;
use sp_core::H256;
use sp_runtime::{traits::IdentityLookup, BuildStorage, Perbill};

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

type AccountId = u32;
pub type NegativeImbalance =
    frame_support::traits::fungible::Credit<<Test as frame_system::Config>::AccountId, Balances>;

pub const EXISTENTIAL_DEPOSIT: Balance = 1_000_000_000;

pub const INFLATION_RECEIVER: AccountId = 0;
pub const ALICE: AccountId = 1;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        CheckedInflation: pallet,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type Nonce = u64;
    type Block = Block;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type PalletInfo = PalletInfo;
    type AccountData = AccountData<Balance>;
    type MaxConsumers = ConstU32<16>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const Inflation: InflationMethod<BalanceOf<Test>> = InflationMethod::Rate(Perbill::from_percent(10));
}

pub struct DealWithInflation;
impl OnUnbalanced<NegativeImbalance> for DealWithInflation {
    fn on_unbalanced(amount: NegativeImbalance) {
        <Balances as fungible::Balanced<AccountId>>::resolve(&INFLATION_RECEIVER, amount)
            .expect("should work");
    }
}

pub const BLOCKS_PER_ERA: u64 = 4;
pub const ERAS_PER_YEAR: u32 = 365;

impl pallet::Config for Test {
    type BlocksPerEra = ConstU64<BLOCKS_PER_ERA>;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type ErasPerYear = ConstU32<ERAS_PER_YEAR>;
    type Inflation = Inflation;
    type DealWithInflation = DealWithInflation;
    type WeightInfo = weights::SubstrateWeight<Test>;
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

pub const GENESIS_ISSUANCE: u128 = 11700000000000000000;

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(INFLATION_RECEIVER, GENESIS_ISSUANCE)],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(0));

        //   ext.execute_with(|| YearStartIssuance::<Test>::put(Balances::total_issuance()));

        // ext.execute_with(|| run_to_block(1));

        ext
    }
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
    // run_to_next_era();

    let current_era = CheckedInflation::current_era();

    run_to_block(System::block_number() + ((ERAS_PER_YEAR - current_era) as u64 * BLOCKS_PER_ERA));

    run_to_next_era();
}

pub fn run_to_half_year() {
    run_to_next_era();

    let current_era = CheckedInflation::current_era();

    run_to_block(
        System::block_number() + (((ERAS_PER_YEAR / 2) - current_era) as u64 * BLOCKS_PER_ERA),
    );
}
