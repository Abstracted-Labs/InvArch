use crate::testing::*;
use crate::{pallet::Error, pallet::Event, *};
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use mock::Balances;
use sp_runtime::{traits::Zero, Perbill};

#[test]
fn on_initialize_when_ip_staking_enabled_in_mid_of_an_era_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        System::set_block_number(2);

        assert_eq!(0u32, IpStaking::current_era());

        IpStaking::on_initialize(System::block_number());
        assert_eq!(1u32, IpStaking::current_era());
    })
}

#[test]
fn rewards_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        assert_eq!(RewardAccumulator::<Test>::get(), Default::default());
        assert!(Balances::free_balance(&IpStaking::account_id()).is_zero());

        let total_reward = 22344;
        IpStaking::rewards(Balances::issue(total_reward));

        assert_eq!(
            total_reward,
            Balances::free_balance(&IpStaking::account_id())
        );
        let reward_accumulator = RewardAccumulator::<Test>::get();

        let (ip_reward, stakers_reward) = split_reward_amount(total_reward);

        assert_eq!(reward_accumulator.stakers, stakers_reward);
        assert_eq!(reward_accumulator.ip, ip_reward);

        IpStaking::on_initialize(System::block_number());
        assert_eq!(RewardAccumulator::<Test>::get(), Default::default());
        assert_eq!(
            total_reward,
            Balances::free_balance(&IpStaking::account_id())
        );
    })
}

#[test]
fn on_initialize_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        assert!(IpStaking::current_era().is_zero());

        initialize_first_block();
        let current_era = IpStaking::current_era();
        assert_eq!(1, current_era);

        let previous_era = current_era;
        advance_to_era(previous_era + 10);

        let current_era = IpStaking::current_era();
        for era in 1..current_era {
            let reward_info = GeneralEraInfo::<Test>::get(era).unwrap().rewards;
            assert_eq!(ISSUE_PER_ERA, reward_info.stakers + reward_info.ip);
        }
        let era_rewards = GeneralEraInfo::<Test>::get(current_era).unwrap();
        assert_eq!(0, era_rewards.staked);
        assert_eq!(era_rewards.rewards, Default::default());
    })
}

#[test]
fn new_era_length_is_always_blocks_per_era() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();
        let blocks_per_era = mock::BLOCKS_PER_ERA;

        advance_to_era(mock::IpStaking::current_era() + 1);

        let start_era = mock::IpStaking::current_era();
        let starting_block_number = System::block_number();

        advance_to_era(mock::IpStaking::current_era() + 1);
        let ending_block_number = System::block_number();

        assert_eq!(mock::IpStaking::current_era(), start_era + 1);
        assert_eq!(ending_block_number - starting_block_number, blocks_per_era);
    })
}

#[test]
fn new_era_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        advance_to_era(IpStaking::current_era() + 10);
        let starting_era = IpStaking::current_era();

        assert_eq!(IpStaking::reward_accumulator(), Default::default());

        run_for_blocks(1);
        let current_era = IpStaking::current_era();
        assert_eq!(starting_era, current_era);

        let block_reward = IpStaking::reward_accumulator();
        assert_eq!(ISSUE_PER_BLOCK, block_reward.stakers + block_reward.ip);

        let staker = account(C);
        let staked_amount = 100;
        assert_register(A);
        assert_stake(staker, &A, staked_amount);

        advance_to_era(IpStaking::current_era() + 1);

        let current_era = IpStaking::current_era();
        assert_eq!(starting_era + 1, current_era);
        System::assert_last_event(mock::Event::IpStaking(Event::NewEra {
            era: starting_era + 1,
        }));

        let block_reward = IpStaking::reward_accumulator();
        assert_eq!(block_reward, Default::default());

        let expected_era_reward = ISSUE_PER_ERA;

        let (expected_ip_reward, expected_stakers_reward) = split_reward_amount(ISSUE_PER_ERA);

        let era_rewards = GeneralEraInfo::<Test>::get(starting_era).unwrap();
        assert_eq!(staked_amount, era_rewards.staked);
        assert_eq!(
            expected_era_reward,
            era_rewards.rewards.ip + era_rewards.rewards.stakers
        );
        assert_eq!(expected_ip_reward, era_rewards.rewards.ip);
        assert_eq!(expected_stakers_reward, era_rewards.rewards.stakers);
    })
}

#[test]
fn general_staker_info_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        assert_register(A);

        assert_register(B);

        let (staker_1, staker_2, staker_3) = (account(C), account(D), account(E));
        let amount = 100;

        let starting_era = 3;
        advance_to_era(starting_era);
        assert_stake(staker_1, &A, amount);
        assert_stake(staker_2, &A, amount);

        let mid_era = 7;
        advance_to_era(mid_era);
        assert_unstake(staker_2, &A, amount);
        assert_stake(staker_3, &A, amount);
        assert_stake(staker_3, &B, amount);

        let final_era = 12;
        advance_to_era(final_era);

        let mut first_staker_info = IpStaking::staker_info(&A, &staker_1);
        let mut second_staker_info = IpStaking::staker_info(&A, &staker_2);
        let mut third_staker_info = IpStaking::staker_info(&A, &staker_3);

        for era in starting_era..mid_era {
            let ip_info = IpStaking::ip_stake_info(&A, era).unwrap();
            assert_eq!(2, ip_info.number_of_stakers);

            assert_eq!((era, amount), first_staker_info.claim());
            assert_eq!((era, amount), second_staker_info.claim());

            assert!(!IpEraStake::<Test>::contains_key(&B, era));
        }

        for era in mid_era..=final_era {
            let first_ip_info = IpStaking::ip_stake_info(&A, era).unwrap();
            assert_eq!(2, first_ip_info.number_of_stakers);

            assert_eq!((era, amount), first_staker_info.claim());
            assert_eq!((era, amount), third_staker_info.claim());

            assert_eq!(
                IpStaking::ip_stake_info(&B, era).unwrap().number_of_stakers,
                1
            );
        }

        assert!(!IpEraStake::<Test>::contains_key(&A, starting_era - 1));
        assert!(!IpEraStake::<Test>::contains_key(&B, starting_era - 1));
    })
}

#[test]
fn register_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        assert!(<Test as Config>::Currency::reserved_balance(&account(A)).is_zero());
        assert_register(A);
        System::assert_last_event(mock::Event::IpStaking(Event::IpRegistered { ip: A }));

        assert_eq!(
            RegisterDeposit::get(),
            <Test as Config>::Currency::reserved_balance(&account(A))
        );
    })
}

