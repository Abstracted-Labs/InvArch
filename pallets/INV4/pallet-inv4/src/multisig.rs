//! Multisig Operations.
//!
//! ## Overview
//!
//! Handles the core actions within an already established multisig.
//!
//! ### Core functionalities:
//! - Minting/Burning voting tokens to existing and new members.
//! - Handling proposal votes.
//! - Dispatching approved proposals when both support and approval meet/exceed their minimum required thresholds.
//! - Canceling proposals.

use super::pallet::{self, *};
use crate::{
    account_derivation::CoreAccountDerivation,
    fee_handling::{FeeAsset, FeeAssetNegativeImbalance, MultisigFeeHandler},
    origin::{ensure_multisig, INV4Origin},
    voting::{Tally, Vote},
};
use codec::DecodeLimit;
use core::{
    convert::{TryFrom, TryInto},
    iter::Sum,
};
use frame_support::{
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate},
        tokens::{Fortitude, Precision},
        Currency, ExistenceRequirement, VoteTally, WithdrawReasons,
    },
    weights::WeightToFee,
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

/// Details of a multisig operation.
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
    Result<INV4Origin<T>, <T as frame_system::Config>::RuntimeOrigin>:
        From<<T as frame_system::Config>::RuntimeOrigin>,
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: Sum,
    <T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
    /// Inner function for the token_mint call.
    pub(crate) fn inner_token_mint(
        origin: OriginFor<T>,
        amount: BalanceOf<T>,
        target: T::AccountId,
    ) -> DispatchResult {
        // Grab the core id from the origin
        let core_origin = ensure_multisig::<T, OriginFor<T>>(origin)?;
        let core_id = core_origin.id;

        // Mint the core's voting token to the target.
        T::AssetsProvider::mint_into(core_id, &target, amount)?;

        Self::deposit_event(Event::Minted {
            core_id,
            target,
            amount,
        });

        Ok(())
    }

    /// Inner function for the token_burn call.
    pub(crate) fn inner_token_burn(
        origin: OriginFor<T>,
        amount: BalanceOf<T>,
        target: T::AccountId,
    ) -> DispatchResult {
        // Grab the core id from the origin
        let core_origin = ensure_multisig::<T, OriginFor<T>>(origin)?;
        let core_id = core_origin.id;

        // Burn the core's voting token from the target.
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

    /// Inner function for the operate_multisig call.
    pub(crate) fn inner_operate_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        metadata: Option<BoundedVec<u8, T::MaxMetadata>>,
        fee_asset: FeeAsset,
        call: Box<<T as Config>::RuntimeCall>,
    ) -> DispatchResultWithPostInfo {
        let owner = ensure_signed(caller)?;

        // Get the voting token balance of the caller
        let owner_balance: BalanceOf<T> = T::AssetsProvider::balance(core_id, &owner);

        ensure!(!owner_balance.is_zero(), Error::<T>::NoPermission);

        // Get the minimum support value of the target core
        let (minimum_support, _) = Pallet::<T>::minimum_support_and_required_approval(core_id)
            .ok_or(Error::<T>::CoreNotFound)?;

        // Get the total issuance of the core's voting token
        let total_issuance: BalanceOf<T> = T::AssetsProvider::total_issuance(core_id);

        // Compute the call hash
        let call_hash = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&call);

        // Make sure this exact multisig call doesn't already exist
        ensure!(
            Multisig::<T>::get(core_id, call_hash).is_none(),
            Error::<T>::MultisigCallAlreadyExists
        );

        // If caller has enough balance to meet/exeed the threshold, then go ahead and execute the call now
        // There is no need to check against required_approval as it's assumed the caller is voting aye
        if Perbill::from_rational(owner_balance, total_issuance) >= minimum_support {
            let dispatch_result =
                crate::dispatch::dispatch_call::<T>(core_id, &fee_asset, *call.clone());

            Self::deposit_event(Event::MultisigExecuted {
                core_id,
                executor_account: Self::derive_core_account(core_id),
                voter: owner,
                call_hash,
                call: *call,
                result: dispatch_result.map(|_| ()).map_err(|e| e.error),
            });
        } else {
            // Wrap the call making sure it fits the size boundary
            let bounded_call: BoundedCallBytes<T> = (*call)
                .encode()
                .try_into()
                .map_err(|_| Error::<T>::MaxCallLengthExceeded)?;

            let total_lenght = (bounded_call.len() as u64)
                .saturating_add(metadata.clone().unwrap_or_default().len() as u64);

            let storage_cost: BalanceOf<T> =
                T::LengthToFee::weight_to_fee(&Weight::from_parts(total_lenght as u64, 0));

            T::FeeCharger::handle_creation_fee(FeeAssetNegativeImbalance::Native(
                <T as Config>::Currency::withdraw(
                    &owner,
                    storage_cost,
                    WithdrawReasons::TRANSACTION_PAYMENT,
                    ExistenceRequirement::KeepAlive,
                )?,
            ));

            // Insert proposal in storage, it's now in the voting stage
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
                executor_account: Self::derive_core_account(core_id),
                voter: owner,
                votes_added: Vote::Aye(owner_balance),
                call_hash,
            });
        }

        Ok(().into())
    }

    /// Inner function for the vote_multisig call.
    pub(crate) fn inner_vote_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        call_hash: T::Hash,
        aye: bool,
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists(core_id, call_hash, |data| {
            let owner = ensure_signed(caller.clone())?;

            // Get the voting token balance of the caller
            let voter_balance: BalanceOf<T> = T::AssetsProvider::balance(core_id, &owner);

            // If caller doesn't own the token, they have no voting power.
            ensure!(!voter_balance.is_zero(), Error::<T>::NoPermission);

            // Get the multisig call data from the storage
            let mut old_data = data.take().ok_or(Error::<T>::MultisigCallNotFound)?;

            // Get the minimum support and required approval values of the target core
            let (minimum_support, required_approval) =
                Pallet::<T>::minimum_support_and_required_approval(core_id)
                    .ok_or(Error::<T>::CoreNotFound)?;

            let new_vote_record = if aye {
                Vote::Aye(voter_balance)
            } else {
                Vote::Nay(voter_balance)
            };

            // Mutate tally with the new vote
            old_data
                .tally
                .process_vote(owner.clone(), Some(new_vote_record))?;

            let support = old_data.tally.support(core_id);
            let approval = old_data.tally.approval(core_id);

            // Check if the multisig proposal passes the thresholds with the added vote
            if (support >= minimum_support) && (approval >= required_approval) {
                // Decode the call
                let decoded_call = <T as Config>::RuntimeCall::decode_all_with_depth_limit(
                    sp_api::MAX_EXTRINSIC_DEPTH / 4,
                    &mut &old_data.actual_call[..],
                )
                .map_err(|_| Error::<T>::FailedDecodingCall)?;

                // If the proposal thresholds are met, remove proposal from storage
                *data = None;

                // Dispatch the call and get the result
                let dispatch_result = crate::dispatch::dispatch_call::<T>(
                    core_id,
                    &old_data.fee_asset,
                    decoded_call.clone(),
                );

                Self::deposit_event(Event::MultisigExecuted {
                    core_id,
                    executor_account: Self::derive_core_account(core_id),
                    voter: owner,
                    call_hash,
                    call: decoded_call,
                    result: dispatch_result.map(|_| ()).map_err(|e| e.error),
                });
            } else {
                // If the thresholds aren't met, update storage with the new tally
                *data = Some(old_data.clone());

                Self::deposit_event(Event::MultisigVoteAdded {
                    core_id,
                    executor_account: Self::derive_core_account(core_id),
                    voter: owner,
                    votes_added: new_vote_record,
                    current_votes: old_data.tally,
                    call_hash,
                });
            }

            Ok(().into())
        })
    }

    /// Inner function for the withdraw_token_multisig call.
    pub(crate) fn inner_withdraw_vote_multisig(
        caller: OriginFor<T>,
        core_id: T::CoreId,
        call_hash: T::Hash,
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists(core_id, call_hash, |data| {
            let owner = ensure_signed(caller.clone())?;

            // Get the voting token balance of the caller
            let mut old_data = data.take().ok_or(Error::<T>::MultisigCallNotFound)?;

            // Try to mutate tally to remove the vote
            let old_vote = old_data.tally.process_vote(owner.clone(), None)?;

            // Update storage with the new tally
            *data = Some(old_data.clone());

            Self::deposit_event(Event::MultisigVoteWithdrawn {
                core_id,
                executor_account: Self::derive_core_account(core_id),
                voter: owner,
                votes_removed: old_vote,
                call_hash,
            });

            Ok(().into())
        })
    }

    /// Inner function for the cancel_multisig_proposal call.
    pub(crate) fn inner_cancel_multisig_proposal(
        origin: OriginFor<T>,
        call_hash: T::Hash,
    ) -> DispatchResultWithPostInfo {
        // Ensure that this is being called by the multisig origin rather than by a normal caller
        let core_origin = ensure_multisig::<T, OriginFor<T>>(origin)?;
        let core_id = core_origin.id;

        // Remove the proposal from storage
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
