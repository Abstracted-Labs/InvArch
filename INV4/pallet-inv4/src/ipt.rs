use super::pallet::{self, *};
use core::convert::TryInto;
use frame_support::dispatch::CallMetadata;
use frame_support::dispatch::Dispatchable;
use frame_support::dispatch::GetCallMetadata;
use frame_support::dispatch::GetDispatchInfo;
use frame_support::dispatch::RawOrigin;
use frame_support::pallet_prelude::*;
use frame_support::traits::WrapperKeepOpaque;
use frame_support::weights::WeightToFeePolynomial;
use frame_system::ensure_signed;
use frame_system::pallet_prelude::*;
use primitives::utils::multi_account_id;
use primitives::{OneOrPercent, Parentage, SubIptInfo};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::CheckedSub;
use sp_runtime::traits::StaticLookup;
use sp_std::boxed::Box;
use sp_std::vec;
use sp_std::vec::Vec;

pub type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::Call>;

#[derive(Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct MultisigOperation<AccountId, Signers, Call, Args> {
    signers: Signers,
    include_original_caller: bool,
    original_caller: AccountId,
    actual_call: Call,
    call_metadata: [u8; 2],
    call_arguments: Args,
    call_weight: Weight,
}

pub type MultisigOperationOf<T> = MultisigOperation<
    <T as frame_system::Config>::AccountId,
    BoundedVec<
        (
            <T as frame_system::Config>::AccountId,
            Option<<T as pallet::Config>::IpId>,
        ),
        <T as Config>::MaxCallers,
    >,
    OpaqueCall<T>,
    BoundedVec<u8, <T as pallet::Config>::MaxWasmPermissionBytes>,
>;

pub type SubAssetsWithEndowment<T> = Vec<(
    SubIptInfo<<T as pallet::Config>::IpId, BoundedVec<u8, <T as pallet::Config>::MaxMetadata>>,
    (
        <T as frame_system::Config>::AccountId,
        <T as pallet::Config>::Balance,
    ),
)>;