#[test]
fn register_twice_with_same_account_fails() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        assert_register(A);

        System::assert_last_event(mock::Event::IpStaking(Event::IpRegistered { ip: A }));

        assert_noop!(
            IpStaking::register_ip(
                Origin::signed(account(A)),
                A,
                Vec::new(),
                Vec::new(),
                H256::default()
            ),
            Error::<Test>::IpAlreadyRegistered
        );
    })
}

#[test]
fn unregister_after_register_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        assert_register(A);
        assert_unregister(A);

        assert!(<Test as Config>::Currency::reserved_balance(&account(A)).is_zero());

        assert_noop!(
            IpStaking::unregister_ip(Origin::signed(account(A)), A),
            Error::<Test>::NotRegistered
        );
    })
}

#[test]
fn unregister_stake_and_unstake_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(C);

        assert_register(A);
        assert_stake(staker, &A, 100);
        assert_unstake(staker, &A, 10);

        assert_unregister(A);

        assert_noop!(
            IpStaking::stake(Origin::signed(staker), A, 100),
            Error::<Test>::NotRegistered
        );
        assert_noop!(
            IpStaking::unstake(Origin::signed(staker), A, 100),
            Error::<Test>::NotRegistered
        );
    })
}

#[test]
fn withdraw_from_unregistered_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_1 = account(D);
        let staker_2 = account(E);
        let staked_value_1 = 150;
        let staked_value_2 = 330;
        let ip_id = A;
        let dummy_ip_id = B;

        assert_register(ip_id);
        assert_register(dummy_ip_id);
        assert_stake(staker_1, &ip_id, staked_value_1);
        assert_stake(staker_2, &ip_id, staked_value_2);

        assert_stake(staker_1, &dummy_ip_id, staked_value_1);

        advance_to_era(5);

        assert_unregister(ip_id);

        for era in 1..IpStaking::current_era() {
            assert_claim_staker(staker_1, ip_id);
            assert_claim_staker(staker_2, ip_id);

            assert_claim_ip(ip_id, era);
        }

        assert_noop!(
            IpStaking::staker_claim_rewards(Origin::signed(staker_1), ip_id),
            Error::<Test>::NoStakeAvailable
        );
        assert_noop!(
            IpStaking::staker_claim_rewards(Origin::signed(staker_2), ip_id),
            Error::<Test>::NoStakeAvailable
        );
        assert_noop!(
            IpStaking::ip_claim_rewards(
                Origin::signed(account(ip_id)),
                ip_id,
                IpStaking::current_era()
            ),
            Error::<Test>::IncorrectEra
        );

        advance_to_era(8);

        assert_withdraw_unbonded(staker_1);
        assert_withdraw_unbonded(staker_2);
    })
}

#[test]
fn bond_and_stake_different_eras_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_id = account(B);
        let ip_id = A;
        assert_register(ip_id);

        let current_era = IpStaking::current_era();
        assert!(IpStaking::ip_stake_info(&ip_id, current_era).is_none());

        assert_stake(staker_id, &ip_id, 100);

        advance_to_era(current_era + 2);

        assert_stake(staker_id, &ip_id, 300);
    })
}

#[test]
fn bond_and_stake_two_different_ip_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_id = account(B);
        let first_ip_id = A;
        let second_ip_id = C;

        assert_register(first_ip_id);
        assert_register(second_ip_id);

        assert_stake(staker_id, &first_ip_id, 100);
        assert_stake(staker_id, &second_ip_id, 300);
    })
}

#[test]
fn bond_and_stake_two_stakers_one_ip_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let first_staker_id = account(B);
        let second_staker_id = account(C);
        let first_stake_value = 50;
        let second_stake_value = 235;
        let ip_id = A;

        assert_register(ip_id);

        assert_stake(first_staker_id, &ip_id, first_stake_value);
        assert_stake(second_staker_id, &ip_id, second_stake_value);
    })
}

#[test]
fn bond_and_stake_different_value_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_id = account(B);
        let ip_id = A;

        assert_register(ip_id);

        let staker_free_balance =
            Balances::free_balance(&staker_id).saturating_sub(EXISTENTIAL_DEPOSIT);
        assert_stake(staker_id, &ip_id, staker_free_balance - 1);

        assert_stake(staker_id, &ip_id, 1);

        let staker_id = account(C);
        let staker_free_balance = Balances::free_balance(&staker_id);
        assert_stake(staker_id, &ip_id, staker_free_balance + 1);

        let transferable_balance =
            Balances::free_balance(&staker_id) - Ledger::<Test>::get(staker_id).locked;
        assert_eq!(EXISTENTIAL_DEPOSIT, transferable_balance);

        let staker_id = account(D);
        let staker_free_balance =
            Balances::free_balance(&staker_id).saturating_sub(EXISTENTIAL_DEPOSIT);
        assert_stake(staker_id, &ip_id, staker_free_balance - 200);

        assert_stake(staker_id, &ip_id, 500);
    })
}

#[test]
fn bond_and_stake_on_unregistered_ip_fails() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_id = account(B);
        let stake_value = 100;

        let ip_id = A;
        assert_noop!(
            IpStaking::stake(Origin::signed(staker_id), ip_id, stake_value),
            Error::<Test>::NotRegistered
        );
    })
}

#[test]
fn bond_and_stake_insufficient_value() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();
        let staker_id = account(B);
        let ip_id = A;

        assert_register(ip_id);

        assert_noop!(
            IpStaking::stake(Origin::signed(staker_id), ip_id, MINIMUM_STAKING_AMOUNT - 1),
            Error::<Test>::InsufficientBalance
        );

        let staker_free_balance = Balances::free_balance(&staker_id);
        assert_stake(staker_id, &ip_id, staker_free_balance);

        assert_noop!(
            IpStaking::stake(Origin::signed(staker_id), ip_id, 1),
            Error::<Test>::StakingNothing
        );
    })
}

#[test]
fn bond_and_stake_too_many_stakers_per_ip() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let ip_id = A;
        assert_register(ip_id);

        for staker_id in 1..=MAX_NUMBER_OF_STAKERS {
            assert_stake(account(staker_id.into()), &ip_id, 100);
        }

        assert_noop!(
            IpStaking::stake(
                Origin::signed(account((1 + MAX_NUMBER_OF_STAKERS).into())),
                ip_id,
                100
            ),
            Error::<Test>::MaxStakersReached
        );
    })
}

