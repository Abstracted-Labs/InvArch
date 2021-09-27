//! Unit tests for the IPO pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;
use sp_runtime::DispatchError;

#[test]
fn issue_ipo_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Ipo::next_ipo_id(), 0);
        assert_ok!(Ipo::issue_ipo(
            Origin::signed(BOB),
            MOCK_METADATA.to_vec(),
            H256::from(MOCK_DATA)
        ));
        assert_eq!(Ipo::next_ipo_id(), 1);
        assert_ok!(Ipo::issue_ipo(
            Origin::signed(ALICE),
            MOCK_METADATA_SECONDARY.to_vec(),
            H256::from(MOCK_DATA_SECONDARY)
        ));
        assert_eq!(Ipo::next_ipo_id(), 2);

        assert_eq!(
            IpoStorage::<T>::get(0),
            Some(IpoInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA)
            })
        );

        assert_eq!(
            IpoStorage::<T>::get(1),
            Some(IpoInfoOf::<Runtime> {
                owner: BOB,
                metadata: MOCK_METADATA_SECONDARY.to_vec().try_into().unwrap(),
                data: H256::from(MOCK_DATA_SECONDARY)
            })
        );
    });
}

fn issue_ipo_should_fail() {}

fn transfer_should_work() {}

fn transfer_should_fail() {}

fn set_balance_should_work() {}

fn set_balance_should_fail() {}

fn bind_should_work() {}

fn bind_should_fail() {}

fn unbind_should_work() {}

fn unbind_should_fail() {}