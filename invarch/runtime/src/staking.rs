use crate::{
    Balance, Balances, BlockNumber, ExistentialDeposit, Runtime, RuntimeEvent, DAYS, UNIT,
};
use frame_support::{parameter_types, PalletId};

parameter_types! {
    pub const BlocksPerEra: BlockNumber = DAYS;
    pub const RegisterDeposit: Balance = 5000 * UNIT;
    pub const MaxStakersPerCore: u32 = 10000;
    // Temporarily dropping down from 50 to 5.
    pub const MinimumStakingAmount: Balance = 5 * UNIT;
    pub const MaxEraStakeValues: u32 = 5;
    pub const MaxUnlockingChunks: u32 = 5;
    pub const UnbondingPeriod: u32 = 28;
    pub const OcifStakingPot: PalletId = PalletId(*b"inv/stak");
    pub const RewardRatio: (u32, u32) = (60, 40);
    pub const StakeThresholdForActiveCore: Balance = 250_000 * UNIT;
    pub const MaxNameLength: u32 = 20;
    pub const MaxDescriptionLength: u32 = 300;
    pub const MaxImageUrlLength: u32 = 100;
}

impl pallet_ocif_staking::Config for Runtime {
    type Currency = Balances;
    type BlocksPerEra = BlocksPerEra;
    type RegisterDeposit = RegisterDeposit;
    type RuntimeEvent = RuntimeEvent;
    type MaxStakersPerCore = MaxStakersPerCore;
    type ExistentialDeposit = ExistentialDeposit;
    type PotId = OcifStakingPot;
    type MaxUnlocking = MaxUnlockingChunks;
    type UnbondingPeriod = UnbondingPeriod;
    type MinimumStakingAmount = MinimumStakingAmount;
    type MaxEraStakeValues = MaxEraStakeValues;
    type RewardRatio = RewardRatio;
    type StakeThresholdForActiveCore = StakeThresholdForActiveCore;
    type MaxNameLength = MaxNameLength;
    type MaxDescriptionLength = MaxDescriptionLength;
    type MaxImageUrlLength = MaxImageUrlLength;

    type WeightInfo = pallet_ocif_staking::weights::SubstrateWeight<Runtime>;
}