#[test]
fn bond_and_stake_too_many_era_stakes() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_id = account(B);
        let ip_id = A;
        assert_register(ip_id);

        let start_era = IpStaking::current_era();
        for offset in 1..MAX_ERA_STAKE_VALUES {
            assert_stake(staker_id, &ip_id, 100);
            advance_to_era(start_era + offset);
        }

        assert_noop!(
            IpStaking::stake(Origin::signed(staker_id.into()), ip_id, 100),
            Error::<Test>::TooManyEraStakeValues
        );
    })
}

#[test]
fn unbond_and_unstake_multiple_time_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_id = account(B);
        let ip_id = A;
        let original_staked_value = 300 + EXISTENTIAL_DEPOSIT;
        let old_era = IpStaking::current_era();

        assert_register(ip_id);
        assert_stake(staker_id, &ip_id, original_staked_value);
        advance_to_era(old_era + 1);

        let unstaked_value = 100;
        assert_unstake(staker_id, &ip_id, unstaked_value);

        let unstaked_value = 50;
        assert_unstake(staker_id, &ip_id, unstaked_value);
    })
}

#[test]
fn unbond_and_unstake_value_below_staking_threshold() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_id = account(B);
        let ip_id = A;
        let first_value_to_unstake = 300;
        let staked_value = first_value_to_unstake + MINIMUM_STAKING_AMOUNT;

        assert_register(ip_id);
        assert_stake(staker_id, &ip_id, staked_value);

        assert_unstake(staker_id, &ip_id, first_value_to_unstake);

        assert_unstake(staker_id, &ip_id, 1);
    })
}

#[test]
fn unbond_and_unstake_in_different_eras() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let (first_staker_id, second_staker_id) = (account(B), account(C));
        let ip_id = A;
        let staked_value = 500;

        assert_register(ip_id);
        assert_stake(first_staker_id, &ip_id, staked_value);
        assert_stake(second_staker_id, &ip_id, staked_value);

        advance_to_era(IpStaking::current_era() + 10);
        let current_era = IpStaking::current_era();
        assert_unstake(first_staker_id, &ip_id, 100);

        advance_to_era(current_era + 10);
        assert_unstake(second_staker_id, &ip_id, 333);
    })
}

#[test]
fn unbond_and_unstake_calls_in_same_era_can_exceed_max_chunks() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let ip_id = A;
        assert_register(ip_id);

        let staker = account(B);
        assert_stake(staker, &ip_id, 200 * MAX_UNLOCKING as Balance);

        for _ in 0..MAX_UNLOCKING * 2 {
            assert_unstake(staker, &ip_id, 10);
            assert_eq!(1, Ledger::<Test>::get(&staker).unbonding_info.len());
        }
    })
}

#[test]
fn unbond_and_unstake_with_zero_value_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let ip_id = A;
        assert_register(ip_id);

        assert_noop!(
            IpStaking::unstake(Origin::signed(account(B)), ip_id, 0),
            Error::<Test>::UnstakingNothing
        );
    })
}

#[test]
fn unbond_and_unstake_on_not_registered_ip_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let ip_id = A;
        assert_noop!(
            IpStaking::unstake(Origin::signed(account(B)), ip_id, 100),
            Error::<Test>::NotRegistered
        );
    })
}

#[test]
fn unbond_and_unstake_too_many_unlocking_chunks_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let ip_id = A;
        assert_register(ip_id);

        let staker = account(B);
        let unstake_amount = 10;
        let stake_amount = MINIMUM_STAKING_AMOUNT * 10 + unstake_amount * MAX_UNLOCKING as Balance;

        assert_stake(staker, &ip_id, stake_amount);

        for _ in 0..MAX_UNLOCKING {
            advance_to_era(IpStaking::current_era() + 1);
            assert_unstake(staker, &ip_id, unstake_amount);
        }

        assert_eq!(
            MAX_UNLOCKING,
            IpStaking::ledger(&staker).unbonding_info.len()
        );
        assert_unstake(staker, &ip_id, unstake_amount);

        advance_to_era(IpStaking::current_era() + 1);
        assert_noop!(
            IpStaking::unstake(Origin::signed(staker), ip_id.clone(), unstake_amount),
            Error::<Test>::TooManyUnlockingChunks,
        );
    })
}

#[test]
fn unbond_and_unstake_on_not_staked_ip_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let ip_id = A;
        assert_register(ip_id);

        assert_noop!(
            IpStaking::unstake(Origin::signed(account(B)), ip_id, 10),
            Error::<Test>::NoStakeAvailable,
        );
    })
}

#[test]
fn unbond_and_unstake_too_many_era_stakes() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker_id = account(B);
        let ip_id = A;
        assert_register(ip_id);

        let start_era = IpStaking::current_era();
        for offset in 1..MAX_ERA_STAKE_VALUES {
            assert_stake(staker_id, &ip_id, 100);
            advance_to_era(start_era + offset);
        }

        assert_noop!(
            IpStaking::unstake(Origin::signed(staker_id), ip_id, 10),
            Error::<Test>::TooManyEraStakeValues
        );
    })
}

#[test]
fn withdraw_unbonded_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let ip_id = A;
        assert_register(ip_id);

        let staker_id = account(B);
        assert_stake(staker_id, &ip_id, 1000);

        let first_unbond_value = 75;
        let second_unbond_value = 39;
        let initial_era = IpStaking::current_era();

        assert_unstake(staker_id, &ip_id, first_unbond_value);

        advance_to_era(initial_era + 1);
        assert_unstake(staker_id, &ip_id, second_unbond_value);

        advance_to_era(initial_era + UNBONDING_PERIOD - 1);
        assert_noop!(
            IpStaking::withdraw_unstaked(Origin::signed(staker_id)),
            Error::<Test>::NothingToWithdraw
        );

        advance_to_era(IpStaking::current_era() + 1);
        assert_ok!(IpStaking::withdraw_unstaked(Origin::signed(staker_id),));
        System::assert_last_event(mock::Event::IpStaking(Event::Withdrawn {
            staker: staker_id,
            amount: first_unbond_value,
        }));

        advance_to_era(IpStaking::current_era() + 1);
        assert_ok!(IpStaking::withdraw_unstaked(Origin::signed(staker_id),));
        System::assert_last_event(mock::Event::IpStaking(Event::Withdrawn {
            staker: staker_id,
            amount: second_unbond_value,
        }));

        advance_to_era(initial_era + UNBONDING_PERIOD - 1);
        assert_noop!(
            IpStaking::withdraw_unstaked(Origin::signed(staker_id)),
            Error::<Test>::NothingToWithdraw
        );
    })
}

