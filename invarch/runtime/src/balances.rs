use crate::{
    AccountId, Balance, Balances, BlockNumber, CouncilRejectOrigin, ExtrinsicBaseWeight, Runtime,
    RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, System, Treasury, TreasurySpender, DAYS,
    EXISTENTIAL_DEPOSIT, MICROUNIT, MILLIUNIT, UNIT,
};
use frame_support::{
    pallet_prelude::ConstU32,
    parameter_types,
    traits::{
        fungible::{self, Balanced},
        tokens::{PayFromAccount, UnityAssetBalanceConversion},
        Currency, Imbalance, OnUnbalanced, SortedMembers,
    },
    weights::{
        ConstantMultiplier, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    PalletId,
};
use frame_system::EnsureSignedBy;
use polkadot_runtime_common::SlowAdjustingFeeUpdate;
use sp_runtime::{
    traits::{AccountIdConversion, IdentityLookup},
    Perbill, Permill,
};
use sp_std::vec::Vec;

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const MaxFreezes: u32 = 50;
    pub const MaxReserves: u32 = 50;
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
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = [u8; 8];
    type MaxFreezes = MaxFreezes;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type RuntimeHoldReason = RuntimeHoldReason;
}

parameter_types! {
    // Relay Chain `TransactionByteFee` / 10
    pub const TransactionByteFee: Balance = 10 * MICROUNIT;
    pub const OperationalFeeMultiplier: u8 = 5;
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let p = 10 * MILLIUNIT;
        let q = 2 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
        smallvec::smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}

pub type NegativeImbalance =
    fungible::Credit<<Runtime as frame_system::Config>::AccountId, Balances>;

pub struct ToCollatorPot;
impl OnUnbalanced<NegativeImbalance> for ToCollatorPot {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        let collator_pot =
            <Runtime as pallet_collator_selection::Config>::PotId::get().into_account_truncating();
        let _ = Balances::resolve(&collator_pot, amount);
    }
}
pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(mut fees) = fees_then_tips.next() {
            if let Some(tips) = fees_then_tips.next() {
                tips.merge_into(&mut fees);
            }

            let (to_treasury, to_collators) = fees.ration(50, 50);

            ToCollatorPot::on_unbalanced(to_collators);
            // Treasury is still based on the old Currency trait.
            Treasury::on_nonzero_unbalanced(
                <Balances as Currency<AccountId>>::NegativeImbalance::new(to_treasury.peek()),
            );
        }
    }
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, DealWithFees>;
    type WeightToFee = WeightToFee;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
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

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(1);
    pub const ProposalBondMinimum: Balance = 100 * UNIT;
    pub const SpendPeriod: BlockNumber = 30 * DAYS;
    pub const Burn: Permill = Permill::from_percent(1);
    pub const TreasuryPalletId: PalletId = PalletId(*b"ia/trsry");
    pub const MaxApprovals: u32 = 100;
    pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;
    pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

impl pallet_treasury::Config for Runtime {
    type AssetKind = ();
    type BalanceConverter = UnityAssetBalanceConversion;
    type Beneficiary = AccountId;
    type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type RejectOrigin = CouncilRejectOrigin;
    type RuntimeEvent = RuntimeEvent;
    type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
    type PayoutPeriod = PayoutSpendPeriod;
    type SpendPeriod = SpendPeriod;
    type Burn = ();
    type BurnDestination = ();
    type SpendFunds = ();
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
    type SpendOrigin = TreasurySpender;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}
