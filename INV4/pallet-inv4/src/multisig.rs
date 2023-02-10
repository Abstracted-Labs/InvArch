use super::pallet::{self, *};
use crate::{
    util::derive_core_account,
    voting::{Tally, Vote},
};
use core::{convert::TryInto, iter::Sum};
use frame_support::{
    dispatch::{Dispatchable, GetDispatchInfo, RawOrigin},
    pallet_prelude::*,
    traits::{Currency, VoteTally, WrapperKeepOpaque},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::SubTokenInfo;
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, Hash, Zero},
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
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: Sum,
{
    /// Mint `amount` of specified token to `target` account
    pub(crate) fn inner_token_mint(
        owner: OriginFor<T>,
        core_id: T::CoreId,
        token: Option<T::CoreId>,
        amount: BalanceOf<T>,
        target: T::AccountId,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        // IP Set must exist for there to be a token
        let core = CoreStorage::<T>::get(core_id).ok_or(Error::<T>::CoreDoesntExist)?;

        ensure!(core.account == owner, Error::<T>::NoPermission);

        // If trying to mint more of a sub token, token must already exist
        if let Some(sub_asset) = token {
            ensure!(
                SubAssets::<T>::get(core_id, sub_asset).is_some(),
                Error::<T>::SubAssetNotFound
            );
        }

        // Actually mint tokens
        Pallet::<T>::internal_mint(core_id, token, target.clone(), amount)?;

        Self::deposit_event(Event::Minted {
            token: (core_id, token),
            target,
            amount,
        });

        Ok(())
    }

    /// Burn `amount` of specified token from `target` account
    pub(crate) fn inner_token_burn(
        owner: OriginFor<T>,
        core_id: T::CoreId,
        token: Option<T::CoreId>,
        amount: BalanceOf<T>,
        target: T::AccountId,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        // IP Set must exist for their to be a token
        let core = CoreStorage::<T>::get(core_id).ok_or(Error::<T>::CoreDoesntExist)?;

        ensure!(core.account == owner, Error::<T>::NoPermission);

        // If trying to burn sub tokens, token must already exist
        if let Some(sub_asset) = token {
            ensure!(
                SubAssets::<T>::get(core_id, sub_asset).is_some(),
                Error::<T>::SubAssetNotFound
            );
        }

        // Actually burn tokens
        Pallet::<T>::internal_burn(target.clone(), core_id, token, amount)?;

        Self::deposit_event(Event::Burned {
            token: (core_id, token),
            target,
            amount,
        });

        Ok(())
    }

    /// Initiates a multisig transaction. If `caller` has enough votes, execute `call` immediately, otherwise a vote begins.
    pub(crate) fn inner_operate_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        sub_token: Option<T::CoreId>,
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
            .ok_or(Error::<T>::CoreDoesntExist)?;

        // Get call metadata
        let call_metadata: [u8; 2] = call
            .encode()
            .split_at(2)
            .0
            .try_into()
            .map_err(|_| Error::<T>::CallHasTooFewBytes)?;

        // Get caller balance of `ipt_id` token, weight adjusted
        let owner_balance: BalanceOf<T> = {
            if let Some(sub_asset) = sub_token {
                ensure!(
                    Pallet::<T>::has_permission(core_id, sub_asset, call_metadata,)?,
                    Error::<T>::SubAssetHasNoPermission
                );
            }

            Balances::<T>::get((core_id, sub_token, owner.clone()))
                .ok_or(Error::<T>::NoPermission)?
        };

        let opaque_call: OpaqueCall<T> = WrapperKeepOpaque::from_encoded(call.encode());

        // Compute the `call` hash
        let call_hash = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&call);

        ensure!(
            Multisig::<T>::get(core_id, call_hash).is_none(),
            Error::<T>::MultisigOperationAlreadyExists
        );

        // If `caller` has enough balance to meet/exeed the threshold, then go ahead and execute the `call` now.
        if Perbill::from_rational(owner_balance, TotalIssuance::<T>::get(core_id))
            >= minimum_support
        {
            // Actually dispatch this call and return the result of it
            let dispatch_result = call.dispatch(
                RawOrigin::Signed(derive_core_account::<
                    T,
                    <T as Config>::CoreId,
                    <T as frame_system::Config>::AccountId,
                >(core_id))
                .into(),
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
                call: opaque_call,
                result: dispatch_result.map(|_| ()).map_err(|e| e.error),
            });
        } else {
            VoteStorage::<T>::insert(
                (core_id, call_hash, (owner.clone(), sub_token)),
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
        sub_token: Option<T::CoreId>,
        call_hash: T::Hash,
        aye: bool,
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists(core_id, call_hash, |data| {
            let owner = ensure_signed(caller.clone())?;

            let mut old_data = data
                .take()
                .ok_or(Error::<T>::MultisigOperationUninitialized)?;

            VoteStorage::<T>::try_mutate(
                (core_id, call_hash, (owner.clone(), sub_token)),
                |vote_record| {
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

                    // Get caller balance of `sub_token` token
                    let voter_balance = {
                        if let Some(sub_asset) = sub_token {
                            ensure!(
                                Pallet::<T>::has_permission(
                                    core_id,
                                    sub_asset,
                                    old_data.call_metadata
                                )?,
                                Error::<T>::SubAssetHasNoPermission
                            );
                        }

                        Balances::<T>::get((core_id, sub_token, owner.clone()))
                            .ok_or(Error::<T>::NoPermission)?
                    };

                    let (minimum_support, required_approval) =
                        Pallet::<T>::minimum_support_and_required_approval(core_id)
                            .ok_or(Error::<T>::CoreDoesntExist)?;

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
                        let dispatch_result = old_data
                            .actual_call
                            .try_decode()
                            .ok_or(Error::<T>::CouldntDecodeCall)?
                            .dispatch(
                                RawOrigin::Signed(derive_core_account::<
                                    T,
                                    <T as Config>::CoreId,
                                    <T as frame_system::Config>::AccountId,
                                >(core_id))
                                .into(),
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
                },
            )
        })
    }

    /// Withdraw vote from an ongoing multisig operation
    pub(crate) fn inner_withdraw_vote_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        sub_token: Option<T::CoreId>,
        call_hash: T::Hash,
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists(core_id, call_hash, |data| {
            let owner = ensure_signed(caller.clone())?;

            let mut old_data = data
                .take()
                .ok_or(Error::<T>::MultisigOperationUninitialized)?;

            // Can only withdraw your vote if you have already voted on this multisig operation
            ensure!(
                VoteStorage::<T>::contains_key((core_id, call_hash, (owner.clone(), sub_token))),
                Error::<T>::NotAVoter
            );

            VoteStorage::<T>::try_mutate(
                (core_id, call_hash, (owner.clone(), sub_token)),
                |vote_record| {
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
                },
            )
        })
    }

    /// Create one or more sub tokens for an IP Set
    pub(crate) fn inner_create_sub_token(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        sub_token_id: T::CoreId,
        sub_token_metadata: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        CoreStorage::<T>::try_mutate_exists(core_id, |ipt| -> DispatchResultWithPostInfo {
            let caller = ensure_signed(caller.clone())?;

            let old_ipt = ipt.clone().ok_or(Error::<T>::CoreDoesntExist)?;

            ensure!(old_ipt.account == caller, Error::<T>::NoPermission);

            let metadata: BoundedVec<u8, T::MaxMetadata> = sub_token_metadata
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

            ensure!(
                !SubAssets::<T>::contains_key(core_id, sub_token_id),
                Error::<T>::SubAssetAlreadyExists
            );

            let sub_token_info = SubTokenInfo {
                id: sub_token_id,
                metadata,
            };

            SubAssets::<T>::insert(core_id, sub_token_id, sub_token_info);

            Self::deposit_event(Event::SubTokenCreated {
                id: sub_token_id,
                metadata: sub_token_metadata,
            });

            Ok(().into())
        })
    }

    /// Mint `amount` of specified token to `target` account
    pub fn internal_mint(
        core_id: T::CoreId,
        token: Option<T::CoreId>,
        target: T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        TotalIssuance::<T>::try_mutate(core_id, |issuance| {
            Balances::<T>::try_mutate((core_id, token, target), |balance| -> DispatchResult {
                let old_balance = balance.take().unwrap_or_default();
                // Increase `target` account's balance of `ipt_id` sub token by `amount`
                *balance = Some(
                    old_balance
                        .checked_add(&amount)
                        .ok_or(Error::<T>::Overflow)?,
                );

                *issuance = issuance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;

                Ok(())
            })
        })
    }

    /// Burn `amount` of specified token from `target` account
    pub fn internal_burn(
        target: T::AccountId,
        core_id: T::CoreId,
        token: Option<T::CoreId>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        TotalIssuance::<T>::try_mutate(core_id, |issuance| {
            Balances::<T>::try_mutate((core_id, token, target), |balance| -> DispatchResult {
                let old_balance = balance.take().ok_or(Error::<T>::CoreDoesntExist)?;
                // Decrease `target` account's balance of `ipt_id` sub token by `amount`
                *balance = Some(
                    old_balance
                        .checked_sub(&amount)
                        .ok_or(Error::<T>::Underflow)?,
                );

                *issuance = issuance.checked_sub(&amount).ok_or(Error::<T>::Underflow)?;

                Ok(())
            })
        })
    }
}