#[test]
fn withdraw_unbonded_full_vector_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let ip_id = A;
        assert_register(ip_id);

        let staker_id = account(B);
        assert_stake(staker_id, &ip_id, 1000);

        let init_unbonding_amount = 15;
        for x in 1..=MAX_UNLOCKING {
            assert_unstake(staker_id, &ip_id, init_unbonding_amount * x as u128);
            advance_to_era(IpStaking::current_era() + 1);
        }

        assert_withdraw_unbonded(staker_id);

        assert!(!Ledger::<Test>::get(&staker_id).unbonding_info.is_empty());

        while !Ledger::<Test>::get(&staker_id).unbonding_info.is_empty() {
            advance_to_era(IpStaking::current_era() + 1);
            assert_withdraw_unbonded(staker_id);
        }
    })
}

#[test]
fn withdraw_unbonded_no_value_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        assert_noop!(
            IpStaking::withdraw_unstaked(Origin::signed(account(B))),
            Error::<Test>::NothingToWithdraw,
        );
    })
}

#[test]
fn claim_not_staked_ip() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(B);
        let ip_id = A;

        assert_register(ip_id);

        assert_noop!(
            IpStaking::staker_claim_rewards(Origin::signed(staker), ip_id),
            Error::<Test>::NoStakeAvailable
        );

        advance_to_era(IpStaking::current_era() + 1);
        assert_noop!(
            IpStaking::ip_claim_rewards(Origin::signed(account(ip_id)), ip_id, 1),
            Error::<Test>::NoStakeAvailable
        );
    })
}

#[test]
fn claim_not_registered_ip() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(B);
        let ip_id = A;

        assert_register(ip_id);
        assert_stake(staker, &ip_id, 100);

        advance_to_era(IpStaking::current_era() + 1);
        assert_unregister(ip_id);

        assert_claim_staker(staker, ip_id);
        assert_noop!(
            IpStaking::staker_claim_rewards(Origin::signed(staker), ip_id),
            Error::<Test>::NoStakeAvailable
        );

        assert_claim_ip(ip_id, 1);
        assert_noop!(
            IpStaking::ip_claim_rewards(Origin::signed(account(ip_id)), ip_id, 2),
            Error::<Test>::IncorrectEra
        );
    })
}

#[test]
fn claim_invalid_era() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(B);
        let ip_id = A;

        let start_era = IpStaking::current_era();
        assert_register(ip_id);
        assert_stake(staker, &ip_id, 100);
        advance_to_era(start_era + 5);

        for era in start_era..IpStaking::current_era() {
            assert_claim_staker(staker, ip_id);
            assert_claim_ip(ip_id, era);
        }

        assert_noop!(
            IpStaking::staker_claim_rewards(Origin::signed(staker), ip_id),
            Error::<Test>::IncorrectEra
        );
        assert_noop!(
            IpStaking::ip_claim_rewards(
                Origin::signed(account(ip_id)),
                ip_id,
                IpStaking::current_era()
            ),
            Error::<Test>::IncorrectEra
        );
    })
}

#[test]
fn claim_ip_same_era_twice() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(B);
        let ip_id = A;

        let start_era = IpStaking::current_era();
        assert_register(ip_id);
        assert_stake(staker, &ip_id, 100);
        advance_to_era(start_era + 1);

        assert_claim_ip(ip_id, start_era);
        assert_noop!(
            IpStaking::ip_claim_rewards(Origin::signed(account(ip_id)), ip_id, start_era),
            Error::<Test>::RewardAlreadyClaimed
        );
    })
}

#[test]
fn claim_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let first_staker = account(D);
        let second_staker = account(E);
        let first_ip_id = A;
        let second_ip_id = B;

        let start_era = IpStaking::current_era();

        assert_register(first_ip_id);
        assert_register(second_ip_id);
        assert_stake(first_staker, &first_ip_id, 100);
        assert_stake(second_staker, &first_ip_id, 45);

        assert_stake(first_staker, &second_ip_id, 33);
        assert_stake(second_staker, &second_ip_id, 22);

        let eras_advanced = 3;
        advance_to_era(start_era + eras_advanced);

        for x in 0..eras_advanced.into() {
            assert_stake(first_staker, &first_ip_id, 20 + x * 3);
            assert_stake(second_staker, &first_ip_id, 5 + x * 5);
            advance_to_era(IpStaking::current_era() + 1);
        }

        let current_era = IpStaking::current_era();
        for era in start_era..current_era {
            assert_claim_staker(first_staker, first_ip_id);
            assert_claim_ip(first_ip_id, era);
            assert_claim_staker(second_staker, first_ip_id);
        }

        assert_noop!(
            IpStaking::staker_claim_rewards(Origin::signed(first_staker), first_ip_id.clone()),
            Error::<Test>::IncorrectEra
        );
        assert_noop!(
            IpStaking::ip_claim_rewards(
                Origin::signed(account(first_ip_id)),
                first_ip_id,
                current_era
            ),
            Error::<Test>::IncorrectEra
        );
    })
}

