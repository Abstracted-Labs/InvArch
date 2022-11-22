use crate::{
    Balance, Balances, BlockNumber, CommonId, Event, ExistentialDeposit, Runtime, DAYS, UNIT,
};
use frame_support::{parameter_types, PalletId};

parameter_types! {
    pub const BlocksPerEra: BlockNumber = DAYS;
    pub const RegisterDeposit: Balance = 1000 * UNIT;
    pub const MaxStakersPerIp: u32 = 1000;
    pub const MinimumStakingAmount: Balance = 10 * UNIT;
    pub const MaxEraStakeValues: u32 = 5;
    pub const MaxUnlockingChunks: u32 = 5;
    pub const UnbondingPeriod: u32 = 7;
    pub const IpStakingPot: PalletId = PalletId(*b"tkr/ipst");
    pub const PercentForIp: u32 = 60;
    pub const StakeThresholdForActiveIp: Balance = 5000 * UNIT;
    pub const MaxNameLength: u32 = 20;
    pub const MaxDescriptionLength: u32 = 100;
}

impl pallet_ip_staking::Config for Runtime {
    type Currency = Balances;
    type BlocksPerEra = BlocksPerEra;
    type IpId = CommonId;
    type RegisterDeposit = RegisterDeposit;
    type Event = Event;
    type MaxStakersPerIp = MaxStakersPerIp;
    type ExistentialDeposit = ExistentialDeposit;
    type PotId = IpStakingPot;
    type MaxUnlocking = MaxUnlockingChunks;
    type UnbondingPeriod = UnbondingPeriod;
    type MinimumStakingAmount = MinimumStakingAmount;
    type MaxEraStakeValues = MaxEraStakeValues;
    type PercentForIp = PercentForIp;
    type StakeThresholdForActiveIp = StakeThresholdForActiveIp;
    type MaxNameLength = MaxNameLength;
    type MaxDescriptionLength = MaxDescriptionLength;
}
