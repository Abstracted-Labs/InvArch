use super::pallet::{self, *};
use crate::{
    origin::{ensure_multisig, INV4Origin},
    util::derive_core_account,
    voting::{Tally, Vote},
};
use core::{convert::TryInto, iter::Sum};
use frame_support::{
    dispatch::GetDispatchInfo,
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate},
        Currency, VoteTally, WrapperKeepOpaque,
    },
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::{
    traits::{Hash, Zero},
    Perbill, Saturating,
};
use sp_std::{boxed::Box, vec::Vec};

pub type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::RuntimeCall>;

/// Details of a multisig operation
#[derive(Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct MultisigOperation<AccountId, TallyOf, Call, Metadata> {
    pub tally: TallyOf,
    pub original_caller: AccountId,
    pub actual_call: Call,
    pub call_metadata: [u8; 2],
    pub call_weight: Weight,
    pub metadata: Option<Metadata>,
}

pub type MultisigOperationOf<T> = MultisigOperation<
    <T as frame_system::Config>::AccountId,
    Tally<T>,
    OpaqueCall<T>,
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

        T::AssetsProvider::burn_from(core_id, &target, amount)?;

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
        metadata: Option<Vec<u8>>,
        call: Box<<T as pallet::Config>::RuntimeCall>,
    ) -> DispatchResultWithPostInfo {
        let owner = ensure_signed(caller)?;

        let bounded_metadata: Option<BoundedVec<u8, T::MaxMetadata>> = if let Some(vec) = metadata {
            Some(
                vec.try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?,
            )
        } else {
            None
        };

        let (minimum_support, _) = Pallet::<T>::minimum_support_and_required_approval(core_id)
            .ok_or(Error::<T>::CoreNotFound)?;

        // Get call metadata
        let call_metadata: [u8; 2] = call
            .encode()
            .split_at(2)
            .0
            .try_into()
            .map_err(|_| Error::<T>::CallHasTooFewBytes)?;

        let owner_balance: BalanceOf<T> = T::AssetsProvider::balance(core_id, &owner);

        let total_issuance: BalanceOf<T> = T::AssetsProvider::total_issuance(core_id);

        let opaque_call: OpaqueCall<T> = WrapperKeepOpaque::from_encoded(call.encode());

        // Compute the `call` hash
        let call_hash = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&call);

        ensure!(
            Multisig::<T>::get(core_id, call_hash).is_none(),
            Error::<T>::MultisigCallAlreadyExists
        );

        // If `caller` has enough balance to meet/exeed the threshold, then go ahead and execute the `call` now.
        if Perbill::from_rational(owner_balance, total_issuance) >= minimum_support {
            let dispatch_result = crate::dispatch::dispatch_call::<T>(core_id, *call);

            Self::deposit_event(Event::MultisigExecuted {
                core_id,
                executor_account: derive_core_account::<
                    T,
                    <T as Config>::CoreId,
                    <T as frame_system::Config>::AccountId,
                >(core_id),
                voter: owner,
                call_hash,
                call: opaque_call,
                result: dispatch_result.map(|_| ()).map_err(|e| e.error),
            });
        } else {
            VoteStorage::<T>::insert(
                (core_id, call_hash, owner.clone()),
                Vote::Aye(owner_balance),
            );

            // Multisig call is now in the voting stage, so update storage.
            Multisig::<T>::insert(
                core_id,
                call_hash,
                MultisigOperation {
                    tally: Tally::from_parts(owner_balance, Zero::zero()),
                    original_caller: owner.clone(),
                    actual_call: opaque_call.clone(),
                    call_metadata,
                    call_weight: call.get_dispatch_info().weight,
                    metadata: bounded_metadata,
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
                call: opaque_call,
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

            let mut old_data = data.take().ok_or(Error::<T>::MultisigCallNotFound)?;

            VoteStorage::<T>::try_mutate((core_id, call_hash, owner.clone()), |vote_record| {
                let old_vote_record = vote_record.take();

                match old_vote_record {
                    Some(Vote::Aye(old_votes)) => {
                        old_data.tally.ayes = old_data.tally.ayes.saturating_sub(old_votes)
                    }
                    Some(Vote::Nay(old_votes)) => {
                        old_data.tally.nays = old_data.tally.nays.saturating_sub(old_votes)
                    }
                    None => (),
                }

                let voter_balance: BalanceOf<T> = T::AssetsProvider::balance(core_id, &owner);

                let (minimum_support, required_approval) =
                    Pallet::<T>::minimum_support_and_required_approval(core_id)
                        .ok_or(Error::<T>::CoreNotFound)?;

                let new_vote_record = if aye {
                    old_data.tally.ayes = old_data.tally.ayes.saturating_add(voter_balance);

                    Vote::Aye(voter_balance)
                } else {
                    old_data.tally.nays = old_data.tally.nays.saturating_add(voter_balance);

                    Vote::Nay(voter_balance)
                };

                let support = old_data.tally.support(core_id);
                let approval = old_data.tally.approval(core_id);

                if (support >= minimum_support) && (approval >= required_approval) {
                    if VoteStorage::<T>::clear_prefix(
                        (core_id, call_hash),
                        T::MaxCallers::get(),
                        None,
                    )
                    .maybe_cursor
                    .is_some()
                    {
                        Err(Error::<T>::IncompleteVoteCleanup)
                    } else {
                        Ok(())
                    }?;

                    // Multisig storage records are removed when the transaction is executed or the vote on the transaction is withdrawn
                    *data = None;

                    // Actually dispatch this call and return the result of it
                    let dispatch_result = crate::dispatch::dispatch_call::<T>(
                        core_id,
                        old_data
                            .actual_call
                            .try_decode()
                            .ok_or(Error::<T>::FailedDecodingCall)?,
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
                        call: old_data.actual_call,
                        result: dispatch_result.map(|_| ()).map_err(|e| e.error),
                    });
                } else {
                    *vote_record = Some(new_vote_record);

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
                        call: old_data.actual_call,
                    });
                }

                Ok(().into())
            })
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

            // Can only withdraw your vote if you have already voted on this multisig operation
            ensure!(
                VoteStorage::<T>::contains_key((core_id, call_hash, owner.clone())),
                Error::<T>::NotAVoter
            );

            VoteStorage::<T>::try_mutate((core_id, call_hash, owner.clone()), |vote_record| {
                let old_vote = vote_record.take().ok_or(Error::<T>::NotAVoter)?;

                match old_vote {
                    Vote::Aye(v) => old_data.tally.ayes = old_data.tally.ayes.saturating_sub(v),
                    Vote::Nay(v) => old_data.tally.nays = old_data.tally.nays.saturating_sub(v),
                };

                *vote_record = None;

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
                    call: old_data.actual_call,
                });

                Ok(().into())
            })
        })
    }
}
