use crate::{testing::mock::*, Config, Event, *};
use frame_support::assert_ok;

pub mod mock;
pub mod test;

pub(crate) struct MemorySnapshot {
    era_info: EraInfo<Balance>,
    staker_info: StakerInfo<Balance>,
    core_stake_info: CoreStakeInfo<Balance>,
    free_balance: Balance,
    ledger: AccountLedger<Balance>,
}

impl MemorySnapshot {
    pub(crate) fn all(era: EraIndex, core: &CoreId, account: AccountId) -> Self {
        Self {
            era_info: OcifStaking::general_era_info(era).unwrap(),
            staker_info: GeneralStakerInfo::<Test>::get(core, &account),
            core_stake_info: OcifStaking::core_stake_info(core, era).unwrap_or_default(),
            ledger: OcifStaking::ledger(&account),
            free_balance: <Test as Config>::Currency::free_balance(&account),
        }
    }
}

pub(crate) fn assert_register(core: mock::CoreId) {
    let account = pallet_inv4::util::derive_core_account::<Test, CoreId, AccountId>(core);

    let init_reserved_balance = <Test as Config>::Currency::reserved_balance(&account);

    assert!(!RegisteredCore::<Test>::contains_key(core));

    assert_ok!(OcifStaking::register_core(
        pallet_inv4::Origin::Multisig(pallet_inv4::origin::MultisigInternalOrigin::new(core))
            .into(),
        vec![].try_into().unwrap(),
        vec![].try_into().unwrap(),
        vec![].try_into().unwrap()
    ));

    let core_info = RegisteredCore::<Test>::get(core).unwrap();
    assert_eq!(core_info.account, account);

    let final_reserved_balance = <Test as Config>::Currency::reserved_balance(&account);
    assert_eq!(
        final_reserved_balance,
        init_reserved_balance + <Test as Config>::RegisterDeposit::get()
    );
}

pub(crate) fn assert_stake(staker: AccountId, core: &CoreId, value: Balance) {
    let current_era = OcifStaking::current_era();
    let init_state = MemorySnapshot::all(current_era, &core, staker);

    let available_for_staking = init_state.free_balance
        - init_state.ledger.locked
        - <Test as Config>::ExistentialDeposit::get();
    let staking_value = available_for_staking.min(value);

    assert_ok!(OcifStaking::stake(
        RuntimeOrigin::signed(staker),
        core.clone(),
        value
    ));
    System::assert_last_event(mock::RuntimeEvent::OcifStaking(Event::Staked {
        staker,
        core: core.clone(),
        amount: staking_value,
    }));

    let final_state = MemorySnapshot::all(current_era, &core, staker);

    if init_state.staker_info.latest_staked_value() == 0 {
        assert!(GeneralStakerInfo::<Test>::contains_key(core, &staker));
        assert_eq!(
            final_state.core_stake_info.number_of_stakers,
            init_state.core_stake_info.number_of_stakers + 1
        );
    }

    assert_eq!(
        final_state.era_info.staked,
        init_state.era_info.staked + staking_value
    );
    assert_eq!(
        final_state.era_info.locked,
        init_state.era_info.locked + staking_value
    );
    assert_eq!(
        final_state.core_stake_info.total,
        init_state.core_stake_info.total + staking_value
    );
    assert_eq!(
        final_state.staker_info.latest_staked_value(),
        init_state.staker_info.latest_staked_value() + staking_value
    );
    assert_eq!(
        final_state.ledger.locked,
        init_state.ledger.locked + staking_value
    );
}

pub(crate) fn assert_unstake(staker: AccountId, core: &CoreId, value: Balance) {
    let current_era = OcifStaking::current_era();
    let init_state = MemorySnapshot::all(current_era, &core, staker);

    let remaining_staked = init_state
        .staker_info
        .latest_staked_value()
        .saturating_sub(value);
    let expected_unbond_amount = if remaining_staked < MINIMUM_STAKING_AMOUNT {
        init_state.staker_info.latest_staked_value()
    } else {
        value
    };
    let remaining_staked = init_state.staker_info.latest_staked_value() - expected_unbond_amount;

    assert_ok!(OcifStaking::unstake(
        RuntimeOrigin::signed(staker),
        core.clone(),
        value
    ));
    System::assert_last_event(mock::RuntimeEvent::OcifStaking(Event::Unstaked {
        staker,
        core: core.clone(),
        amount: expected_unbond_amount,
    }));

    let final_state = MemorySnapshot::all(current_era, &core, staker);
    let expected_unlock_era = current_era + UNBONDING_PERIOD;
    match init_state
        .ledger
        .unbonding_info
        .unlocking_chunks
        .binary_search_by(|x| x.unlock_era.cmp(&expected_unlock_era))
    {
        Ok(_) => assert_eq!(
            init_state.ledger.unbonding_info.len(),
            final_state.ledger.unbonding_info.len()
        ),
        Err(_) => assert_eq!(
            init_state.ledger.unbonding_info.len() + 1,
            final_state.ledger.unbonding_info.len()
        ),
    }
    assert_eq!(
        init_state.ledger.unbonding_info.sum() + expected_unbond_amount,
        final_state.ledger.unbonding_info.sum()
    );

    let mut unbonding_info = init_state.ledger.unbonding_info.clone();
    unbonding_info.add(UnlockingChunk {
        amount: expected_unbond_amount,
        unlock_era: current_era + UNBONDING_PERIOD,
    });
    assert_eq!(unbonding_info, final_state.ledger.unbonding_info);

    assert_eq!(init_state.ledger.locked, final_state.ledger.locked);
    if final_state.ledger.is_empty() {
        assert!(!Ledger::<Test>::contains_key(&staker));
    }

    assert_eq!(
        init_state.core_stake_info.total - expected_unbond_amount,
        final_state.core_stake_info.total
    );
    assert_eq!(
        init_state.staker_info.latest_staked_value() - expected_unbond_amount,
        final_state.staker_info.latest_staked_value()
    );

    let delta = if remaining_staked > 0 { 0 } else { 1 };
    assert_eq!(
        init_state.core_stake_info.number_of_stakers - delta,
        final_state.core_stake_info.number_of_stakers
    );

    assert_eq!(
        init_state.era_info.staked - expected_unbond_amount,
        final_state.era_info.staked
    );
    assert_eq!(init_state.era_info.locked, final_state.era_info.locked);
}