#[test]
fn claim_check_amount() {
    ExternalityBuilder::build().execute_with(|| {
        assert_eq!(System::block_number(), 1 as BlockNumber);

        IpStaking::on_initialize(System::block_number());

        let first_staker = account(C);
        let second_staker = account(D);
        let first_ip_id = A;
        let second_ip_id = B;

        assert_eq!(IpStaking::current_era(), 1);

        // Make sure current block is 1.
        assert_eq!(System::block_number(), 1);

        assert_register(first_ip_id);
        assert_register(second_ip_id);

        // 130 for stakers, 130 for Ip.
        issue_rewards(260);

        run_to_block_no_rewards(2);

        // Make sure current block is 2.
        assert_eq!(System::block_number(), 2);

        // User stakes in the middle of era 1, their stake should not account for era 1.
        assert_stake(first_staker, &first_ip_id, 100);
        assert_stake(second_staker, &second_ip_id, 30);

        advance_to_era_no_rewards(2);

        // Make sure current era is 2.
        assert_eq!(IpStaking::current_era(), 2);

        // 130 for stakers, 130 for Ip.
        issue_rewards(260);

        // Nothing else happens in era 2.
        advance_to_era_no_rewards(3);

        assert_eq!(
            IpStaking::ip_stake_info(first_ip_id, 1),
            Some(IpStakeInfo {
                total: 100,
                number_of_stakers: 1,
                reward_claimed: false,
                active: false
            })
        );

        assert_eq!(
            IpStaking::ip_stake_info(second_ip_id, 1),
            Some(IpStakeInfo {
                total: 30,
                number_of_stakers: 1,
                reward_claimed: false,
                active: false
            })
        );

        assert_eq!(
            IpStaking::general_era_info(1),
            Some(EraInfo {
                rewards: RewardInfo {
                    stakers: 130,
                    ip: 130
                },
                staked: 130,
                active_stake: 0,
                locked: 130
            })
        );

        assert_eq!(
            IpStaking::ip_stake_info(first_ip_id, 2),
            Some(IpStakeInfo {
                total: 100,
                number_of_stakers: 1,
                reward_claimed: false,
                active: true
            })
        );

        assert_eq!(
            IpStaking::ip_stake_info(second_ip_id, 2),
            Some(IpStakeInfo {
                total: 30,
                number_of_stakers: 1,
                reward_claimed: false,
                active: false
            })
        );

        assert_eq!(
            IpStaking::general_era_info(2),
            Some(EraInfo {
                rewards: RewardInfo {
                    stakers: 130,
                    ip: 130
                },
                staked: 130,
                active_stake: 100,
                locked: 130
            })
        );

        // Let's try claiming rewards for era 1 for the first ip...
        assert_ok!(IpStaking::ip_claim_rewards(
            Origin::signed(account(first_ip_id)),
            first_ip_id,
            1
        ));

        // ...there should be nothing.
        System::assert_last_event(mock::Event::IpStaking(Event::IpClaimed {
            ip: first_ip_id,
            destination_account: account(first_ip_id),
            era: 1,
            amount: 0,
        }));

        // Let's try claiming rewards for era 1 for the second ip...
        assert_ok!(IpStaking::ip_claim_rewards(
            Origin::signed(account(second_ip_id)),
            second_ip_id,
            1
        ));

        // ...there should be nothing.
        System::assert_last_event(mock::Event::IpStaking(Event::IpClaimed {
            ip: second_ip_id,
            destination_account: account(second_ip_id),
            era: 1,
            amount: 0,
        }));

        // Now let's try claiming rewards for era 2 for the first ip...
        assert_ok!(IpStaking::ip_claim_rewards(
            Origin::signed(account(first_ip_id)),
            first_ip_id,
            2
        ));

        // ...there should be 130 since it's 50% of the issue 260 and the second ip shouldn't be active yet.
        System::assert_last_event(mock::Event::IpStaking(Event::IpClaimed {
            ip: first_ip_id,
            destination_account: account(first_ip_id),
            era: 2,
            amount: 130,
        }));

        // Now let's try claiming rewards for era 2 for the second ip...
        assert_ok!(IpStaking::ip_claim_rewards(
            Origin::signed(account(second_ip_id)),
            second_ip_id,
            2
        ));

        // ...there should be 0 since the current stake is 30, which is below the active threshold.
        System::assert_last_event(mock::Event::IpStaking(Event::IpClaimed {
            ip: second_ip_id,
            destination_account: account(second_ip_id),
            era: 2,
            amount: 0,
        }));

        // User stakes in the middle of era 3, their stake should not account for era 3.
        assert_stake(first_staker, &second_ip_id, 20);

        advance_to_era_no_rewards(4);

        // Make sure current era is 4.
        assert_eq!(IpStaking::current_era(), 4);

        // 150 for stakers, 150 for Ip.
        issue_rewards(300);

        // Nothing else happens in era 4.
        advance_to_era_no_rewards(5);

        assert_eq!(
            IpStaking::ip_stake_info(first_ip_id, 4),
            Some(IpStakeInfo {
                total: 100,
                number_of_stakers: 1,
                reward_claimed: false,
                active: true
            })
        );

        assert_eq!(
            IpStaking::ip_stake_info(second_ip_id, 4),
            Some(IpStakeInfo {
                total: 50,
                number_of_stakers: 2,
                reward_claimed: false,
                active: true
            })
        );

        assert_eq!(
            IpStaking::general_era_info(4),
            Some(EraInfo {
                rewards: RewardInfo {
                    stakers: 150,
                    ip: 150
                },
                staked: 150,
                active_stake: 150,
                locked: 150
            })
        );

        // Let's try claiming rewards for era 4 for the first ip...
        assert_ok!(IpStaking::ip_claim_rewards(
            Origin::signed(account(first_ip_id)),
            first_ip_id,
            4
        ));

        // ...there should be 100 out of the 150, because the second ip should be active now.
        System::assert_last_event(mock::Event::IpStaking(Event::IpClaimed {
            ip: first_ip_id,
            destination_account: account(first_ip_id),
            era: 4,
            amount: 100,
        }));

        // Let's try claiming rewards for era 4 for the second ip...
        assert_ok!(IpStaking::ip_claim_rewards(
            Origin::signed(account(second_ip_id)),
            second_ip_id,
            4
        ));

        // ...there should be 50 out of the 150, because the second ip should be active now.
        System::assert_last_event(mock::Event::IpStaking(Event::IpClaimed {
            ip: second_ip_id,
            destination_account: account(second_ip_id),
            era: 4,
            amount: 50,
        }));

        // Now let's check the same stuff for the stakers instead of the ip.

        assert_eq!(
            IpStaking::staker_info(first_ip_id, first_staker),
            StakerInfo {
                stakes: vec![EraStake {
                    staked: 100,
                    era: 1
                }]
            }
        );

        assert_eq!(
            IpStaking::staker_info(second_ip_id, first_staker),
            StakerInfo {
                stakes: vec![EraStake { staked: 20, era: 3 }]
            }
        );

        assert_eq!(
            IpStaking::staker_info(second_ip_id, second_staker),
            StakerInfo {
                stakes: vec![EraStake { staked: 30, era: 1 }]
            }
        );

        assert_eq!(
            IpStaking::staker_info(first_ip_id, second_staker),
            StakerInfo { stakes: vec![] }
        );

        // Era 1:

        // Let's try claiming rewards for the first staker in the first ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(first_staker),
            first_ip_id,
        ));

        // ...there should be 100 out of the 130, because the second staker had 30 staked in era 1.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: first_staker,
            ip: first_ip_id,
            era: 1,
            amount: 100,
        }));

        // Let's try claiming rewards for the second staker in the second ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(second_staker),
            second_ip_id,
        ));

        // ...there should be 30 out of the 130, because the first staker had 100 staked in era 1.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: second_staker,
            ip: second_ip_id,
            era: 1,
            amount: 30,
        }));

        // Era 2:

        // Let's try claiming rewards for the first staker in the first ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(first_staker),
            first_ip_id,
        ));

        // ...there should be 100 out of the 130, because the second staker had 30 staked in era 2.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: first_staker,
            ip: first_ip_id,
            era: 2,
            amount: 100,
        }));

        // Let's try claiming rewards for the second staker in the second ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(second_staker),
            second_ip_id,
        ));

        // ...there should be 30 out of the 130, because the first staker had 100 staked in era 2.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: second_staker,
            ip: second_ip_id,
            era: 2,
            amount: 30,
        }));

        // Era 3:

        // Let's try claiming rewards for the first staker in the first ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(first_staker),
            first_ip_id,
        ));

        // ...there should be nothing, because no rewards were issue in era 3.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: first_staker,
            ip: first_ip_id,
            era: 3,
            amount: 0,
        }));

        // Let's try claiming rewards for the first staker in the second ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(first_staker),
            second_ip_id,
        ));

        // ...there should be nothing, because no rewards were issue in era 3.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: first_staker,
            ip: second_ip_id,
            era: 3,
            amount: 0,
        }));

        // Let's try claiming rewards for the second staker in the second ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(second_staker),
            second_ip_id,
        ));

        // ...there should be nothing, because no rewards were issue in era 3.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: second_staker,
            ip: second_ip_id,
            era: 3,
            amount: 0,
        }));

        // Era 4:

        // Let's try claiming rewards for the first staker in the first ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(first_staker),
            first_ip_id,
        ));

        // ...there should be 100 out of the 150, because the second staker had 30 staked in era 4 and first staker had 20 in the second ip.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: first_staker,
            ip: first_ip_id,
            era: 4,
            amount: 100,
        }));

        // Let's try claiming rewards for the first staker in the second ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(first_staker),
            second_ip_id,
        ));

        // ...there should be 20 out of the 150, because the second staker had 30 staked in era 4 and first staker had 100 in the first ip.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: first_staker,
            ip: second_ip_id,
            era: 4,
            amount: 20,
        }));

        // Let's try claiming rewards for the second staker in the second ip...
        assert_ok!(IpStaking::staker_claim_rewards(
            Origin::signed(second_staker),
            second_ip_id,
        ));

        // ...there should be 30 out of the 150, because the first staker had 120 staked in era 4.
        System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
            staker: second_staker,
            ip: second_ip_id,
            era: 4,
            amount: 30,
        }));
    })
}

