//! Councils for Governance

use super::*;

pub type TinkerCouncil = pallet_collective::Instance1;

parameter_types! {
    pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
    pub MaxMotionDuration: u32 =  3 * DAYS ;
    pub MaxProposals: u32 = 20;
    pub MaxMembers: u32 = 5;

}

impl pallet_collective::Config<TinkerCouncil> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type Proposal = RuntimeCall;
    /// The maximum amount of time (in blocks) council members to vote on motions.
    /// Motions may end in fewer blocks if enough votes are cast to determine the result.
    type MotionDuration = MaxMotionDuration;
    /// The maximum number of proposals that can be open in council at once.
    type MaxProposals = MaxProposals;
    /// The maximum number of council members.
    type MaxMembers = MaxMembers;
    type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    type SetMembersOrigin = EitherOf<CouncilApproveOrigin, CouncilAdmin>;
    type MaxProposalWeight = MaxProposalWeight;
}
