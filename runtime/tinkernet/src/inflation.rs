use crate::{Balance, Balances, BlockNumber, Event, NegativeImbalance, OcifStaking, Runtime, DAYS};
use frame_support::{parameter_types, traits::OnUnbalanced};
use sp_runtime::Perbill;

pub const TEN_PERCENT_PER_YEAR: pallet_checked_inflation::InflationMethod<Balance> =
    pallet_checked_inflation::InflationMethod::Rate(Perbill::from_percent(10));

const _YEAR: u32 = 365;
const MONTH: u32 = 30;

parameter_types! {
    pub const BlocksPerEra: BlockNumber = DAYS;
    pub const ErasPerYear: u32 = MONTH;
    pub const Inflation: pallet_checked_inflation::InflationMethod<Balance> = TEN_PERCENT_PER_YEAR;
}

pub struct DealWithInflation;
impl OnUnbalanced<NegativeImbalance> for DealWithInflation {
    fn on_unbalanced(amount: NegativeImbalance) {
        OcifStaking::rewards(amount);
    }
}

impl pallet_checked_inflation::Config for Runtime {
    type BlocksPerEra = BlocksPerEra;
    type Currency = Balances;
    type Event = Event;
    type ErasPerYear = ErasPerYear;
    type Inflation = Inflation;
    type DealWithInflation = DealWithInflation;
}
