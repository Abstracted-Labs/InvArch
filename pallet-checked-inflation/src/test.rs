use crate::{mock::*, *};
use frame_support::traits::{Currency, Imbalance};

#[test]
fn inflate_one_era() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(CheckedInflation::current_era(), 0);

        run_to_block(1);

        assert_eq!(CheckedInflation::current_era(), 1);

        let per_era = GetInflation::<Test>::get_inflation_args(
            &Inflation::get(),
            ERAS_PER_YEAR,
            GENESIS_ISSUANCE,
        );

        assert_eq!(Balances::total_issuance(), GENESIS_ISSUANCE + per_era);

        run_to_next_era();

        assert_eq!(CheckedInflation::current_era(), 2);

        assert_eq!(Balances::total_issuance(), GENESIS_ISSUANCE + (per_era * 2));
    });
}

#[test]
fn inflate_one_year() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(CheckedInflation::current_era(), 0);

        run_to_block(1);

        assert_eq!(CheckedInflation::current_era(), 1);

        let per_era = GetInflation::<Test>::get_inflation_args(
            &Inflation::get(),
            ERAS_PER_YEAR,
            GENESIS_ISSUANCE,
        );

        assert_eq!(Balances::total_issuance(), GENESIS_ISSUANCE + per_era);

        run_to_next_year();

        assert_eq!(
            Balances::total_issuance(),
            GENESIS_ISSUANCE + (per_era * ERAS_PER_YEAR as u128)
        );
    })
}

#[test]
fn overinflate_then_run_to_next_year() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(CheckedInflation::current_era(), 0);

        run_to_block(1);

        assert_eq!(CheckedInflation::current_era(), 1);

        let per_era = GetInflation::<Test>::get_inflation_args(
            &Inflation::get(),
            ERAS_PER_YEAR,
            GENESIS_ISSUANCE,
        );

        run_to_half_year();

        let pre_mint = Balances::total_issuance();

        Balances::deposit_creating(&ALICE, (per_era * ERAS_PER_YEAR as u128) / 4).peek();

        assert_ne!(pre_mint, Balances::total_issuance());

        run_to_next_year();

        assert_eq!(
            Balances::total_issuance(),
            GENESIS_ISSUANCE + (per_era * ERAS_PER_YEAR as u128)
        );
    })
}