pub(crate) fn assert_withdraw_unbonded(staker: AccountId) {
    let current_era = OcifStaking::current_era();

    let init_era_info = GeneralEraInfo::<Test>::get(current_era).unwrap();
    let init_ledger = Ledger::<Test>::get(&staker);

    let (valid_info, remaining_info) = init_ledger.unbonding_info.partition(current_era);
    let expected_unbond_amount = valid_info.sum();

    assert_ok!(OcifStaking::withdraw_unstaked(RuntimeOrigin::signed(
        staker
    ),));
    System::assert_last_event(mock::RuntimeEvent::OcifStaking(Event::Withdrawn {
        staker,
        amount: expected_unbond_amount,
    }));

    let final_ledger = Ledger::<Test>::get(&staker);
    assert_eq!(remaining_info, final_ledger.unbonding_info);
    if final_ledger.unbonding_info.is_empty() && final_ledger.locked == 0 {
        assert!(!Ledger::<Test>::contains_key(&staker));
    }

    let final_rewards_and_stakes = GeneralEraInfo::<Test>::get(current_era).unwrap();
    assert_eq!(final_rewards_and_stakes.staked, init_era_info.staked);
    assert_eq!(
        final_rewards_and_stakes.locked,
        init_era_info.locked - expected_unbond_amount
    );
    assert_eq!(
        final_ledger.locked,
        init_ledger.locked - expected_unbond_amount
    );
}

pub(crate) fn assert_unregister(core: CoreId) {
    let init_reserved_balance = <Test as Config>::Currency::reserved_balance(&account(core));

    assert_ok!(OcifStaking::unregister_core(
        pallet_inv4::Origin::Multisig(pallet_inv4::origin::MultisigInternalOrigin::new(core))
            .into()
    ));
    System::assert_last_event(mock::RuntimeEvent::OcifStaking(Event::CoreUnregistered {
        core,
    }));

    let final_reserved_balance = <Test as Config>::Currency::reserved_balance(&account(core));
    assert_eq!(
        final_reserved_balance,
        init_reserved_balance - <Test as Config>::RegisterDeposit::get()
    );
}

pub(crate) fn assert_claim_staker(claimer: AccountId, core: CoreId) {
    let (claim_era, _) = OcifStaking::staker_info(core, &claimer).claim();
    let current_era = OcifStaking::current_era();

    System::reset_events();

    let init_state_claim_era = MemorySnapshot::all(claim_era, &core, claimer);
    let init_state_current_era = MemorySnapshot::all(current_era, &core, claimer);

    let (_, stakers_joint_reward) = OcifStaking::core_stakers_split(
        &init_state_claim_era.core_stake_info,
        &init_state_claim_era.era_info,
    );

    let (claim_era, staked) = init_state_claim_era.staker_info.clone().claim();

    let calculated_reward =
        Perbill::from_rational(staked, init_state_claim_era.core_stake_info.total)
            * stakers_joint_reward;
    let issuance_before_claim = <Test as Config>::Currency::total_issuance();

    assert_ok!(OcifStaking::staker_claim_rewards(
        RuntimeOrigin::signed(claimer),
        core
    ));

    let final_state_current_era = MemorySnapshot::all(current_era, &core, claimer);

    assert_reward(
        &init_state_current_era,
        &final_state_current_era,
        calculated_reward,
    );

    System::assert_last_event(mock::RuntimeEvent::OcifStaking(Event::StakerClaimed {
        staker: claimer,
        core,
        era: claim_era,
        amount: calculated_reward,
    }));

    let (new_era, _) = final_state_current_era.staker_info.clone().claim();
    if final_state_current_era.staker_info.is_empty() {
        assert!(new_era.is_zero());
        assert!(!GeneralStakerInfo::<Test>::contains_key(core, &claimer));
    } else {
        assert!(new_era > claim_era);
    }
    assert!(new_era.is_zero() || new_era > claim_era);

    let issuance_after_claim = <Test as Config>::Currency::total_issuance();
    assert_eq!(issuance_before_claim, issuance_after_claim);

    let final_state_claim_era = MemorySnapshot::all(claim_era, &core, claimer);
    assert_eq!(
        init_state_claim_era.core_stake_info,
        final_state_claim_era.core_stake_info
    );
}

