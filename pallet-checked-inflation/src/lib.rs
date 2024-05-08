//! # Checked Inflation Pallet
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This is a supporting pallet that provides the functionality for inflation. It is used to mint new tokens at the beginning of every era.
//!
//! The amount of tokens minted is determined by the inflation method and its amount, and is configurable in the runtime,
//! see the [`inflation`] module for the methods of inflation available and how their inflation amounts are calculated.
//!
//! Most of the logic is implemented in the `on_initialize` hook, which is called at the beginning of every block.
//!
//! ## Dispatchable Functions
//!
//! - `set_first_year_supply` - For configuring the pallet, sets the token's `YearStartIssuance` to its current total issuance.
//! - `halt_unhalt_pallet` - To start or stop the inflation process.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
use sp_arithmetic::traits::Zero;
use sp_std::convert::TryInto;

mod inflation;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub(crate) mod mock;

#[cfg(test)]
mod test;

pub use inflation::*;
pub use pallet::*;

pub mod weights;

pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, LockableCurrency, OnUnbalanced, ReservableCurrency},
    };
    use frame_system::{
        ensure_root,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use num_traits::CheckedSub;

    /// The balance type of this pallet.
    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// The opaque token type for an imbalance. This is returned by unbalanced operations and must be dealt with.
    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The currency (token) used in this pallet.
        type Currency: LockableCurrency<Self::AccountId, Moment = BlockNumberFor<Self>>
            + ReservableCurrency<Self::AccountId>
            + Currency<Self::AccountId>;

        /// Number of blocks per era.
        #[pallet::constant]
        type BlocksPerEra: Get<BlockNumberFor<Self>>;

        /// Number of eras per year.
        #[pallet::constant]
        type ErasPerYear: Get<u32>;

        /// The inflation method and its amount.
        #[pallet::constant]
        type Inflation: Get<InflationMethod<BalanceOf<Self>>>;

        /// The `NegativeImbalanceOf` the currency, i.e. the amount of inflation to be applied.
        type DealWithInflation: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// The current era. Starts from 1 and is reset every year.
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Block that the next era starts at.
    #[pallet::storage]
    #[pallet::getter(fn next_era_starting_block)]
    pub type NextEraStartingBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Total token supply at the very beginning of the year before any inflation has been minted.
    #[pallet::storage]
    #[pallet::getter(fn year_start_issuance)]
    pub type YearStartIssuance<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The number of tokens minted at the beginning of every era during a year.
    #[pallet::storage]
    #[pallet::getter(fn inflation_per_era)]
    pub type YearlyInflationPerEra<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Whether the inflation process is halted.
    #[pallet::storage]
    #[pallet::getter(fn is_halted)]
    pub type Halted<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::error]
    pub enum Error<T> {
        /// The pallet is already in the state that the user is trying to change it to.
        NoHaltChange,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Beginning of a new year.
        NewYear {
            starting_issuance: BalanceOf<T>,
            next_era_starting_block: BlockNumberFor<T>,
        },

        /// Beginning of a new era.
        NewEra {
            era: u32,
            next_era_starting_block: BlockNumberFor<T>,
        },

        /// Tokens minted due to inflation.
        InflationMinted {
            year_start_issuance: BalanceOf<T>,
            current_issuance: BalanceOf<T>,
            expected_new_issuance: BalanceOf<T>,
            minted: BalanceOf<T>,
        },

        /// Total supply of the token is higher than expected by Checked Inflation.
        OverInflationDetected {
            expected_issuance: BalanceOf<T>,
            current_issuance: BalanceOf<T>,
        },

        /// Halt status changed.
        HaltChanged { is_halted: bool },
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        BalanceOf<T>: CheckedSub,
    {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let previous_era = Self::current_era();
            let next_era_starting_block = Self::next_era_starting_block();

            let blocks_per_era = T::BlocksPerEra::get();

            let eras_per_year = T::ErasPerYear::get();

            let is_halted = Self::is_halted();

            if previous_era >= eras_per_year && now >= next_era_starting_block
                || next_era_starting_block == Zero::zero()
            {
                // Reset block # back to 1 for the new year
                CurrentEra::<T>::put(1);

                NextEraStartingBlock::<T>::put(now + blocks_per_era);

                let current_issuance =
                    <<T as Config>::Currency as Currency<T::AccountId>>::total_issuance();

                YearStartIssuance::<T>::put(current_issuance);

                let inflation_per_era = GetInflation::<T>::get_inflation_args(
                    &T::Inflation::get(),
                    eras_per_year,
                    current_issuance,
                );

                YearlyInflationPerEra::<T>::put(inflation_per_era);

                Self::deposit_event(Event::NewYear {
                    starting_issuance: current_issuance,
                    next_era_starting_block: (now + blocks_per_era),
                });

                if !is_halted {
                    Self::mint(inflation_per_era);

                    Self::deposit_event(Event::InflationMinted {
                        year_start_issuance: current_issuance,
                        current_issuance,
                        expected_new_issuance: current_issuance + inflation_per_era,
                        minted: inflation_per_era,
                    });
                }

                T::DbWeight::get().reads_writes(7, 3)
            } else {
                let inflation_per_era = Self::inflation_per_era();

                if now >= next_era_starting_block || previous_era.is_zero() {
                    CurrentEra::<T>::put(previous_era + 1);

                    NextEraStartingBlock::<T>::put(now + blocks_per_era);

                    Self::deposit_event(Event::NewEra {
                        era: (previous_era + 1),
                        next_era_starting_block: (now + blocks_per_era),
                    });

                    if !is_halted {
                        // Get issuance that the year started at
                        let start_issuance = Self::year_start_issuance();

                        // Get actual current total token issuance
                        let current_issuance =
                            <<T as Config>::Currency as Currency<T::AccountId>>::total_issuance();

                        // Calculate the expected current total token issuance
                        let expected_current_issuance =
                            start_issuance + (inflation_per_era * previous_era.into());

                        // Check that current_issuance and expected_current_issuance match in value.
                        // If the result is > 0, too many tokens were minted.
                        match current_issuance.checked_sub(&expected_current_issuance) {
                            // Either current issuance matches the expected issuance, or current issuance is higher than expected
                            // meaning too many tokens were minted
                            Some(over_inflation) if over_inflation > Zero::zero() => {
                                Self::deposit_event(Event::OverInflationDetected {
                                    expected_issuance: expected_current_issuance,
                                    current_issuance,
                                });

                                // Mint the difference
                                if let Some(to_mint) =
                                    inflation_per_era.checked_sub(&over_inflation)
                                {
                                    Self::mint(to_mint);

                                    Self::deposit_event(Event::InflationMinted {
                                        year_start_issuance: start_issuance,
                                        current_issuance,
                                        expected_new_issuance: expected_current_issuance
                                            + inflation_per_era,
                                        minted: to_mint,
                                    });
                                }
                            }

                            _ => {
                                Self::mint(inflation_per_era);

                                Self::deposit_event(Event::InflationMinted {
                                    year_start_issuance: start_issuance,
                                    current_issuance,
                                    expected_new_issuance: expected_current_issuance
                                        + inflation_per_era,
                                    minted: inflation_per_era,
                                });
                            }
                        }

                        T::DbWeight::get().reads_writes(8, 2)
                    } else {
                        T::DbWeight::get().reads_writes(6, 2)
                    }
                } else {
                    T::DbWeight::get().reads(6)
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// This call is used for configuring the inflation mechanism and sets the token's `YearStartIssuance` to its current total issuance.
        ///
        /// The origin has to have `root` access.
        #[pallet::call_index(0)]
        #[pallet::weight(
            <T as Config>::WeightInfo::set_first_year_supply()
        )]
        pub fn set_first_year_supply(root: OriginFor<T>) -> DispatchResult {
            ensure_root(root)?;

            YearStartIssuance::<T>::put(
                <<T as Config>::Currency as Currency<T::AccountId>>::total_issuance(),
            );

            Ok(())
        }

        /// Halts or unhalts the inflation process.
        ///
        /// The origin has to have `root` access.
        ///
        /// - `halt`: `true` to halt the inflation process, `false` to unhalt it.
        #[pallet::call_index(1)]
        #[pallet::weight(
            <T as Config>::WeightInfo::halt_unhalt_pallet()
        )]
        pub fn halt_unhalt_pallet(root: OriginFor<T>, halt: bool) -> DispatchResult {
            ensure_root(root)?;

            let is_halted = Self::is_halted();

            ensure!(is_halted ^ halt, Error::<T>::NoHaltChange);

            Self::internal_halt_unhalt(halt);

            Self::deposit_event(Event::<T>::HaltChanged { is_halted: halt });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Internal function for minting tokens to the currency due to inflation.
        fn mint(amount: BalanceOf<T>) {
            let inflation = T::Currency::issue(amount);
            <T as Config>::DealWithInflation::on_unbalanced(inflation);
        }

        /// Internal function to set the halt status to storage.
        pub fn internal_halt_unhalt(halt: bool) {
            Halted::<T>::put(halt);
        }
    }
}
