use crate::{testing::mock::*, Config, Event, *};
use frame_support::assert_ok;
use pallet_inv4::util::derive_ips_account;

pub mod mock;
pub mod test;

pub(crate) struct MemorySnapshot {
    era_info: EraInfo<Balance>,
    staker_info: StakerInfo<Balance>,
    ip_stake_info: IpStakeInfo<Balance>,
    free_balance: Balance,
    ledger: AccountLedger<Balance>,
}

impl MemorySnapshot {
    pub(crate) fn all(era: EraIndex, ip: &IpId, account: AccountId) -> Self {
        Self {
            era_info: IpStaking::general_era_info(era).unwrap(),
            staker_info: GeneralStakerInfo::<Test>::get(ip, &account),
            ip_stake_info: IpStaking::ip_stake_info(ip, era).unwrap_or_default(),
            ledger: IpStaking::ledger(&account),
            free_balance: <Test as Config>::Currency::free_balance(&account),
        }
    }
}

pub(crate) fn assert_register(ip: mock::IpId) {
    let account = derive_ips_account::<Test, IpId, AccountId>(ip, None);

    let init_reserved_balance = <Test as Config>::Currency::reserved_balance(&account);

    assert!(!RegisteredIp::<Test>::contains_key(ip));

    assert_ok!(IpStaking::register_ip(
        Origin::signed(account),
        ip,
        IpMetadata {
            name: BoundedVec::default(),
            description: BoundedVec::default(),
            image: BoundedVec::default()
        }
    ));

    let ip_info = RegisteredIp::<Test>::get(ip).unwrap();
    assert_eq!(ip_info.account, account);

    let final_reserved_balance = <Test as Config>::Currency::reserved_balance(&account);
    assert_eq!(
        final_reserved_balance,
        init_reserved_balance + <Test as Config>::RegisterDeposit::get()
    );
}

