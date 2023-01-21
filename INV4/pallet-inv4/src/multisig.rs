use super::pallet::{self, *};
use crate::util::derive_core_account;
use core::{convert::TryInto, iter::Sum};
use frame_support::{
    dispatch::{Dispatchable, GetDispatchInfo, RawOrigin},
    pallet_prelude::*,
    traits::{Currency, WrapperKeepOpaque},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::{OneOrPercent, SubTokenInfo};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{CheckedAdd, CheckedSub};
use sp_std::{boxed::Box, vec, vec::Vec};

pub type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::RuntimeCall>;

/// Details of a multisig operation
#[derive(Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct MultisigOperation<AccountId, Signers, Call, Metadata> {
    signers: Signers,
    original_caller: AccountId,
    actual_call: Call,
    call_metadata: [u8; 2],
    call_weight: Weight,
    metadata: Option<Metadata>,
}

pub type MultisigOperationOf<T> = MultisigOperation<
    <T as frame_system::Config>::AccountId,
    BoundedVec<
        (
            <T as frame_system::Config>::AccountId,
            // Token account voted with???
            Option<<T as pallet::Config>::CoreId>,
        ),
        <T as Config>::MaxCallers,
    >,
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
        let owner = ensure_signed(caller.clone())?;

        let bounded_metadata: Option<BoundedVec<u8, T::MaxMetadata>> = if let Some(vec) = metadata {
            Some(
                vec.try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?,
            )
        } else {
            None
        };

        let total_issuance: BalanceOf<T> = TotalIssuance::<T>::iter_prefix(core_id)
            .map(|(asset, total)| {
                Some(if let Some(sub_asset) = asset {
                    // Take into account that some sub tokens have full weight while others may have partial weight or none at all
                    if let OneOrPercent::ZeroPoint(weight) =
                        Pallet::<T>::asset_weight(core_id, sub_asset)?
                    {
                        weight * total
                    } else {
                        total
                    }
                } else {
                    total
                })
            })
            .sum::<Option<BalanceOf<T>>>()
            .ok_or(Error::<T>::SubAssetNotFound)?;

        // Get minimum # of votes (tokens w/non-zero weight) required to execute a multisig call
        let total_per_threshold: BalanceOf<T> = if let OneOrPercent::ZeroPoint(percent) =
            Pallet::<T>::execution_threshold(core_id).ok_or(Error::<T>::CoreDoesntExist)?
        {
            percent * total_issuance
        } else {
            total_issuance
        };

        // Get call metadata
        let call_metadata: [u8; 2] = call
            .encode()
            .split_at(2)
            .0
            .try_into()
            .map_err(|_| Error::<T>::CallHasTooFewBytes)?;

        // Get caller balance of `ipt_id` token, weight adjusted
        let owner_balance: BalanceOf<T> = if let OneOrPercent::ZeroPoint(percent) = {
            // Function called with some sub token
            if let Some(sub_asset) = sub_token {
                ensure!(
                    Pallet::<T>::has_permission(core_id, sub_asset, call_metadata,)?,
                    Error::<T>::SubAssetHasNoPermission
                );

                Pallet::<T>::asset_weight(core_id, sub_asset).ok_or(Error::<T>::CoreDoesntExist)?
            } else {
                // Function called with IPT0 token
                OneOrPercent::One
            }
        } {
            // `ZeroPoint` sub token, so apply asset weight to caller balance
            percent
                * Balance::<T>::get((core_id, sub_token), owner.clone())
                    .ok_or(Error::<T>::NoPermission)?
        } else {
            // Either IPT0 token or 100% asset weight sub token
            Balance::<T>::get((core_id, sub_token), owner.clone())
                .ok_or(Error::<T>::NoPermission)?
        };

        let opaque_call: OpaqueCall<T> = WrapperKeepOpaque::from_encoded(call.encode());

        // Compute the `call` hash
        let call_hash: [u8; 32] = blake2_256(&call.encode());

        // Ensure that this exact `call` has not been executed before???
        ensure!(
            Multisig::<T>::get(core_id, call_hash).is_none(),
            Error::<T>::MultisigOperationAlreadyExists
        );

        // If `caller` has enough balance to meet/exeed the threshold, then go ahead and execute the `call` now.
        if owner_balance >= total_per_threshold {
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
            // Multisig call is now in the voting stage, so update storage.
            Multisig::<T>::insert(
                core_id,
                call_hash,
                MultisigOperation {
                    signers: vec![(owner.clone(), sub_token)]
                        .try_into()
                        .map_err(|_| Error::<T>::TooManySignatories)?,
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
                votes_added: owner_balance,
                votes_required: total_per_threshold,
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
        call_hash: [u8; 32],
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists(core_id, call_hash, |data| {
            let owner = ensure_signed(caller.clone())?;

            let mut old_data = data
                .take()
                .ok_or(Error::<T>::MultisigOperationUninitialized)?;

            // Get caller balance of `ipt_id` token, weight adjusted
            let voter_balance = if let OneOrPercent::ZeroPoint(percent) = {
                // Function called with some sub token
                if let Some(sub_asset) = sub_token {
                    ensure!(
                        Pallet::<T>::has_permission(core_id, sub_asset, old_data.call_metadata,)?,
                        Error::<T>::SubAssetHasNoPermission
                    );

                    Pallet::<T>::asset_weight(core_id, sub_asset)
                        .ok_or(Error::<T>::CoreDoesntExist)?
                } else {
                    // Function called with IPT0 token
                    OneOrPercent::One
                }
            } {
                percent
                    * Balance::<T>::get((core_id, sub_token), owner.clone())
                        .ok_or(Error::<T>::NoPermission)?
            } else {
                Balance::<T>::get((core_id, sub_token), owner.clone())
                    .ok_or(Error::<T>::NoPermission)?
            };

            // Get total # of votes cast so far towards this multisig call
            let total_in_operation: BalanceOf<T> = old_data
                .signers
                .clone()
                .into_iter()
                .map(|(voter, asset): (T::AccountId, Option<T::CoreId>)| {
                    Balance::<T>::get((core_id, asset), voter).map(|balance| {
                        if let OneOrPercent::ZeroPoint(percent) = if let Some(sub_asset) = asset {
                            Pallet::<T>::asset_weight(core_id, sub_asset).unwrap()
                        } else {
                            OneOrPercent::One
                        } {
                            percent * balance
                        } else {
                            balance
                        }
                    })
                })
                .collect::<Option<Vec<BalanceOf<T>>>>()
                .ok_or(Error::<T>::NoPermission)?
                .into_iter()
                .sum();

            let total_issuance: BalanceOf<T> = TotalIssuance::<T>::iter_prefix(core_id)
                .map(|(asset, total)| {
                    Some(if let Some(sub_asset) = asset {
                        // Take into account that some sub tokens have full weight while others may have partial weight or none at all
                        if let OneOrPercent::ZeroPoint(weight) =
                            Pallet::<T>::asset_weight(core_id, sub_asset)?
                        {
                            weight * total
                        } else {
                            total
                        }
                    } else {
                        total
                    })
                })
                .sum::<Option<BalanceOf<T>>>()
                .ok_or(Error::<T>::SubAssetNotFound)?;

            // Get minimum # of votes (tokens w/non-zero weight) required to execute a multisig call.
            let total_per_threshold: BalanceOf<T> = if let OneOrPercent::ZeroPoint(percent) =
                Pallet::<T>::execution_threshold(core_id).ok_or(Error::<T>::CoreDoesntExist)?
            {
                percent * total_issuance
            } else {
                total_issuance
            };

            // If already cast votes + `caller` weighted votes are enough to meet/exeed the threshold, then go ahead and execute the `call` now.
            if (total_in_operation + voter_balance) >= total_per_threshold {
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
                // Update storage
                old_data.signers = {
                    let mut v = old_data.signers.to_vec();
                    v.push((owner.clone(), sub_token));
                    v.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?
                };
                *data = Some(old_data.clone());

                Self::deposit_event(Event::MultisigVoteAdded {
                    core_id,
                    executor_account: derive_core_account::<
                        T,
                        <T as Config>::CoreId,
                        <T as frame_system::Config>::AccountId,
                    >(core_id),
                    voter: owner,
                    votes_added: voter_balance,
                    current_votes: (total_in_operation + voter_balance),
                    votes_required: total_per_threshold,
                    call_hash,
                    call: old_data.actual_call,
                });
            }

            Ok(().into())
        })
    }

    /// Withdraw vote from an ongoing multisig operation
    pub(crate) fn inner_withdraw_vote_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        sub_token: Option<T::CoreId>,
        call_hash: [u8; 32],
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists(core_id, call_hash, |data| {
            let owner = ensure_signed(caller.clone())?;

            let mut old_data = data
                .take()
                .ok_or(Error::<T>::MultisigOperationUninitialized)?;

            // Can only withdraw your vote if you have already voted on this multisig operation
            ensure!(
                old_data.signers.iter().any(|signer| signer.0 == owner),
                Error::<T>::NotAVoter
            );

            // if `caller` is the account who created this vote, they can dissolve it immediately
            if owner == old_data.original_caller {
                // Multisig storage records are removed when the transaction is executed or the vote on the transaction is withdrawn
                *data = None;

                Self::deposit_event(Event::MultisigCanceled {
                    core_id,
                    executor_account: derive_core_account::<
                        T,
                        <T as Config>::CoreId,
                        <T as frame_system::Config>::AccountId,
                    >(core_id),
                    call_hash,
                });
            } else {
                // caller is not the creator of this vote
                // Get caller balance of `ipt_id` token, weight adjusted
                let voter_balance = if let OneOrPercent::ZeroPoint(percent) = {
                    if let Some(sub_asset) = sub_token {
                        Pallet::<T>::asset_weight(core_id, sub_asset)
                            .ok_or(Error::<T>::CoreDoesntExist)?
                    } else {
                        OneOrPercent::One
                    }
                } {
                    percent
                        * Balance::<T>::get((core_id, sub_token), owner.clone())
                            .ok_or(Error::<T>::NoPermission)?
                } else {
                    Balance::<T>::get((core_id, sub_token), owner.clone())
                        .ok_or(Error::<T>::NoPermission)?
                };

                // Remove caller from the list of signers
                old_data.signers = old_data
                    .signers
                    .into_iter()
                    .filter(|signer| signer.0 != owner)
                    .collect::<Vec<(T::AccountId, Option<T::CoreId>)>>()
                    .try_into()
                    .map_err(|_| Error::<T>::TooManySignatories)?;

                *data = Some(old_data.clone());

                Self::deposit_event(Event::MultisigVoteWithdrawn {
                    core_id,
                    executor_account: derive_core_account::<
                        T,
                        <T as Config>::CoreId,
                        <T as frame_system::Config>::AccountId,
                    >(core_id),
                    voter: owner,
                    votes_removed: voter_balance,
                    call_hash,
                    call: old_data.actual_call,
                });
            }

            Ok(().into())
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
        TotalIssuance::<T>::try_mutate(core_id, token, |issuance| {
            Balance::<T>::try_mutate((core_id, token), target, |balance| -> DispatchResult {
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
        TotalIssuance::<T>::try_mutate(core_id, token, |issuance| {
            Balance::<T>::try_mutate((core_id, token), target, |balance| -> DispatchResult {
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
