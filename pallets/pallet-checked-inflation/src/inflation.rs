//! Available inflation methods and resulting inflation amount generated per era.
//!
//! ## Overview
//!
//! This module contains the available inflation methods and the resulting inflation amount generated per era.

use crate::{BalanceOf, Config};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_arithmetic::per_things::Perbill;

/// Inflation methods.
///
/// The inflation methods are used to determine the amount of inflation generated per era.
#[derive(TypeInfo, Encode, Decode)]
pub enum InflationMethod<Balance> {
    /// The inflation is calculated as a percentage (`Perbill`) of the current supply.
    Rate(Perbill),
    /// The inflation is a fixed amount per year.
    FixedYearly(Balance),
    /// The inflation is a fixed amount per era.
    FixedPerEra(Balance),
}

/// Getter trait for the inflation amount to be minted in each era.
pub trait GetInflation<T: Config> {
    /// Returns the inflation amount to be minted per era.
    fn get_inflation_args(&self, eras_per_year: u32, current_supply: BalanceOf<T>) -> BalanceOf<T>;
}

impl<T: Config> GetInflation<T> for InflationMethod<BalanceOf<T>>
where
    u32: Into<BalanceOf<T>>,
{
    /// Returns the inflation amount to be minted per era based on the inflation method.
    fn get_inflation_args(&self, eras_per_year: u32, current_supply: BalanceOf<T>) -> BalanceOf<T> {
        match self {
            Self::Rate(rate) => (*rate * current_supply) / eras_per_year.into(),
            Self::FixedYearly(amount) => *amount / eras_per_year.into(),
            Self::FixedPerEra(amount) => *amount,
        }
    }
}
