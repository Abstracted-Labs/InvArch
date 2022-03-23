//! Unit tests for the IPT pallet.

use frame_support::{assert_noop, assert_ok};

use crate::{
    mock::{ExistentialDeposit, ExtBuilder, Ipt, Origin, Runtime, ALICE, BOB},
    AssetDetails, Error, Ipt as IptStorage,
};

use sp_runtime::DispatchError;

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Ipt::create(ALICE, 0, vec![(ALICE, ExistentialDeposit::get())]);

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get(),
                deposit: 0,
            })
        );

        assert_ok!(Ipt::mint(Origin::signed(ALICE), 0, 1000, ALICE));

        assert_eq!(
            IptStorage::<Runtime>::get(0),
            Some(AssetDetails {
                owner: ALICE,
                supply: ExistentialDeposit::get() + 1000,
                deposit: 0,
            })
        );
    });
}
