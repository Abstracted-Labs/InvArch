use super::pallet::{self, *};
use crate::{
    fee_handling::FeeAsset,
    origin::{ensure_multisig, INV4Origin},
    util::derive_core_account,
    voting::{Tally, Vote},
};
use core::{
    convert::{TryFrom, TryInto},
    iter::Sum,
};
use frame_support::{
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate},
        tokens::{Fortitude, Precision},
        Currency, VoteTally,
    },
    BoundedBTreeMap,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::{
    traits::{Hash, Zero},
    Perbill,
};
use sp_std::{boxed::Box, collections::btree_map::BTreeMap};

/// Maximum size of call we store is 50kb.
pub const MAX_SIZE: u32 = 50 * 1024;

pub type BoundedCallBytes<T> = BoundedVec<u8, <T as Config>::MaxCallSize>;

/// Details of a multisig operation
#[derive(Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, PartialEq, Eq)]
pub struct MultisigOperation<AccountId, TallyOf, Call, Metadata> {
    pub tally: TallyOf,
    pub original_caller: AccountId,
    pub actual_call: Call,
    pub metadata: Option<Metadata>,
    pub fee_asset: FeeAsset,
}

pub type MultisigOperationOf<T> = MultisigOperation<
    <T as frame_system::Config>::AccountId,
    Tally<T>,
    BoundedCallBytes<T>,
    BoundedVec<u8, <T as pallet::Config>::MaxMetadata>,
>;