pub(crate) fn assert_stake(staker: AccountId, ip: &IpId, value: Balance) {
    let current_era = IpStaking::current_era();
    let init_state = MemorySnapshot::all(current_era, &ip, staker);

    let available_for_staking = init_state.free_balance
        - init_state.ledger.locked
        - <Test as Config>::ExistentialDeposit::get();
    let staking_value = available_for_staking.min(value);

    assert_ok!(IpStaking::stake(Origin::signed(staker), ip.clone(), value));
    System::assert_last_event(mock::Event::IpStaking(Event::Staked {
        staker,
        ip: ip.clone(),
        amount: staking_value,
    }));

    let final_state = MemorySnapshot::all(current_era, &ip, staker);

    if init_state.staker_info.latest_staked_value() == 0 {
        assert!(GeneralStakerInfo::<Test>::contains_key(ip, &staker));
        assert_eq!(
            final_state.ip_stake_info.number_of_stakers,
            init_state.ip_stake_info.number_of_stakers + 1
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
        final_state.ip_stake_info.total,
        init_state.ip_stake_info.total + staking_value
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

pub(crate) fn assert_unstake(staker: AccountId, ip: &IpId, value: Balance) {
    let current_era = IpStaking::current_era();
    let init_state = MemorySnapshot::all(current_era, &ip, staker);

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

    assert_ok!(IpStaking::unstake(
        Origin::signed(staker),
        ip.clone(),
        value
    ));
    System::assert_last_event(mock::Event::IpStaking(Event::Unstaked {
        staker,
        ip: ip.clone(),
        amount: expected_unbond_amount,
    }));

    let final_state = MemorySnapshot::all(current_era, &ip, staker);
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
        init_state.ip_stake_info.total - expected_unbond_amount,
        final_state.ip_stake_info.total
    );
    assert_eq!(
        init_state.staker_info.latest_staked_value() - expected_unbond_amount,
        final_state.staker_info.latest_staked_value()
    );

    let delta = if remaining_staked > 0 { 0 } else { 1 };
    assert_eq!(
        init_state.ip_stake_info.number_of_stakers - delta,
        final_state.ip_stake_info.number_of_stakers
    );

    assert_eq!(
        init_state.era_info.staked - expected_unbond_amount,
        final_state.era_info.staked
    );
    assert_eq!(init_state.era_info.locked, final_state.era_info.locked);
}

pub(crate) fn assert_withdraw_unbonded(staker: AccountId) {
    let current_era = IpStaking::current_era();

    let init_era_info = GeneralEraInfo::<Test>::get(current_era).unwrap();
    let init_ledger = Ledger::<Test>::get(&staker);

    let (valid_info, remaining_info) = init_ledger.unbonding_info.partition(current_era);
    let expected_unbond_amount = valid_info.sum();

    assert_ok!(IpStaking::withdraw_unstaked(Origin::signed(staker),));
    System::assert_last_event(mock::Event::IpStaking(Event::Withdrawn {
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

pub(crate) fn assert_unregister(ip: IpId) {
    let init_reserved_balance = <Test as Config>::Currency::reserved_balance(&account(ip));

    assert_ok!(IpStaking::unregister_ip(
        Origin::signed(account(ip)),
        ip.clone()
    ));
    System::assert_last_event(mock::Event::IpStaking(Event::IpUnregistered { ip }));

    let final_reserved_balance = <Test as Config>::Currency::reserved_balance(&account(ip));
    assert_eq!(
        final_reserved_balance,
        init_reserved_balance - <Test as Config>::RegisterDeposit::get()
    );
}

pub(crate) fn assert_claim_staker(claimer: AccountId, ip: IpId) {
    let (claim_era, _) = IpStaking::staker_info(ip, &claimer).claim();
    let current_era = IpStaking::current_era();

    System::reset_events();

    let init_state_claim_era = MemorySnapshot::all(claim_era, &ip, claimer);
    let init_state_current_era = MemorySnapshot::all(current_era, &ip, claimer);

    let (_, stakers_joint_reward) = IpStaking::ip_stakers_split(
        &init_state_claim_era.ip_stake_info,
        &init_state_claim_era.era_info,
    );

    let (claim_era, staked) = init_state_claim_era.staker_info.clone().claim();

    let calculated_reward =
        Perbill::from_rational(staked, init_state_claim_era.ip_stake_info.total)
            * stakers_joint_reward;
    let issuance_before_claim = <Test as Config>::Currency::total_issuance();

    assert_ok!(IpStaking::staker_claim_rewards(Origin::signed(claimer), ip));

    let final_state_current_era = MemorySnapshot::all(current_era, &ip, claimer);

    assert_reward(
        &init_state_current_era,
        &final_state_current_era,
        calculated_reward,
    );

    System::assert_last_event(mock::Event::IpStaking(Event::StakerClaimed {
        staker: claimer,
        ip,
        era: claim_era,
        amount: calculated_reward,
    }));

    let (new_era, _) = final_state_current_era.staker_info.clone().claim();
    if final_state_current_era.staker_info.is_empty() {
        assert!(new_era.is_zero());
        assert!(!GeneralStakerInfo::<Test>::contains_key(ip, &claimer));
    } else {
        assert!(new_era > claim_era);
    }
    assert!(new_era.is_zero() || new_era > claim_era);

    let issuance_after_claim = <Test as Config>::Currency::total_issuance();
    assert_eq!(issuance_before_claim, issuance_after_claim);

    let final_state_claim_era = MemorySnapshot::all(claim_era, &ip, claimer);
    assert_eq!(
        init_state_claim_era.ip_stake_info,
        final_state_claim_era.ip_stake_info
    );
}

pub(crate) fn assert_claim_ip(ip: IpId, claim_era: EraIndex) {
    let init_state = MemorySnapshot::all(claim_era, &ip, account(ip));
    assert!(!init_state.ip_stake_info.reward_claimed);

    let (calculated_reward, _) =
        IpStaking::ip_stakers_split(&init_state.ip_stake_info, &init_state.era_info);

    assert_ok!(IpStaking::ip_claim_rewards(
        Origin::signed(account(ip)),
        ip,
        claim_era,
    ));
    System::assert_last_event(mock::Event::IpStaking(Event::IpClaimed {
        ip,
        destination_account: account(ip),
        era: claim_era,
        amount: calculated_reward,
    }));

    let final_state = MemorySnapshot::all(claim_era, &ip, account(ip));
    assert_eq!(
        init_state.free_balance + calculated_reward,
        final_state.free_balance
    );

    assert!(final_state.ip_stake_info.reward_claimed);

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
        init_state_current_era.ip_stake_info,
        final_state_current_era.ip_stake_info
    );
}
