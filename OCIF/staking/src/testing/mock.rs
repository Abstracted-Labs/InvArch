use crate as pallet_ocif_staking;
use core::convert::{TryFrom, TryInto};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU128, ConstU32, Currency, OnFinalize, OnInitialize},
    weights::Weight,
    PalletId,
};
use pallet_inv4::util::derive_ips_account;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;
pub(crate) type EraIndex = u32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) const EXISTENTIAL_DEPOSIT: Balance = 2;
pub(crate) const MAX_NUMBER_OF_STAKERS: u32 = 4;
pub(crate) const MINIMUM_STAKING_AMOUNT: Balance = 10;
pub(crate) const MAX_UNLOCKING: u32 = 4;
pub(crate) const UNBONDING_PERIOD: EraIndex = 3;
pub(crate) const MAX_ERA_STAKE_VALUES: u32 = 8;
pub(crate) const BLOCKS_PER_ERA: BlockNumber = 3;
pub(crate) const REGISTER_DEPOSIT: Balance = 10;

construct_runtime!(
    pub struct Test
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        OcifStaking: pallet_ocif_staking,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(Weight::from_ref_time(1024));
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Index = u64;
    type Call = Call;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MaxLocks: u32 = 4;
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 3;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const RegisterDeposit: Balance = REGISTER_DEPOSIT;
    pub const BlockPerEra: BlockNumber = BLOCKS_PER_ERA;
    pub const MaxStakersPerCore: u32 = MAX_NUMBER_OF_STAKERS;
    pub const MinimumStakingAmount: Balance = MINIMUM_STAKING_AMOUNT;
    pub const PotId: PalletId = PalletId(*b"ocif-pot");
    pub const MaxUnlocking: u32 = MAX_UNLOCKING;
    pub const UnbondingPeriod: EraIndex = UNBONDING_PERIOD;
    pub const MaxEraStakeValues: u32 = MAX_ERA_STAKE_VALUES;
    pub const RewardRatio: (u32, u32) = (50, 50);
}

pub type CoreId = u32;

pub const THRESHOLD: u128 = 50;

impl pallet_ocif_staking::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type BlocksPerEra = BlockPerEra;
    type RegisterDeposit = RegisterDeposit;
    type CoreId = CoreId;
    type MaxStakersPerCore = MaxStakersPerCore;
    type MinimumStakingAmount = MinimumStakingAmount;
    type PotId = PotId;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxUnlocking = MaxUnlocking;
    type UnbondingPeriod = UnbondingPeriod;
    type MaxEraStakeValues = MaxEraStakeValues;
    type MaxDescriptionLength = ConstU32<300>;
    type MaxNameLength = ConstU32<20>;
    type MaxImageUrlLength = ConstU32<60>;
    type RewardRatio = RewardRatio;
    type StakeThresholdForActiveCore = ConstU128<THRESHOLD>;
}

pub struct ExternalityBuilder;

pub fn account(core: CoreId) -> AccountId {
    derive_ips_account::<Test, CoreId, AccountId>(core, None)
}

pub const A: CoreId = 0;
pub const B: CoreId = 1;
pub const C: CoreId = 2;
pub const D: CoreId = 3;
pub const E: CoreId = 4;
pub const F: CoreId = 5;
pub const G: CoreId = 6;
pub const H: CoreId = 7;
pub const I: CoreId = 8;
pub const J: CoreId = 9;
pub const K: CoreId = 10;
pub const L: CoreId = 11;
pub const M: CoreId = 12;
pub const N: CoreId = 13;

impl ExternalityBuilder {
    pub fn build() -> TestExternalities {
        let storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);

        ext.execute_with(|| {
            Balances::resolve_creating(&account(A), Balances::issue(9000));
            Balances::resolve_creating(&account(B), Balances::issue(800));
            Balances::resolve_creating(&account(C), Balances::issue(10000));
            Balances::resolve_creating(&account(D), Balances::issue(4900));
            Balances::resolve_creating(&account(E), Balances::issue(3800));
            Balances::resolve_creating(&account(F), Balances::issue(10));
            Balances::resolve_creating(&account(G), Balances::issue(1000));
            Balances::resolve_creating(&account(H), Balances::issue(2000));
            Balances::resolve_creating(&account(I), Balances::issue(10000));
            Balances::resolve_creating(&account(J), Balances::issue(300));
            Balances::resolve_creating(&account(K), Balances::issue(400));
            Balances::resolve_creating(&account(L), Balances::issue(10));
            Balances::resolve_creating(&account(M), Balances::issue(EXISTENTIAL_DEPOSIT));
            Balances::resolve_creating(&account(N), Balances::issue(1_000_000_000_000));
        });

        ext.execute_with(|| System::set_block_number(1));

        ext
    }
}

pub const ISSUE_PER_BLOCK: Balance = 1000000;

pub const ISSUE_PER_ERA: Balance = ISSUE_PER_BLOCK * BLOCKS_PER_ERA as u128;

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        OcifStaking::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);

        OcifStaking::rewards(Balances::issue(ISSUE_PER_BLOCK));

        OcifStaking::on_initialize(System::block_number());
    }
}

pub fn run_to_block_no_rewards(n: u64) {
    while System::block_number() < n {
        OcifStaking::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        OcifStaking::on_initialize(System::block_number());
    }
}

pub fn issue_rewards(amount: Balance) {
    OcifStaking::rewards(Balances::issue(amount));
}

pub fn run_for_blocks(n: u64) {
    run_to_block(System::block_number() + n);
}

pub fn run_for_blocks_no_rewards(n: u64) {
    run_to_block_no_rewards(System::block_number() + n);
}

pub fn advance_to_era(n: EraIndex) {
    while OcifStaking::current_era() < n {
        run_for_blocks(1);
    }
}

pub fn advance_to_era_no_rewards(n: EraIndex) {
    while OcifStaking::current_era() < n {
        run_for_blocks_no_rewards(1);
    }
}

pub fn initialize_first_block() {
    assert_eq!(System::block_number(), 1 as BlockNumber);

    OcifStaking::on_initialize(System::block_number());
    run_to_block(2);
}

pub fn split_reward_amount(amount: Balance) -> (Balance, Balance) {
    let percent = Perbill::from_percent(RewardRatio::get().0);

    let amount_for_core = percent * amount;

    (amount_for_core, amount - amount_for_core)
}
