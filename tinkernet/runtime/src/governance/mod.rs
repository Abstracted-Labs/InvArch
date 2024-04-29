use self::councils::TinkerCouncil;

use super::*;
// use crate::xcm_config::CollectivesLocation;
use frame_support::{
    parameter_types,
    traits::{EitherOf, EitherOfDiverse},
};

use frame_system::EnsureRootWithSuccess;
use polkadot_runtime_common::prod_or_fast;

mod origins;
pub use origins::{
    pallet_custom_origins, CouncilAdmin, GeneralManagement, ReferendumCanceller, ReferendumKiller,
    Spender, WhitelistedCaller,
};
mod tracks;
pub use tracks::TracksInfo;

mod councils;

parameter_types! {
    pub const VoteLockingPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1);
}

impl pallet_conviction_voting::Config for Runtime {
    type WeightInfo = pallet_conviction_voting::weights::SubstrateWeight<Runtime>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type VoteLockingPeriod = VoteLockingPeriod;
    type MaxVotes = ConstU32<512>;
    type MaxTurnout =
        frame_support::traits::tokens::currency::ActiveIssuanceOf<Balances, Self::AccountId>;
    type Polls = Referenda;
}

parameter_types! {
    pub const AlarmInterval: BlockNumber = 1;
    pub const SubmissionDeposit: Balance = 1 * UNIT;
    pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
}

parameter_types! {
    pub const MaxBalance: Balance = Balance::max_value();
}

pub type TreasurySpender = EitherOf<EnsureRootWithSuccess<AccountId, MaxBalance>, Spender>;
pub type RootOrGeneralManagement = EitherOf<EnsureRoot<AccountId>, GeneralManagement>;

pub type AllCouncil = pallet_collective::EnsureProportionAtLeast<AccountId, TinkerCouncil, 1, 1>;

pub type CouncilApproveOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, TinkerCouncil, 2, 3>,
>;

pub type CouncilRejectOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<AccountId, TinkerCouncil, 1, 2>,
>;

impl pallet_custom_origins::Config for Runtime {}

impl pallet_whitelist::Config for Runtime {
    type WeightInfo = pallet_whitelist::weights::SubstrateWeight<Runtime>;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type WhitelistOrigin = CouncilApproveOrigin;
    type DispatchWhitelistedOrigin = EitherOf<EnsureRoot<Self::AccountId>, WhitelistedCaller>;
    type Preimages = Preimage;
}

impl pallet_referenda::Config for Runtime {
    type WeightInfo = pallet_referenda::weights::SubstrateWeight<Runtime>;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type Scheduler = Scheduler;
    type Currency = Balances;
    type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
    type CancelOrigin = EitherOf<EitherOf<EnsureRoot<AccountId>, ReferendumCanceller>, AllCouncil>;
    type KillOrigin = EitherOf<EitherOf<EnsureRoot<AccountId>, ReferendumKiller>, AllCouncil>;
    type Slash = Treasury;
    type Votes = pallet_conviction_voting::VotesOf<Runtime>;
    type Tally = pallet_conviction_voting::TallyOf<Runtime>;
    type SubmissionDeposit = SubmissionDeposit;
    type MaxQueued = ConstU32<100>;
    type UndecidingTimeout = UndecidingTimeout;
    type AlarmInterval = AlarmInterval;
    type Tracks = TracksInfo;
    type Preimages = Preimage;
}