#[test]
fn claim_after_unregister_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(B);
        let ip_id = A;

        let start_era = IpStaking::current_era();
        assert_register(ip_id);
        let stake_value = 100;
        assert_stake(staker, &ip_id, stake_value);

        advance_to_era(start_era + 5);
        assert_unstake(staker, &ip_id, stake_value);
        let full_unstake_era = IpStaking::current_era();
        let number_of_staking_eras = full_unstake_era - start_era;

        advance_to_era(IpStaking::current_era() + 3);
        let stake_value = 75;
        let restake_era = IpStaking::current_era();
        assert_stake(staker, &ip_id, stake_value);

        advance_to_era(IpStaking::current_era() + 3);
        assert_unregister(ip_id);
        let unregister_era = IpStaking::current_era();
        let number_of_staking_eras = number_of_staking_eras + unregister_era - restake_era;
        advance_to_era(IpStaking::current_era() + 2);

        for _ in 0..number_of_staking_eras {
            assert_claim_staker(staker, ip_id);
        }
        assert_noop!(
            IpStaking::staker_claim_rewards(Origin::signed(staker), ip_id.clone()),
            Error::<Test>::NoStakeAvailable
        );

        for era in start_era..unregister_era {
            if era >= full_unstake_era && era < restake_era {
                assert_noop!(
                    IpStaking::ip_claim_rewards(Origin::signed(account(A)), ip_id.clone(), era),
                    Error::<Test>::NoStakeAvailable
                );
            } else {
                assert_claim_ip(ip_id, era);
            }
        }
    })
}

#[test]
fn claim_only_payout_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(B);
        let ip_id = A;

        let start_era = IpStaking::current_era();
        assert_register(ip_id);
        let stake_value = 100;
        assert_stake(staker, &ip_id, stake_value);

        advance_to_era(start_era + 1);

        assert_claim_staker(staker, ip_id);
    })
}

#[test]
fn claim_with_zero_staked_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(B);
        let ip_id = A;
        let start_era = IpStaking::current_era();
        assert_register(ip_id);

        let stake_value = 100;
        assert_stake(staker, &ip_id, stake_value);
        advance_to_era(start_era + 1);

        assert_unstake(staker, &ip_id, stake_value);

        assert_claim_staker(staker, ip_id);
    })
}

#[test]
fn claim_ip_with_zero_stake_periods_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let staker = account(B);
        let ip_id = A;

        let start_era = IpStaking::current_era();
        assert_register(ip_id);
        let stake_value = 100;
        assert_stake(staker, &ip_id, stake_value);

        advance_to_era(start_era + 5);
        let first_full_unstake_era = IpStaking::current_era();
        assert_unstake(staker, &ip_id, stake_value);

        advance_to_era(IpStaking::current_era() + 7);
        let restake_era = IpStaking::current_era();
        assert_stake(staker, &ip_id, stake_value);

        advance_to_era(IpStaking::current_era() + 4);
        let second_full_unstake_era = IpStaking::current_era();
        assert_unstake(staker, &ip_id, stake_value);
        advance_to_era(IpStaking::current_era() + 10);

        for era in start_era..first_full_unstake_era {
            assert_claim_ip(ip_id, era);
        }

        for era in first_full_unstake_era..restake_era {
            assert_noop!(
                IpStaking::ip_claim_rewards(Origin::signed(account(ip_id)), ip_id.clone(), era),
                Error::<Test>::NoStakeAvailable
            );
        }

        for era in restake_era..second_full_unstake_era {
            assert_claim_ip(ip_id, era);
        }

        assert_noop!(
            IpStaking::ip_claim_rewards(
                Origin::signed(account(ip_id)),
                ip_id.clone(),
                second_full_unstake_era
            ),
            Error::<Test>::NoStakeAvailable
        );

        let last_claim_era = IpStaking::current_era();
        assert_stake(staker, &ip_id, stake_value);
        advance_to_era(last_claim_era + 1);
        assert_claim_ip(ip_id, last_claim_era);
    })
}

