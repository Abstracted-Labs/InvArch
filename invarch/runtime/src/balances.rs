use crate::{
    AccountId, Balance, Balances, Runtime, RuntimeEvent, System, EXISTENTIAL_DEPOSIT, UNIT,
};
use frame_support::{pallet_prelude::ConstU32, parameter_types, traits::SortedMembers};
use frame_system::EnsureSignedBy;
use sp_std::vec::Vec;

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];

    type MaxHolds = ConstU32<1>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type HoldIdentifier = [u8; 8];
}

parameter_types! {
    pub const MinVestedTransfer: Balance = UNIT;
    pub const MaxVestingSchedules: u32 = 50u32;
}

pub struct InvArchAccounts;
impl SortedMembers<AccountId> for InvArchAccounts {
    fn sorted_members() -> Vec<AccountId> {
        [
            // InvArch/Tinkernet Root Account (i53Pqi67ocj66W81cJNrUvjjoM3RcAsGhXVTzREs5BRfwLnd7)
            hex_literal::hex!["f430c3461d19cded0bb3195af29d2b0379a96836c714ceb8e64d3f10902cec55"]
                .into(),
            // InvArch/Tinkernet Rewards Account (i4zTcKHr38MbSUrhFLVKHG5iULhYttBVrqVon2rv6iWcxQwQQ)
            hex_literal::hex!["725bf57f1243bf4b06e911a79eb954d1fe1003f697ef5db9640e64d6e30f9a42"]
                .into(),
        ]
        .to_vec()
    }
}

pub type EnsureInvArchAccount = EnsureSignedBy<InvArchAccounts, AccountId>;

impl orml_vesting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MinVestedTransfer = MinVestedTransfer;
    type VestedTransferOrigin = EnsureInvArchAccount;
    type WeightInfo = ();
    type MaxVestingSchedules = MaxVestingSchedules;
    // Relay chain block number provider (6 seconds)
    type BlockNumberProvider = cumulus_pallet_parachain_system::RelaychainDataProvider<Runtime>;
}