impl<T: Config> Pallet<T> {
    pub(crate) fn inner_ipt_mint(
        owner: OriginFor<T>,
        ipt_id: (T::IpId, Option<T::IpId>),
        amount: <T as pallet::Config>::Balance,
        target: T::AccountId,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let ip = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

        match &ip.parentage {
            Parentage::Parent(ips_account) => {
                ensure!(ips_account == &owner, Error::<T>::NoPermission)
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        if let Some(sub_asset) = ipt_id.1 {
            ensure!(
                SubAssets::<T>::get(ipt_id.0, sub_asset).is_some(),
                Error::<T>::SubAssetNotFound
            );
        }

        Pallet::<T>::internal_mint(ipt_id, target.clone(), amount)?;

        Self::deposit_event(Event::Minted(ipt_id, target, amount));

        Ok(())
    }

    pub(crate) fn inner_ipt_burn(
        owner: OriginFor<T>,
        ipt_id: (T::IpId, Option<T::IpId>),
        amount: <T as pallet::Config>::Balance,
        target: T::AccountId,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let ip = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

        match &ip.parentage {
            Parentage::Parent(ips_account) => {
                ensure!(ips_account == &owner, Error::<T>::NoPermission)
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        if let Some(sub_asset) = ipt_id.1 {
            ensure!(
                SubAssets::<T>::get(ipt_id.0, sub_asset).is_some(),
                Error::<T>::SubAssetNotFound
            );
        }

        Pallet::<T>::internal_burn(target.clone(), ipt_id, amount)?;

        Self::deposit_event(Event::Burned(ipt_id, target, amount));

        Ok(())
    }

    pub(crate) fn inner_operate_multisig(
        caller: OriginFor<T>,
        include_caller: bool,
        ipt_id: (T::IpId, Option<T::IpId>),
        call: Box<<T as pallet::Config>::Call>,
    ) -> DispatchResultWithPostInfo {
        let owner = ensure_signed(caller.clone())?;

        ensure!(
            !matches!(
                call.get_call_metadata(),
                CallMetadata {
                    pallet_name: "RmrkCore",
                    function_name: "send"
                        | "burn_nft"
                        | "destroy_collection"
                        | "change_collection_issuer",
                }
            ),
            Error::<T>::CantExecuteThisCall
        );

        let ipt = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

        let total_issuance = ipt.supply
            + SubAssets::<T>::iter_prefix_values(ipt_id.0)
                .map(|sub_asset| {
                    let supply = IpStorage::<T>::get(sub_asset.id)?.supply;

                    if let OneOrPercent::ZeroPoint(weight) =
                        Pallet::<T>::asset_weight(ipt_id.0, sub_asset.id)?
                    {
                        Some(weight * supply)
                    } else {
                        Some(supply)
                    }
                })
                .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                .ok_or(Error::<T>::IpDoesntExist)?
                .into_iter()
                .sum();

        let total_per_threshold: <T as pallet::Config>::Balance =
            if let OneOrPercent::ZeroPoint(percent) =
                Pallet::<T>::execution_threshold(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?
            {
                percent * total_issuance
            } else {
                total_issuance
            };

        let call_metadata: [u8; 2] = call
            .encode()
            .split_at(2)
            .0
            .try_into()
            .map_err(|_| Error::<T>::CallHasTooFewBytes)?;

        let call_arguments: BoundedVec<u8, T::MaxWasmPermissionBytes> =
            call.encode().split_at(2).1.to_vec().try_into().unwrap(); // TODO: Remove unwrap

        let owner_balance: <T as Config>::Balance = if let OneOrPercent::ZeroPoint(percent) = {
            if let Some(sub_asset) = ipt_id.1 {
                ensure!(
                    Pallet::<T>::has_permission(
                        ipt_id.0,
                        sub_asset,
                        call_metadata,
                        call_arguments.clone()
                    )
                    .ok_or(Error::<T>::IpDoesntExist)?,
                    Error::<T>::SubAssetHasNoPermission
                );

                Pallet::<T>::asset_weight(ipt_id.0, sub_asset).ok_or(Error::<T>::IpDoesntExist)?
            } else {
                OneOrPercent::One
            }
        } {
            percent * Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
        } else {
            Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
        };

        let opaque_call: OpaqueCall<T> = WrapperKeepOpaque::from_encoded(call.encode());

        let call_hash: [u8; 32] = blake2_256(&call.encode());

        ensure!(
            Multisig::<T>::get((ipt_id.0, blake2_256(&call.encode()))).is_none(),
            Error::<T>::MultisigOperationAlreadyExists
        );

        if owner_balance > total_per_threshold {
            pallet_balances::Pallet::<T>::transfer(
                caller,
                <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                    multi_account_id::<T, T::IpId>(ipt_id.0, None),
                ),
                <T as pallet::Config>::Balance::from(T::WeightToFeePolynomial::calc(
                    &call.get_dispatch_info().weight,
                ))
                .into(),
            )?;

            let dispatch_result = call.dispatch(
                RawOrigin::Signed(multi_account_id::<T, T::IpId>(
                    ipt_id.0,
                    if include_caller {
                        Some(owner.clone())
                    } else {
                        None
                    },
                ))
                .into(),
            );

            Self::deposit_event(Event::MultisigExecuted(
                multi_account_id::<T, T::IpId>(
                    ipt_id.0,
                    if include_caller { Some(owner) } else { None },
                ),
                opaque_call,
                dispatch_result.is_ok(),
            ));
        } else {
            pallet_balances::Pallet::<T>::transfer(
                caller,
                <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                    multi_account_id::<T, T::IpId>(ipt_id.0, None),
                ),
                <T as pallet::Config>::Balance::from(
                    (T::WeightToFeePolynomial::calc(&call.get_dispatch_info().weight)
                        / total_per_threshold.into())
                        * owner_balance.into(),
                )
                .into(),
            )?;

            Multisig::<T>::insert(
                (ipt_id.0, call_hash),
                MultisigOperation {
                    signers: vec![(owner.clone(), ipt_id.1)]
                        .try_into()
                        .map_err(|_| Error::<T>::TooManySignatories)?,
                    include_original_caller: include_caller,
                    original_caller: owner.clone(),
                    actual_call: opaque_call.clone(),
                    call_metadata,
                    call_arguments,
                    call_weight: call.get_dispatch_info().weight,
                },
            );

            Self::deposit_event(Event::MultisigVoteStarted(
                multi_account_id::<T, T::IpId>(
                    ipt_id.0,
                    if include_caller { Some(owner) } else { None },
                ),
                owner_balance,
                ipt.supply,
                call_hash,
                opaque_call,
            ));
        }

        Ok(().into())
    }

    pub(crate) fn inner_vote_multisig(
        caller: OriginFor<T>,
        ipt_id: (T::IpId, Option<T::IpId>),
        call_hash: [u8; 32],
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists((ipt_id.0, call_hash), |data| {
            let owner = ensure_signed(caller.clone())?;

            let ipt = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

            let mut old_data = data
                .take()
                .ok_or(Error::<T>::MultisigOperationUninitialized)?;

            let voter_balance = if let OneOrPercent::ZeroPoint(percent) = {
                if let Some(sub_asset) = ipt_id.1 {
                    ensure!(
                        Pallet::<T>::has_permission(
                            ipt_id.0,
                            sub_asset,
                            old_data.call_metadata,
                            old_data.call_arguments.clone()
                        )
                        .ok_or(Error::<T>::IpDoesntExist)?,
                        Error::<T>::SubAssetHasNoPermission
                    );

                    Pallet::<T>::asset_weight(ipt_id.0, sub_asset)
                        .ok_or(Error::<T>::IpDoesntExist)?
                } else {
                    OneOrPercent::One
                }
            } {
                percent
                    * Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
            } else {
                Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
            };

            let total_in_operation: <T as pallet::Config>::Balance = old_data
                .signers
                .clone()
                .into_iter()
                .map(|(voter, sub_asset): (T::AccountId, Option<T::IpId>)| {
                    Balance::<T>::get((ipt_id.0, sub_asset), voter).map(|balance| {
                        if let OneOrPercent::ZeroPoint(percent) = if let Some(sub_asset) = ipt_id.1
                        {
                            Pallet::<T>::asset_weight(ipt_id.0, sub_asset).unwrap()
                        } else {
                            OneOrPercent::One
                        } {
                            percent * balance
                        } else {
                            balance
                        }
                    })
                })
                .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                .ok_or(Error::<T>::NoPermission)?
                .into_iter()
                .sum();

            let total_issuance = ipt.supply
                + SubAssets::<T>::iter_prefix_values(ipt_id.0)
                    .map(|sub_asset| {
                        let supply = IpStorage::<T>::get(sub_asset.id)?.supply;

                        if let OneOrPercent::ZeroPoint(weight) =
                            Pallet::<T>::asset_weight(ipt_id.0, sub_asset.id)?
                        {
                            Some(weight * supply)
                        } else {
                            Some(supply)
                        }
                    })
                    .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                    .ok_or(Error::<T>::IpDoesntExist)?
                    .into_iter()
                    .sum();

            let total_per_threshold: <T as pallet::Config>::Balance =
                if let OneOrPercent::ZeroPoint(percent) =
                    Pallet::<T>::execution_threshold(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?
                {
                    percent * total_issuance
                } else {
                    total_issuance
                };

            let fee: <T as pallet::Config>::Balance =
                T::WeightToFeePolynomial::calc(&old_data.call_weight).into();

            if (total_in_operation + voter_balance) > total_per_threshold {
                pallet_balances::Pallet::<T>::transfer(
                    caller,
                    <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                        multi_account_id::<T, T::IpId>(ipt_id.0, None),
                    ),
                    // Voter will pay the remainder of the fee after subtracting the total IPTs already in the operation converted to real fee value.
                    fee.checked_sub(&(total_in_operation * (fee / total_per_threshold)))
                        .ok_or(Error::<T>::NotEnoughAmount)?
                        .into(),
                )?;

                *data = None;

                let dispatch_result = old_data
                    .actual_call
                    .try_decode()
                    .ok_or(Error::<T>::CouldntDecodeCall)?
                    .dispatch(
                        RawOrigin::Signed(multi_account_id::<T, T::IpId>(
                            ipt_id.0,
                            if old_data.include_original_caller {
                                Some(old_data.original_caller.clone())
                            } else {
                                None
                            },
                        ))
                        .into(),
                    );

                Self::deposit_event(Event::MultisigExecuted(
                    multi_account_id::<T, T::IpId>(
                        ipt_id.0,
                        if old_data.include_original_caller {
                            Some(old_data.original_caller.clone())
                        } else {
                            None
                        },
                    ),
                    old_data.actual_call,
                    dispatch_result.is_ok(),
                ));
            } else {
                pallet_balances::Pallet::<T>::transfer(
                    caller,
                    <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                        multi_account_id::<T, T::IpId>(ipt_id.0, None),
                    ),
                    <T as pallet::Config>::Balance::from(
                        (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                            / total_per_threshold.into())
                            * voter_balance.into(),
                    )
                    .into(),
                )?;

                old_data.signers = {
                    let mut v = old_data.signers.to_vec();
                    v.push((owner, ipt_id.1));
                    v.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?
                };
                *data = Some(old_data.clone());

                Self::deposit_event(Event::MultisigVoteAdded(
                    multi_account_id::<T, T::IpId>(
                        ipt_id.0,
                        if old_data.include_original_caller {
                            Some(old_data.original_caller.clone())
                        } else {
                            None
                        },
                    ),
                    voter_balance,
                    ipt.supply,
                    call_hash,
                    old_data.actual_call,
                ));
            }

            Ok(().into())
        })
    }

    pub(crate) fn inner_withdraw_vote_multisig(
        caller: OriginFor<T>,
        ipt_id: (T::IpId, Option<T::IpId>),
        call_hash: [u8; 32],
    ) -> DispatchResultWithPostInfo {
        Multisig::<T>::try_mutate_exists((ipt_id.0, call_hash), |data| {
            let owner = ensure_signed(caller.clone())?;

            let ipt = IpStorage::<T>::get(ipt_id.0).ok_or(Error::<T>::IpDoesntExist)?;

            let mut old_data = data
                .take()
                .ok_or(Error::<T>::MultisigOperationUninitialized)?;

            ensure!(
                old_data.signers.iter().any(|signer| signer.0 == owner),
                Error::<T>::NotAVoter
            );

            if owner == old_data.original_caller {
                let total_issuance = ipt.supply
                    + SubAssets::<T>::iter_prefix_values(ipt_id.0)
                        .map(|sub_asset| {
                            let supply = IpStorage::<T>::get(sub_asset.id)?.supply;

                            if let OneOrPercent::ZeroPoint(weight) =
                                Pallet::<T>::asset_weight(ipt_id.0, sub_asset.id)?
                            {
                                Some(weight * supply)
                            } else {
                                Some(supply)
                            }
                        })
                        .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                        .ok_or(Error::<T>::IpDoesntExist)?
                        .into_iter()
                        .sum();

                let total_per_threshold: <T as pallet::Config>::Balance =
                    if let OneOrPercent::ZeroPoint(percent) =
                        Pallet::<T>::execution_threshold(ipt_id.0)
                            .ok_or(Error::<T>::IpDoesntExist)?
                    {
                        percent * total_issuance
                    } else {
                        total_issuance
                    };

                for signer in old_data.signers {
                    pallet_balances::Pallet::<T>::transfer(
                        <T as frame_system::Config>::Origin::from(RawOrigin::Signed(
                            multi_account_id::<T, T::IpId>(ipt_id.0, None),
                        )),
                        <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                            signer.0.clone(),
                        ),
                        <T as pallet::Config>::Balance::from(
                            (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                                / total_per_threshold.into())
                                * Balance::<T>::get((ipt_id.0, signer.1), signer.0)
                                    .ok_or(Error::<T>::UnknownError)?
                                    .into(),
                        )
                        .into(),
                    )?;
                }

                *data = None;
                Self::deposit_event(Event::MultisigCanceled(
                    multi_account_id::<T, T::IpId>(
                        ipt_id.0,
                        if old_data.include_original_caller {
                            Some(old_data.original_caller)
                        } else {
                            None
                        },
                    ),
                    call_hash,
                ));
            } else {
                let voter_balance = if let OneOrPercent::ZeroPoint(percent) = {
                    if let Some(sub_asset) = ipt_id.1 {
                        Pallet::<T>::asset_weight(ipt_id.0, sub_asset)
                            .ok_or(Error::<T>::IpDoesntExist)?
                    } else {
                        OneOrPercent::One
                    }
                } {
                    percent
                        * Balance::<T>::get(ipt_id, owner.clone())
                            .ok_or(Error::<T>::NoPermission)?
                } else {
                    Balance::<T>::get(ipt_id, owner.clone()).ok_or(Error::<T>::NoPermission)?
                };

                let total_issuance = ipt.supply
                    + SubAssets::<T>::iter_prefix_values(ipt_id.0)
                        .map(|sub_asset| {
                            let supply = IpStorage::<T>::get(sub_asset.id)?.supply;

                            if let OneOrPercent::ZeroPoint(weight) =
                                Pallet::<T>::asset_weight(ipt_id.0, sub_asset.id)?
                            {
                                Some(weight * supply)
                            } else {
                                Some(supply)
                            }
                        })
                        .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                        .ok_or(Error::<T>::IpDoesntExist)?
                        .into_iter()
                        .sum();

                let total_per_threshold: <T as pallet::Config>::Balance =
                    if let OneOrPercent::ZeroPoint(percent) =
                        Pallet::<T>::execution_threshold(ipt_id.0)
                            .ok_or(Error::<T>::IpDoesntExist)?
                    {
                        percent * total_issuance
                    } else {
                        total_issuance
                    };

                old_data.signers = old_data
                    .signers
                    .into_iter()
                    .filter(|signer| signer.0 != owner)
                    .collect::<Vec<(T::AccountId, Option<T::IpId>)>>()
                    .try_into()
                    .map_err(|_| Error::<T>::TooManySignatories)?;

                pallet_balances::Pallet::<T>::transfer(
                    <T as frame_system::Config>::Origin::from(RawOrigin::Signed(
                        multi_account_id::<T, T::IpId>(ipt_id.0, None),
                    )),
                    <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(owner),
                    <T as pallet::Config>::Balance::from(
                        (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                            / total_per_threshold.into())
                            * voter_balance.into(),
                    )
                    .into(),
                )?;

                *data = Some(old_data.clone());

                Self::deposit_event(Event::MultisigVoteWithdrawn(
                    multi_account_id::<T, T::IpId>(
                        ipt_id.0,
                        if old_data.include_original_caller {
                            Some(old_data.original_caller.clone())
                        } else {
                            None
                        },
                    ),
                    voter_balance,
                    ipt.supply,
                    call_hash,
                    old_data.actual_call,
                ));
            }

            Ok(().into())
        })
    }

    pub(crate) fn inner_create_sub_asset(
        caller: OriginFor<T>,
        ipt_id: T::IpId,
        sub_assets: SubAssetsWithEndowment<T>,
    ) -> DispatchResultWithPostInfo {
        IpStorage::<T>::try_mutate_exists(ipt_id, |ipt| -> DispatchResultWithPostInfo {
            let caller = ensure_signed(caller.clone())?;

            let old_ipt = ipt.clone().ok_or(Error::<T>::IpDoesntExist)?;

            match old_ipt.parentage {
                Parentage::Parent(ips_account) => {
                    ensure!(ips_account == caller, Error::<T>::NoPermission)
                }
                Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
            }

            for sub in sub_assets.clone() {
                ensure!(
                    !SubAssets::<T>::contains_key(ipt_id, sub.0.id),
                    Error::<T>::SubAssetAlreadyExists
                );

                SubAssets::<T>::insert(ipt_id, sub.0.id, &sub.0);

                Balance::<T>::insert((ipt_id, Some(sub.0.id)), sub.1 .0, sub.1 .1);
            }

            Self::deposit_event(Event::SubAssetCreated(
                sub_assets
                    .into_iter()
                    .map(|sub| (ipt_id, sub.0.id))
                    .collect(),
            ));

            Ok(().into())
        })
    }

    pub fn internal_mint(
        ipt_id: (T::IpId, Option<T::IpId>),
        target: T::AccountId,
        amount: <T as pallet::Config>::Balance,
    ) -> DispatchResult {
        IpStorage::<T>::try_mutate(ipt_id.0, |ipt| -> DispatchResult {
            Balance::<T>::try_mutate(ipt_id, target, |balance| -> DispatchResult {
                let old_balance = balance.take().unwrap_or_default();
                *balance = Some(old_balance + amount);

                let mut old_ipt = ipt.take().ok_or(Error::<T>::IpDoesntExist)?;

                if ipt_id.1.is_none() {
                    old_ipt.supply += amount;
                }

                *ipt = Some(old_ipt);

                Ok(())
            })
        })
    }

    pub fn internal_burn(
        target: T::AccountId,
        ipt_id: (T::IpId, Option<T::IpId>),
        amount: <T as pallet::Config>::Balance,
    ) -> DispatchResult {
        IpStorage::<T>::try_mutate(ipt_id.0, |ipt| -> DispatchResult {
            Balance::<T>::try_mutate(ipt_id, target, |balance| -> DispatchResult {
                let old_balance = balance.take().ok_or(Error::<T>::IpDoesntExist)?;
                *balance = Some(
                    old_balance
                        .checked_sub(&amount)
                        .ok_or(Error::<T>::NotEnoughAmount)?,
                );

                let mut old_ipt = ipt.take().ok_or(Error::<T>::IpDoesntExist)?;

                if ipt_id.1.is_none() {
                    old_ipt.supply = old_ipt
                        .supply
                        .checked_sub(&amount)
                        .ok_or(Error::<T>::NotEnoughAmount)?;
                }

                *ipt = Some(old_ipt);

                Ok(())
            })
        })
    }
}