pub(crate) fn assert_claim_core(core: CoreId, claim_era: EraIndex) {
    let init_state = MemorySnapshot::all(claim_era, &core, account(core));
    assert!(!init_state.core_stake_info.reward_claimed);

    let (calculated_reward, _) =
        OcifStaking::core_stakers_split(&init_state.core_stake_info, &init_state.era_info);

    assert_ok!(OcifStaking::core_claim_rewards(
        RuntimeOrigin::signed(account(core)),
        core,
        claim_era,
    ));
    System::assert_last_event(mock::RuntimeEvent::OcifStaking(Event::CoreClaimed {
        core,
        destination_account: account(core),
        era: claim_era,
        amount: calculated_reward,
    }));

    let final_state = MemorySnapshot::all(claim_era, &core, account(core));
    assert_eq!(
        init_state.free_balance + calculated_reward,
        final_state.free_balance
    );

    assert!(final_state.core_stake_info.reward_claimed);

    assert_eq!(init_state.staker_info, final_state.staker_info);
    assert_eq!(init_state.ledger, final_state.ledger);
}

fn assert_reward(
    init_state_current_era: &MemorySnapshot,
    final_state_current_era: &MemorySnapshot,
    reward: Balance,
) {
    assert_eq!(
        init_state_current_era.free_balance + reward,
        final_state_current_era.free_balance
    );
    assert_eq!(
        init_state_current_era.era_info.staked,
        final_state_current_era.era_info.staked
    );
    assert_eq!(
        init_state_current_era.era_info.locked,
        final_state_current_era.era_info.locked
    );
    assert_eq!(
        init_state_current_era.core_stake_info,
        final_state_current_era.core_stake_info
    );
}

pub(crate) fn assert_move_stake(
    staker: AccountId,
    from_core: &CoreId,
    to_core: &CoreId,
    amount: Balance,
) {
    let current_era = OcifStaking::current_era();
    let from_init_state = MemorySnapshot::all(current_era, &from_core, staker);
    let to_init_state = MemorySnapshot::all(current_era, &to_core, staker);

    let init_staked_value = from_init_state.staker_info.latest_staked_value();
    let expected_transfer_amount = if init_staked_value - amount >= MINIMUM_STAKING_AMOUNT {
        amount
    } else {
        init_staked_value
    };

    assert_ok!(OcifStaking::move_stake(
        RuntimeOrigin::signed(staker),
        from_core.clone(),
        amount,
        to_core.clone()
    ));
    System::assert_last_event(mock::RuntimeEvent::OcifStaking(Event::StakeMoved {
        staker,
        from_core: from_core.clone(),
        amount: expected_transfer_amount,
        to_core: to_core.clone(),
    }));

    let from_final_state = MemorySnapshot::all(current_era, &from_core, staker);
    let to_final_state = MemorySnapshot::all(current_era, &to_core, staker);

    assert_eq!(
        from_final_state.staker_info.latest_staked_value(),
        init_staked_value - expected_transfer_amount
    );
    assert_eq!(
        to_final_state.staker_info.latest_staked_value(),
        to_init_state.staker_info.latest_staked_value() + expected_transfer_amount
    );

    assert_eq!(
        from_final_state.core_stake_info.total,
        from_init_state.core_stake_info.total - expected_transfer_amount
    );
    assert_eq!(
        to_final_state.core_stake_info.total,
        to_init_state.core_stake_info.total + expected_transfer_amount
    );

    let from_core_fully_unstaked = init_staked_value == expected_transfer_amount;
    if from_core_fully_unstaked {
        assert_eq!(
            from_final_state.core_stake_info.number_of_stakers + 1,
            from_init_state.core_stake_info.number_of_stakers
        );
    }

    let no_init_stake_on_to_core = to_init_state.staker_info.latest_staked_value().is_zero();
    if no_init_stake_on_to_core {
        assert_eq!(
            to_final_state.core_stake_info.number_of_stakers,
            to_init_state.core_stake_info.number_of_stakers + 1
        );
    }

    let fully_unstaked_and_nothing_to_claim =
        from_core_fully_unstaked && to_final_state.staker_info.clone().claim() == (0, 0);
    if fully_unstaked_and_nothing_to_claim {
        assert!(!GeneralStakerInfo::<Test>::contains_key(&to_core, &staker));
    }
}
