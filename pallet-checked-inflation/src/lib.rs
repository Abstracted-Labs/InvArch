#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
use sp_arithmetic::traits::Zero;
use sp_std::convert::TryInto;

mod inflation;
pub mod migrations;

pub use inflation::*;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, LockableCurrency, OnUnbalanced, ReservableCurrency},
    };
    use frame_system::pallet_prelude::OriginFor;
    use frame_system::{ensure_root, pallet_prelude::BlockNumberFor};
    use num_traits::CheckedSub;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
            + ReservableCurrency<Self::AccountId>
            + Currency<Self::AccountId>;

        #[pallet::constant]
        type BlocksPerEra: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type ErasPerYear: Get<u32>;

        #[pallet::constant]
        type Inflation: Get<InflationMethod<BalanceOf<Self>>>;

        type DealWithInflation: OnUnbalanced<NegativeImbalanceOf<Self>>;
    }

    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_era_starting_block)]
    pub type NextEraStartingBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_year)]
    pub type CurrentYear<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn year_start_issuance)]
    pub type YearStartIssuance<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn inflation_per_era)]
    pub type YearlyInflationPerEra<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        NewYear {
            starting_issuance: BalanceOf<T>,
            next_era_starting_block: BlockNumberFor<T>,
        },

        NewEra {
            era: u32,
            next_era_starting_block: BlockNumberFor<T>,
        },

        InflationMinted {
            year_start_issuance: BalanceOf<T>,
            current_issuance: BalanceOf<T>,
            expected_new_issuance: BalanceOf<T>,
            minted: BalanceOf<T>,
        },

        OverInflationDetected {
            expected_issuance: BalanceOf<T>,
            current_issuance: BalanceOf<T>,
        },
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

            if previous_era >= eras_per_year && now >= next_era_starting_block {
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

                Self::mint(inflation_per_era);

                Self::deposit_event(Event::InflationMinted {
                    year_start_issuance: current_issuance,
                    current_issuance,
                    expected_new_issuance: current_issuance + inflation_per_era,
                    minted: inflation_per_era,
                });

                T::DbWeight::get().reads_writes(6, 3)
            } else {
                let inflation_per_era = Self::inflation_per_era();

                if now >= next_era_starting_block || previous_era.is_zero() {
                    CurrentEra::<T>::put(previous_era + 1);

                    NextEraStartingBlock::<T>::put(now + blocks_per_era);

                    Self::deposit_event(Event::NewEra {
                        era: (previous_era + 1),
                        next_era_starting_block: (now + blocks_per_era),
                    });

                    let start_issuance = Self::year_start_issuance();
                    let current_issuance =
                        <<T as Config>::Currency as Currency<T::AccountId>>::total_issuance();

                    let expected_current_issuance =
                        start_issuance + (inflation_per_era * previous_era.into());

                    match current_issuance.checked_sub(&expected_current_issuance) {
                        Some(over_inflation) if over_inflation > Zero::zero() => {
                            Self::deposit_event(Event::OverInflationDetected {
                                expected_issuance: expected_current_issuance,
                                current_issuance,
                            });

                            if let Some(to_mint) = inflation_per_era.checked_sub(&over_inflation) {
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

                    T::DbWeight::get().reads_writes(7, 2)
                } else {
                    T::DbWeight::get().reads(5)
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000_000)]
        pub fn set_first_year_supply(root: OriginFor<T>) -> DispatchResult {
            ensure_root(root)?;

            YearStartIssuance::<T>::put(
                <<T as Config>::Currency as Currency<T::AccountId>>::total_issuance(),
            );

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn mint(amount: BalanceOf<T>) {
            let inflation = T::Currency::issue(amount);
            <T as Config>::DealWithInflation::on_unbalanced(inflation);
        }
    }
}