impl<T: Config> Pallet<T>
where
    Result<
        INV4Origin<T, <T as pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: Sum,
{
    /// Mint `amount` of specified token to `target` account
    pub(crate) fn inner_token_mint(
        origin: OriginFor<T>,
        amount: BalanceOf<T>,
        target: T::AccountId,
    ) -> DispatchResult {
        let core_origin = ensure_multisig::<T, OriginFor<T>>(origin)?;
        let core_id = core_origin.id;

        T::AssetsProvider::mint_into(core_id, &target, amount)?;

        Self::deposit_event(Event::Minted {
            core_id,
            target,
            amount,
        });

        Ok(())
    }

    /// Burn `amount` of specified token from `target` account
    pub(crate) fn inner_token_burn(
        origin: OriginFor<T>,
        amount: BalanceOf<T>,
        target: T::AccountId,
    ) -> DispatchResult {
        let core_origin = ensure_multisig::<T, OriginFor<T>>(origin)?;
        let core_id = core_origin.id;

        T::AssetsProvider::burn_from(
            core_id,
            &target,
            amount,
            Precision::Exact,
            Fortitude::Polite,
        )?;

        Self::deposit_event(Event::Burned {
            core_id,
            target,
            amount,
        });

        Ok(())
    }

    /// Initiates a multisig transaction. If `caller` has enough votes, execute `call` immediately, otherwise a vote begins.
    pub(crate) fn inner_operate_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        metadata: Option<BoundedVec<u8, T::MaxMetadata>>,
        fee_asset: FeeAsset,
        call: Box<<T as Config>::RuntimeCall>,
    ) -> DispatchResultWithPostInfo {
        let owner = ensure_signed(caller)?;

        let owner_balance: BalanceOf<T> = T::AssetsProvider::balance(core_id, &owner);

        ensure!(!owner_balance.is_zero(), Error::<T>::NoPermission);

        let (minimum_support, _) = Pallet::<T>::minimum_support_and_required_approval(core_id)
            .ok_or(Error::<T>::CoreNotFound)?;

        let total_issuance: BalanceOf<T> = T::AssetsProvider::total_issuance(core_id);

        // Compute the `call` hash
        let call_hash = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&call);

        ensure!(
            Multisig::<T>::get(core_id, call_hash).is_none(),
            Error::<T>::MultisigCallAlreadyExists
        );

        // If `caller` has enough balance to meet/exeed the threshold, then go ahead and execute the `call` now.
        if Perbill::from_rational(owner_balance, total_issuance) >= minimum_support {
            let dispatch_result =
                crate::dispatch::dispatch_call::<T>(core_id, &fee_asset, *call.clone());

            Self::deposit_event(Event::MultisigExecuted {
                core_id,
                executor_account: derive_core_account::<
                    T,
                    <T as Config>::CoreId,
                    <T as frame_system::Config>::AccountId,
                >(core_id),
                voter: owner,
                call_hash,
                call: *call,
                result: dispatch_result.map(|_| ()).map_err(|e| e.error),
            });
        } else {
            let bounded_call: BoundedCallBytes<T> = (*call)
                .encode()
                .try_into()
                .map_err(|_| Error::<T>::MaxCallLengthExceeded)?;

            // Multisig call is now in the voting stage, so update storage.
            Multisig::<T>::insert(
                core_id,
                call_hash,
                MultisigOperation {
                    tally: Tally::from_parts(
                        owner_balance,
                        Zero::zero(),
                        BoundedBTreeMap::try_from(BTreeMap::from([(
                            owner.clone(),
                            Vote::Aye(owner_balance),
                        )]))
                        .map_err(|_| Error::<T>::MaxCallersExceeded)?,
                    ),
                    original_caller: owner.clone(),
                    actual_call: bounded_call,
                    metadata,
                    fee_asset,
                },
            );

            Self::deposit_event(Event::MultisigVoteStarted {
                core_id,
                executor_account: derive_core_account::<
                    T,
                    <T as Config>::CoreId,
                    <T as frame_system::Config>::AccountId,
                >(core_id),
                voter: owner,
                votes_added: Vote::Aye(owner_balance),
                call_hash,
            });
        }

        Ok(().into())
    }

    /// Vote on a multisig transaction that has not been executed yet
    pub(crate) fn inner_vote_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        call_hash: T::Hash,
        aye: bool,
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists(core_id, call_hash, |data| {
            let owner = ensure_signed(caller.clone())?;

            let voter_balance: BalanceOf<T> = T::AssetsProvider::balance(core_id, &owner);

            ensure!(!voter_balance.is_zero(), Error::<T>::NoPermission);

            let mut old_data = data.take().ok_or(Error::<T>::MultisigCallNotFound)?;

            let (minimum_support, required_approval) =
                Pallet::<T>::minimum_support_and_required_approval(core_id)
                    .ok_or(Error::<T>::CoreNotFound)?;

            let new_vote_record = if aye {
                Vote::Aye(voter_balance)
            } else {
                Vote::Nay(voter_balance)
            };

            old_data
                .tally
                .process_vote(owner.clone(), Some(new_vote_record))?;

            let support = old_data.tally.support(core_id);
            let approval = old_data.tally.approval(core_id);

            let decoded_call = <T as Config>::RuntimeCall::decode(&mut &old_data.actual_call[..])
                .map_err(|_| Error::<T>::FailedDecodingCall)?;

            if (support >= minimum_support) && (approval >= required_approval) {
                // Multisig storage records are removed when the transaction is executed or the vote on the transaction is withdrawn
                *data = None;

                // Actually dispatch this call and return the result of it
                let dispatch_result = crate::dispatch::dispatch_call::<T>(
                    core_id,
                    &old_data.fee_asset,
                    decoded_call.clone(),
                );

                Self::deposit_event(Event::MultisigExecuted {
                    core_id,
                    executor_account: derive_core_account::<
                        T,
                        <T as Config>::CoreId,
                        <T as frame_system::Config>::AccountId,
                    >(core_id),
                    voter: owner,
                    call_hash,
                    call: decoded_call,
                    result: dispatch_result.map(|_| ()).map_err(|e| e.error),
                });
            } else {
                *data = Some(old_data.clone());

                Self::deposit_event(Event::MultisigVoteAdded {
                    core_id,
                    executor_account: derive_core_account::<
                        T,
                        <T as Config>::CoreId,
                        <T as frame_system::Config>::AccountId,
                    >(core_id),
                    voter: owner,
                    votes_added: new_vote_record,
                    current_votes: old_data.tally,
                    call_hash,
                });
            }

            Ok(().into())
        })
    }

    /// Withdraw vote from an ongoing multisig operation
    pub(crate) fn inner_withdraw_vote_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        call_hash: T::Hash,
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists(core_id, call_hash, |data| {
            let owner = ensure_signed(caller.clone())?;

            let mut old_data = data.take().ok_or(Error::<T>::MultisigCallNotFound)?;

            let old_vote = old_data.tally.process_vote(owner.clone(), None)?;

            *data = Some(old_data.clone());

            Self::deposit_event(Event::MultisigVoteWithdrawn {
                core_id,
                executor_account: derive_core_account::<
                    T,
                    <T as Config>::CoreId,
                    <T as frame_system::Config>::AccountId,
                >(core_id),
                voter: owner,
                votes_removed: old_vote,
                call_hash,
            });

            Ok(().into())
        })
    }

    pub(crate) fn inner_cancel_multisig_proposal(
        origin: OriginFor<T>,
        call_hash: T::Hash,
    ) -> DispatchResultWithPostInfo {
        let core_origin = ensure_multisig::<T, OriginFor<T>>(origin)?;
        let core_id = core_origin.id;

        Multisig::<T>::remove(core_id, call_hash);

        Self::deposit_event(Event::<T>::MultisigCanceled { core_id, call_hash });

        Ok(().into())
    }

    pub fn add_member(core_id: &T::CoreId, member: &T::AccountId) {
        CoreMembers::<T>::insert(core_id, member, ())
    }

    pub fn remove_member(core_id: &T::CoreId, member: &T::AccountId) {
        CoreMembers::<T>::remove(core_id, member)
    }
}
