//! Councils for Governance

use super::*;

pub type TinkerCouncil = pallet_collective::Instance1;

parameter_types! {
    // TODO: Check value of this parameter
    pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<TinkerCouncil> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type Proposal = RuntimeCall;
    /// The maximum amount of time (in blocks) council members to vote on motions.
    /// Motions may end in fewer blocks if enough votes are cast to determine the result.
    type MotionDuration = ConstU32<{ 3 * DAYS }>;
    /// The maximum number of proposals that can be open in council at once.
    type MaxProposals = ConstU32<20>;
    /// The maximum number of council members.
    type MaxMembers = ConstU32<9>;
    type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    type SetMembersOrigin = EitherOf<EnsureRoot<AccountId>, CouncilAdmin>;
    type MaxProposalWeight = MaxProposalWeight;
}