#[test]
fn ip_stakers_split_util() {
    let ip_rewards = 420;
    let stakers_rewards = 1337;
    let staked_on_ip = 123456;
    let total_staked = staked_on_ip * 2;

    let staking_points_active = IpStakeInfo::<Balance> {
        total: staked_on_ip,
        number_of_stakers: 10,
        reward_claimed: false,
        active: true,
    };

    let staking_points_inactive = IpStakeInfo::<Balance> {
        total: staked_on_ip,
        number_of_stakers: 10,
        reward_claimed: false,
        active: false,
    };

    let era_info = EraInfo::<Balance> {
        rewards: RewardInfo {
            ip: ip_rewards,
            stakers: stakers_rewards,
        },
        staked: total_staked,
        locked: total_staked,
        active_stake: staked_on_ip,
    };

    let (ip_reward, stakers_reward) =
        IpStaking::ip_stakers_split(&staking_points_active, &era_info);

    let ip_stake_ratio = Perbill::from_rational(staked_on_ip, total_staked);
    let calculated_stakers_reward = ip_stake_ratio * stakers_rewards;
    assert_eq!(ip_rewards, ip_reward);
    assert_eq!(calculated_stakers_reward, stakers_reward);

    assert_eq!(
        calculated_stakers_reward + ip_rewards,
        ip_reward + stakers_reward
    );

    let (ip_reward, stakers_reward) =
        IpStaking::ip_stakers_split(&staking_points_inactive, &era_info);

    let ip_stake_ratio = Perbill::from_rational(staked_on_ip, total_staked);
    let calculated_stakers_reward = ip_stake_ratio * stakers_rewards;
    assert_eq!(Balance::zero(), ip_reward);
    assert_eq!(calculated_stakers_reward, stakers_reward);

    assert_eq!(calculated_stakers_reward, ip_reward + stakers_reward);
}

#[test]
pub fn tvl_util_test() {
    ExternalityBuilder::build().execute_with(|| {
        assert!(IpStaking::tvl().is_zero());
        initialize_first_block();
        assert!(IpStaking::tvl().is_zero());

        let ip_id = A;
        assert_register(ip_id);

        let iterations = 10;
        let stake_value = 100;
        for x in 1..=iterations {
            assert_stake(account(ip_id), &ip_id, stake_value);
            assert_eq!(IpStaking::tvl(), stake_value * x);
        }

        advance_to_era(5);
        assert_eq!(IpStaking::tvl(), stake_value * iterations);
    })
}

#[test]
fn unbonding_info_test() {
    let mut unbonding_info = UnbondingInfo::<Balance>::default();

    assert!(unbonding_info.is_empty());
    assert!(unbonding_info.len().is_zero());
    let (first_info, second_info) = unbonding_info.clone().partition(2);
    assert!(first_info.is_empty());
    assert!(second_info.is_empty());

    let count = 5;
    let base_amount: Balance = 100;
    let base_unlock_era = 4 * count;
    let mut chunks = vec![];
    for x in 1_u32..=count as u32 {
        chunks.push(UnlockingChunk {
            amount: base_amount * x as Balance,
            unlock_era: base_unlock_era - 3 * x,
        });
    }

    unbonding_info.add(chunks[0 as usize]);

    assert!(!unbonding_info.is_empty());
    assert_eq!(1, unbonding_info.len());
    assert_eq!(chunks[0 as usize].amount, unbonding_info.sum());

    let (first_info, second_info) = unbonding_info.clone().partition(base_unlock_era);
    assert_eq!(1, first_info.len());
    assert_eq!(chunks[0 as usize].amount, first_info.sum());
    assert!(second_info.is_empty());

    for x in unbonding_info.len() as usize..chunks.len() {
        unbonding_info.add(chunks[x]);
        assert!(unbonding_info
            .unlocking_chunks
            .windows(2)
            .all(|w| w[0].unlock_era <= w[1].unlock_era));
    }
    assert_eq!(chunks.len(), unbonding_info.len() as usize);
    let total: Balance = chunks.iter().map(|c| c.amount).sum();
    assert_eq!(total, unbonding_info.sum());

    let partition_era = chunks[2].unlock_era + 1;
    let (first_info, second_info) = unbonding_info.clone().partition(partition_era);
    assert_eq!(3, first_info.len());
    assert_eq!(2, second_info.len());
    assert_eq!(unbonding_info.sum(), first_info.sum() + second_info.sum());
}

#[test]
fn staker_info_basic() {
    let staker_info = StakerInfo::<Balance>::default();

    assert!(staker_info.is_empty());
    assert_eq!(staker_info.len(), 0);
    assert_eq!(staker_info.latest_staked_value(), 0);
}

#[test]
fn staker_info_stake_ops() {
    let mut staker_info = StakerInfo::<Balance>::default();

    let first_era = 1;
    let first_stake = 100;
    assert_ok!(staker_info.stake(first_era, first_stake));
    assert!(!staker_info.is_empty());
    assert_eq!(staker_info.len(), 1);
    assert_eq!(staker_info.latest_staked_value(), first_stake);

    let second_era = first_era + 1;
    let second_stake = 200;
    assert_ok!(staker_info.stake(second_era, second_stake));
    assert_eq!(staker_info.len(), 2);
    assert_eq!(
        staker_info.latest_staked_value(),
        first_stake + second_stake
    );

    let third_era = second_era + 2;
    let third_stake = 333;
    assert_ok!(staker_info.stake(third_era, third_stake));
    assert_eq!(
        staker_info.latest_staked_value(),
        first_stake + second_stake + third_stake
    );
    assert_eq!(staker_info.len(), 3);

    let fourth_era = third_era;
    let fourth_stake = 444;
    assert_ok!(staker_info.stake(fourth_era, fourth_stake));
    assert_eq!(staker_info.len(), 3);
    assert_eq!(
        staker_info.latest_staked_value(),
        first_stake + second_stake + third_stake + fourth_stake
    );
}

