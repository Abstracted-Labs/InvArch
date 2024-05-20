//! Councils for Governance

use frame_support::traits::ChangeMembers;
use orml_traits::LockIdentifier;
use sp_staking::currency_to_vote::U128CurrencyToVote;

use super::*;

pub type TinkerCouncil = pallet_collective::Instance1;

parameter_types! {
    pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
    pub MaxMotionDuration: u32 =  3 * DAYS ;
    pub MaxProposals: u32 = 20;
    pub MaxMembers: u32 = 7;

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

parameter_types! {
    // Bond for candidacy into governance
    pub const CandidacyBond: Balance = 5 * UNIT;
    // 1 storage item created, key size is 32 bytes, value size is 16+16.
    pub const VotingBondBase: Balance = CENTS;
    // additional data per vote is 32 bytes (account id).
    pub const VotingBondFactor: Balance = CENTS;
    pub const TermDuration: BlockNumber = 7 * DAYS;
    pub const DesiredMembers: u32 = 2;
    pub const DesiredRunnersUp: u32 = 8;
    pub const ElectionsPhragmenPalletId: LockIdentifier = *b"ia/elect";
    pub const MaxElectionCandidates: u32 = 100;
    pub const MaxElectionVoters: u32 = 768;
    pub const MaxVotesPerVoter: u32 = 10;
}

pub struct CouncilFunnel;
impl ChangeMembers<AccountId> for CouncilFunnel {
    fn change_members_sorted(
        incoming: &[AccountId],
        outgoing: &[AccountId],
        _sorted_new: &[AccountId],
    ) {
        let mut council_members: Vec<AccountId> = Council::members();
        council_members.retain(|member| !outgoing.contains(member));
        council_members.extend_from_slice(incoming);
        council_members.sort();
        Council::change_members_sorted(incoming, outgoing, &council_members);
    }
}

impl pallet_elections_phragmen::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ElectionsPhragmenPalletId;
    type Currency = Balances;
    type ChangeMembers = CouncilFunnel;
    type InitializeMembers = ();
    type CurrencyToVote = U128CurrencyToVote;
    type CandidacyBond = CandidacyBond;
    type VotingBondBase = VotingBondBase;
    type VotingBondFactor = VotingBondFactor;
    type LoserCandidate = Treasury;
    type KickedMember = Treasury;
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    #[cfg(feature = "on-chain-release-build")]
    type TermDuration = TermDuration;
    #[cfg(not(feature = "on-chain-release-build"))]
    type TermDuration = ConstU32<{ 7 * MINUTES }>;
    type MaxCandidates = MaxElectionCandidates;
    type MaxVoters = MaxElectionVoters;
    type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
    type MaxVotesPerVoter = MaxVotesPerVoter;
}
