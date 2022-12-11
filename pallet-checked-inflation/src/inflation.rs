use crate::{BalanceOf, Config};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_arithmetic::per_things::Perbill;

#[derive(TypeInfo, Encode, Decode)]
pub enum InflationMethod<Balance> {
    Rate(Perbill),
    FixedYearly(Balance),
    FixedPerEra(Balance),
}

pub trait GetInflation<T: Config> {
    fn get_inflation_args(&self, eras_per_year: u32, current_supply: BalanceOf<T>) -> BalanceOf<T>;
}

impl<T: Config> GetInflation<T> for InflationMethod<BalanceOf<T>>
where
    u32: Into<BalanceOf<T>>,
{
    fn get_inflation_args(&self, eras_per_year: u32, current_supply: BalanceOf<T>) -> BalanceOf<T> {
        match self {
            Self::Rate(rate) => (*rate * current_supply) / eras_per_year.into(),
            Self::FixedYearly(amount) => *amount / eras_per_year.into(),
            Self::FixedPerEra(amount) => *amount,
        }
    }
}
