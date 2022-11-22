use crate::{Balance, Balances, BlockNumber, Event, IpStaking, NegativeImbalance, Runtime, DAYS};
use frame_support::{parameter_types, traits::OnUnbalanced};

pub const TEN_PERCENT_PER_YEAR: pallet_checked_inflation::InflationMethod<Balance> =
    pallet_checked_inflation::InflationMethod::Rate(sp_runtime::Perbill::from_percent(10));

parameter_types! {
    pub const BlocksPerEra: BlockNumber = DAYS;
    pub const ErasPerYear: u32 = 365;
    pub const Inflation: pallet_checked_inflation::InflationMethod<Balance> = TEN_PERCENT_PER_YEAR;
}

pub struct DealWithInflation;
impl OnUnbalanced<NegativeImbalance> for DealWithInflation {
    fn on_unbalanced(amount: NegativeImbalance) {
        IpStaking::rewards(amount);
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