#[test]
fn staker_info_stake_error() {
    let mut staker_info = StakerInfo::<Balance>::default();
    assert_ok!(staker_info.stake(5, 100));
    if let Err(_) = staker_info.stake(4, 100) {
    } else {
        panic!("Mustn't be able to stake with past era.");
    }
}

#[test]
fn staker_info_unstake_ops() {
    let mut staker_info = StakerInfo::<Balance>::default();

    assert!(staker_info.is_empty());
    assert_ok!(staker_info.unstake(1, 100));
    assert!(staker_info.is_empty());

    let (first_era, second_era) = (1, 3);
    let (first_stake, second_stake) = (110, 222);
    let total_staked = first_stake + second_stake;
    assert_ok!(staker_info.stake(first_era, first_stake));
    assert_ok!(staker_info.stake(second_era, second_stake));

    let first_unstake_era = second_era;
    let first_unstake = 55;
    assert_ok!(staker_info.unstake(first_unstake_era, first_unstake));
    assert_eq!(staker_info.len(), 2);
    assert_eq!(
        staker_info.latest_staked_value(),
        total_staked - first_unstake
    );
    let total_staked = total_staked - first_unstake;

    let second_unstake_era = first_unstake_era + 2;
    let second_unstake = 37;
    assert_ok!(staker_info.unstake(second_unstake_era, second_unstake));
    assert_eq!(staker_info.len(), 3);
    assert_eq!(
        staker_info.latest_staked_value(),
        total_staked - second_unstake
    );
    let total_staked = total_staked - second_unstake;

    let temp_staker_info = staker_info.clone();

    assert_ok!(staker_info.unstake(second_unstake_era, total_staked));
    assert_eq!(staker_info.len(), 3);
    assert_eq!(staker_info.latest_staked_value(), 0);

    let mut staker_info = temp_staker_info;
    assert_ok!(staker_info.unstake(second_unstake_era + 1, total_staked));
    assert_eq!(staker_info.len(), 4);
    assert_eq!(staker_info.latest_staked_value(), 0);
}

#[test]
fn stake_after_full_unstake() {
    let mut staker_info = StakerInfo::<Balance>::default();

    let first_era = 1;
    let first_stake = 100;
    assert_ok!(staker_info.stake(first_era, first_stake));
    assert_eq!(staker_info.latest_staked_value(), first_stake);

    let unstake_era = first_era + 1;
    assert_ok!(staker_info.unstake(unstake_era, first_stake));
    assert!(staker_info.latest_staked_value().is_zero());
    assert_eq!(staker_info.len(), 2);

    let restake_era = unstake_era + 2;
    let restake_value = 57;
    assert_ok!(staker_info.stake(restake_era, restake_value));
    assert_eq!(staker_info.latest_staked_value(), restake_value);
    assert_eq!(staker_info.len(), 3);
}

#[test]
fn staker_info_unstake_error() {
    let mut staker_info = StakerInfo::<Balance>::default();
    assert_ok!(staker_info.stake(5, 100));
    if let Err(_) = staker_info.unstake(4, 100) {
    } else {
        panic!("Mustn't be able to unstake with past era.");
    }
}

#[test]
fn staker_info_claim_ops_basic() {
    let mut staker_info = StakerInfo::<Balance>::default();

    assert!(staker_info.is_empty());
    assert_eq!(staker_info.claim(), (0, 0));
    assert!(staker_info.is_empty());

    assert_ok!(staker_info.stake(1, 100));
    assert_ok!(staker_info.unstake(1, 100));
    assert!(staker_info.is_empty());
    assert_eq!(staker_info.claim(), (0, 0));
    assert!(staker_info.is_empty());

    staker_info = StakerInfo::<Balance>::default();
    let stake_era = 1;
    let stake_value = 123;
    assert_ok!(staker_info.stake(stake_era, stake_value));
    assert_eq!(staker_info.len(), 1);
    assert_eq!(staker_info.claim(), (stake_era, stake_value));
    assert_eq!(staker_info.len(), 1);
}

#[test]
fn staker_info_claim_ops_advanced() {
    let mut staker_info = StakerInfo::<Balance>::default();

    let (first_stake_era, second_stake_era, third_stake_era) = (1, 2, 4);
    let (first_stake_value, second_stake_value, third_stake_value) = (123, 456, 789);

    assert_ok!(staker_info.stake(first_stake_era, first_stake_value));
    assert_ok!(staker_info.stake(second_stake_era, second_stake_value));
    assert_ok!(staker_info.stake(third_stake_era, third_stake_value));

    assert_eq!(staker_info.len(), 3);
    assert_eq!(staker_info.claim(), (first_stake_era, first_stake_value));
    assert_eq!(staker_info.len(), 2);

    assert_eq!(
        staker_info.claim(),
        (second_stake_era, first_stake_value + second_stake_value)
    );
    assert_eq!(staker_info.len(), 2);

    assert_eq!(
        staker_info.claim(),
        (3, first_stake_value + second_stake_value)
    );
    assert_eq!(staker_info.len(), 1);

    let total_staked = first_stake_value + second_stake_value + third_stake_value;
    assert_ok!(staker_info.unstake(5, total_staked));
    assert_eq!(staker_info.len(), 2);

    let fourth_era = 7;
    let fourth_stake_value = 147;
    assert_ok!(staker_info.stake(fourth_era, fourth_stake_value));
    assert_eq!(staker_info.len(), 3);

    assert_eq!(staker_info.claim(), (third_stake_era, total_staked));
    assert_eq!(staker_info.len(), 1);

    assert_eq!(staker_info.claim(), (fourth_era, fourth_stake_value));
    assert_eq!(staker_info.len(), 1);
    assert_eq!(staker_info.latest_staked_value(), fourth_stake_value);

    for x in 1..10 {
        assert_eq!(staker_info.claim(), (fourth_era + x, fourth_stake_value));
        assert_eq!(staker_info.len(), 1);
        assert_eq!(staker_info.latest_staked_value(), fourth_stake_value);
    }
}
