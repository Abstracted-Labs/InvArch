use self::councils::Council;

use super::*;
// use crate::xcm_config::CollectivesLocation;
use frame_support::{parameter_types, traits::EitherOf};

use frame_system::EnsureRootWithSuccess;

mod origins;
pub use origins::{
    pallet_custom_origins, GeneralManagement, ReferendumCanceller, ReferendumKiller, Spender,
    WhitelistedCaller,
};
mod tracks;
pub use tracks::TracksInfo;

mod councils;

parameter_types! {
    pub const VoteLockingPeriod: BlockNumber = 7 * DAYS;
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
    pub const SubmissionDeposit: Balance = UNIT;
    pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
}

parameter_types! {
    pub const MaxBalance: Balance = Balance::max_value();
}

pub type AllCouncil = pallet_collective::EnsureProportionAtLeast<AccountId, Council, 1, 1>;
pub type CouncilMoreThanApprove =
    pallet_collective::EnsureProportionMoreThan<AccountId, Council, 3, 5>;
pub type ConcilHalf = pallet_collective::EnsureProportionAtLeast<AccountId, Council, 1, 2>;
pub type CouncilThreeFifths = pallet_collective::EnsureProportionAtLeast<AccountId, Council, 3, 5>;

pub type TreasurySpender = EitherOf<EnsureRootWithSuccess<AccountId, MaxBalance>, Spender>;
pub type RootOrGeneralManagement = EitherOf<EnsureRoot<AccountId>, GeneralManagement>;
pub type CouncilApproveOrigin = EitherOf<EnsureRoot<AccountId>, CouncilThreeFifths>;
pub type CouncilRejectOrigin = EitherOf<EnsureRoot<AccountId>, ConcilHalf>;

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
    type CancelOrigin =
        EitherOf<EitherOf<EnsureRoot<AccountId>, ReferendumCanceller>, CouncilMoreThanApprove>;
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
