#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
    pallet_prelude::*,
    traits::{Currency as FSCurrency, Get, WrapperKeepOpaque},
    Parameter,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, Member};
use sp_std::vec::Vec;

pub use pallet::*;

type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::Call>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct MultisigOperation<AccountId, Signers, Call> {
    signers: Signers,
    include_original_caller: bool,
    original_caller: AccountId,
    actual_call: Call,
    call_weight: Weight,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use core::iter::Sum;
    use frame_support::{
        dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
        storage::bounded_btree_map::BoundedBTreeMap,
        weights::WeightToFeePolynomial,
    };
    use frame_system::RawOrigin;
    use primitives::{utils::multi_account_id, IptInfo, SubIptInfo};
    use scale_info::prelude::fmt::Display;
    use sp_io::hashing::blake2_256;
    use sp_runtime::traits::{CheckedSub, One, StaticLookup};
    use sp_std::{boxed::Box, convert::TryInto, vec};

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        /// The IPS Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Currency
        type Currency: FSCurrency<Self::AccountId>;
        /// The units in which we record balances.
        type Balance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo
            + Sum<<Self as pallet::Config>::Balance>
            + IsType<<Self as pallet_balances::Config>::Balance>
            + IsType<
                <<Self as pallet::Config>::WeightToFeePolynomial as WeightToFeePolynomial>::Balance,
            >;

        /// The IPS ID type
        type IptId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen;

        /// The overarching call type.
        type Call: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>;

        type WeightToFeePolynomial: WeightToFeePolynomial;

        /// The maximum numbers of caller accounts on a single Multisig call
        #[pallet::constant]
        type MaxCallers: Get<u32>;

        #[pallet::constant]
        type MaxSubAssets: Get<u32>;

        #[pallet::constant]
        type ExistentialDeposit: Get<<Self as pallet::Config>::Balance>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as FSCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn multisig)]
    /// Details of a multisig call.
    pub type Multisig<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::IptId, [u8; 32]), MultisigOperationOf<T>>;

    pub type MultisigOperationOf<T> = MultisigOperation<
        <T as frame_system::Config>::AccountId,
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxCallers>,
        OpaqueCall<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn ipt)]
    /// Details of an asset.
    pub type Ipt<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::IptId,
        IptInfo<
            <T as pallet::Config>::Balance,
            T::AccountId,
            T::IptId,
            BoundedBTreeMap<T::IptId, SubIptInfo<T::IptId>, <T as Config>::MaxSubAssets>,
        >,
    >;

    #[pallet::storage]
    #[pallet::getter(fn balance)]
    /// The holdings of a specific account for a specific asset.
    pub type Balance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::IptId, Option<T::IptId>),
        Blake2_128Concat,
        T::AccountId,
        <T as pallet::Config>::Balance,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        Minted(
            (T::IptId, Option<T::IptId>),
            T::AccountId,
            <T as pallet::Config>::Balance,
        ),
        Burned(
            (T::IptId, Option<T::IptId>),
            T::AccountId,
            <T as pallet::Config>::Balance,
        ),
        MultisigVoteStarted(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            OpaqueCall<T>,
        ),
        MultisigVoteAdded(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            OpaqueCall<T>,
        ),
        MultisigVoteWithdrawn(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            [u8; 32],
            OpaqueCall<T>,
        ),
        MultisigExecuted(T::AccountId, OpaqueCall<T>, bool),
        MultisigCanceled(T::AccountId, [u8; 32]),
        SubAssetCreated(Vec<(T::IptId, T::IptId)>),
    }

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        IptDoesntExist,
        NoPermission,
        NotEnoughAmount,
        TooManySignatories,
        UnexistentBalance,
        MultisigOperationUninitialized,
        MaxMetadataExceeded,
        CouldntDecodeCall,
        MultisigOperationAlreadyExists,
        NotAVoter,
        UnknownError,
        SubAssetNotFound,
        SubAssetAlreadyExists,
        TooManySubAssets,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn mint(
            owner: OriginFor<T>,
            ips_id: (T::IptId, Option<T::IptId>),
            amount: <T as pallet::Config>::Balance,
            target: T::AccountId,
        ) -> DispatchResult {
            let owner = ensure_signed(owner)?;

            let ipt = Ipt::<T>::get(ips_id.0).ok_or(Error::<T>::IptDoesntExist)?;

            ensure!(owner == ipt.owner, Error::<T>::NoPermission);

            Pallet::<T>::internal_mint(ips_id, target.clone(), amount)?;

            Self::deposit_event(Event::Minted(ips_id, target, amount));

            Ok(())
        }

        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn burn(
            owner: OriginFor<T>,
            ips_id: (T::IptId, Option<T::IptId>),
            amount: <T as pallet::Config>::Balance,
            target: T::AccountId,
        ) -> DispatchResult {
            let owner = ensure_signed(owner)?;

            let ipt = Ipt::<T>::get(ips_id.0).ok_or(Error::<T>::IptDoesntExist)?;

            ensure!(owner == ipt.owner, Error::<T>::NoPermission);

            Pallet::<T>::internal_burn(target.clone(), ips_id, amount)?;

            Self::deposit_event(Event::Burned(ips_id, target, amount));

            Ok(())
        }

        #[pallet::weight(100_000)]
        pub fn operate_multisig(
            caller: OriginFor<T>,
            include_caller: bool,
            ips_id: T::IptId,
            call: Box<<T as pallet::Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(caller.clone())?;
            let ipt = Ipt::<T>::get(ips_id).ok_or(Error::<T>::IptDoesntExist)?;

            let total_per_2 = ipt.supply / {
                let one: <T as Config>::Balance = One::one();
                one + one
            };

            let id: (T::IptId, Option<T::IptId>) = (ips_id, None);
            let owner_balance =
                Balance::<T>::get(id, owner.clone()).ok_or(Error::<T>::NoPermission)?;

            let opaque_call: OpaqueCall<T> = WrapperKeepOpaque::from_encoded(call.encode());

            let call_hash: [u8; 32] = blake2_256(&call.encode());

            ensure!(
                Multisig::<T>::get((ips_id, blake2_256(&call.encode()))).is_none(),
                Error::<T>::MultisigOperationAlreadyExists
            );

            if owner_balance > total_per_2 {
                pallet_balances::Pallet::<T>::transfer(
                    caller,
                    <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                        multi_account_id::<T, T::IptId>(ips_id, None),
                    ),
                    <T as pallet::Config>::Balance::from(T::WeightToFeePolynomial::calc(
                        &call.get_dispatch_info().weight,
                    ))
                    .into(),
                )?;

                let dispatch_result = call.dispatch(
                    RawOrigin::Signed(multi_account_id::<T, T::IptId>(
                        ips_id,
                        if include_caller {
                            Some(owner.clone())
                        } else {
                            None
                        },
                    ))
                    .into(),
                );

                Self::deposit_event(Event::MultisigExecuted(
                    multi_account_id::<T, T::IptId>(
                        ips_id,
                        if include_caller { Some(owner) } else { None },
                    ),
                    opaque_call,
                    dispatch_result.is_ok(),
                ));
            } else {
                pallet_balances::Pallet::<T>::transfer(
                    caller,
                    <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                        multi_account_id::<T, T::IptId>(ips_id, None),
                    ),
                    <T as pallet::Config>::Balance::from(
                        (T::WeightToFeePolynomial::calc(&call.get_dispatch_info().weight)
                            / total_per_2.into())
                            * owner_balance.into(),
                    )
                    .into(),
                )?;

                Multisig::<T>::insert(
                    (ips_id, call_hash),
                    MultisigOperation {
                        signers: vec![owner.clone()]
                            .try_into()
                            .map_err(|_| Error::<T>::TooManySignatories)?,
                        include_original_caller: include_caller,
                        original_caller: owner.clone(),
                        actual_call: opaque_call.clone(),
                        call_weight: call.get_dispatch_info().weight,
                    },
                );

                Self::deposit_event(Event::MultisigVoteStarted(
                    multi_account_id::<T, T::IptId>(
                        ips_id,
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

        #[pallet::weight(100_000)]
        pub fn vote_multisig(
            caller: OriginFor<T>,
            ips_id: T::IptId,
            call_hash: [u8; 32],
        ) -> DispatchResultWithPostInfo {
            Multisig::<T>::try_mutate_exists((ips_id, call_hash), |data| {
                let owner = ensure_signed(caller.clone())?;

                let ipt = Ipt::<T>::get(ips_id).ok_or(Error::<T>::IptDoesntExist)?;

                let mut old_data = data
                    .take()
                    .ok_or(Error::<T>::MultisigOperationUninitialized)?;

                let id: (T::IptId, Option<T::IptId>) = (ips_id, None);
                let voter_balance =
                    Balance::<T>::get(id, owner.clone()).ok_or(Error::<T>::NoPermission)?;

                let total_in_operation: <T as pallet::Config>::Balance = old_data
                    .signers
                    .clone()
                    .into_iter()
                    .map(|voter| -> Option<<T as pallet::Config>::Balance> {
                        Balance::<T>::get(id, voter)
                    })
                    .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                    .ok_or(Error::<T>::NoPermission)?
                    .into_iter()
                    .sum();

                let total_per_2 = ipt.supply / {
                    let one: <T as Config>::Balance = One::one();
                    one + one
                };

                let fee: <T as pallet::Config>::Balance =
                    T::WeightToFeePolynomial::calc(&old_data.call_weight).into();

                if (total_in_operation + voter_balance) > total_per_2 {
                    pallet_balances::Pallet::<T>::transfer(
                        caller,
                        <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                            multi_account_id::<T, T::IptId>(ips_id, None),
                        ),
                        // Voter will pay the remainder of the fee after subtracting the total IPTs already in the operation converted to real fee value.
                        fee.checked_sub(&(total_in_operation * (fee / total_per_2)))
                            .ok_or(Error::<T>::NotEnoughAmount)?
                            .into(),
                    )?;

                    *data = None;

                    let dispatch_result = old_data
                        .actual_call
                        .try_decode()
                        .ok_or(Error::<T>::CouldntDecodeCall)?
                        .dispatch(
                            RawOrigin::Signed(multi_account_id::<T, T::IptId>(
                                ips_id,
                                if old_data.include_original_caller {
                                    Some(old_data.original_caller.clone())
                                } else {
                                    None
                                },
                            ))
                            .into(),
                        );

                    Self::deposit_event(Event::MultisigExecuted(
                        multi_account_id::<T, T::IptId>(
                            ips_id,
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
                            multi_account_id::<T, T::IptId>(ips_id, None),
                        ),
                        <T as pallet::Config>::Balance::from(
                            (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                                / total_per_2.into())
                                * voter_balance.into(),
                        )
                        .into(),
                    )?;

                    old_data.signers = {
                        let mut v = old_data.signers.to_vec();
                        v.push(owner);
                        v.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?
                    };
                    *data = Some(old_data.clone());

                    Self::deposit_event(Event::MultisigVoteAdded(
                        multi_account_id::<T, T::IptId>(
                            ips_id,
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

        #[pallet::weight(100_000)]
        pub fn withdraw_vote_multisig(
            caller: OriginFor<T>,
            ips_id: T::IptId,
            call_hash: [u8; 32],
        ) -> DispatchResultWithPostInfo {
            Multisig::<T>::try_mutate_exists((ips_id, call_hash), |data| {
                let owner = ensure_signed(caller.clone())?;

                let ipt = Ipt::<T>::get(ips_id).ok_or(Error::<T>::IptDoesntExist)?;

                let mut old_data = data
                    .take()
                    .ok_or(Error::<T>::MultisigOperationUninitialized)?;

                ensure!(old_data.signers.contains(&owner), Error::<T>::NotAVoter);

                let id: (T::IptId, Option<T::IptId>) = (ips_id, None);
                if owner == old_data.original_caller {
                    let total_per_2 = ipt.supply / {
                        let one: <T as Config>::Balance = One::one();
                        one + one
                    };

                    for signer in old_data.signers {
                        pallet_balances::Pallet::<T>::transfer(
                            <T as frame_system::Config>::Origin::from(RawOrigin::Signed(
                                multi_account_id::<T, T::IptId>(ips_id, None),
                            )),
                            <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                                signer.clone(),
                            ),
                            <T as pallet::Config>::Balance::from(
                                (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                                    / total_per_2.into())
                                    * Balance::<T>::get(id, signer)
                                        .ok_or(Error::<T>::UnknownError)?
                                        .into(),
                            )
                            .into(),
                        )?;
                    }

                    *data = None;
                    Self::deposit_event(Event::MultisigCanceled(
                        multi_account_id::<T, T::IptId>(
                            ips_id,
                            if old_data.include_original_caller {
                                Some(old_data.original_caller)
                            } else {
                                None
                            },
                        ),
                        call_hash,
                    ));
                } else {
                    let voter_balance =
                        Balance::<T>::get(id, owner.clone()).ok_or(Error::<T>::NoPermission)?;

                    let total_per_2 = ipt.supply / {
                        let one: <T as Config>::Balance = One::one();
                        one + one
                    };

                    old_data.signers = old_data
                        .signers
                        .into_iter()
                        .filter(|signer| signer != &owner)
                        .collect::<Vec<T::AccountId>>()
                        .try_into()
                        .map_err(|_| Error::<T>::TooManySignatories)?;

                    pallet_balances::Pallet::<T>::transfer(
                        <T as frame_system::Config>::Origin::from(RawOrigin::Signed(
                            multi_account_id::<T, T::IptId>(ips_id, None),
                        )),
                        <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(owner),
                        <T as pallet::Config>::Balance::from(
                            (T::WeightToFeePolynomial::calc(&old_data.call_weight)
                                / total_per_2.into())
                                * voter_balance.into(),
                        )
                        .into(),
                    )?;

                    *data = Some(old_data.clone());

                    Self::deposit_event(Event::MultisigVoteWithdrawn(
                        multi_account_id::<T, T::IptId>(
                            ips_id,
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

        #[pallet::weight(100_000)]
        pub fn create_sub_asset(
            caller: OriginFor<T>,
            ipt_id: T::IptId,
            sub_assets: Vec<(T::IptId, (T::AccountId, <T as pallet::Config>::Balance))>,
        ) -> DispatchResultWithPostInfo {
            Ipt::<T>::try_mutate_exists(ipt_id, |ipt| -> DispatchResultWithPostInfo {
                let caller = ensure_signed(caller.clone())?;

                let mut old_ipt = ipt.take().ok_or(Error::<T>::IptDoesntExist)?;

                ensure!(caller == old_ipt.owner, Error::<T>::NoPermission);

                for sub in sub_assets.clone() {
                    ensure!(
                        old_ipt.sub_assets.get(&sub.0).is_none(),
                        Error::<T>::SubAssetAlreadyExists
                    );

                    old_ipt
                        .sub_assets
                        .try_insert(sub.0, SubIptInfo { id: sub.0 })
                        .map_err(|_| Error::<T>::TooManySubAssets)?;
                    Balance::<T>::insert((ipt_id, Some(sub.0)), sub.1 .0, sub.1 .1);
                }

                *ipt = Some(old_ipt);

                Self::deposit_event(Event::SubAssetCreated(
                    sub_assets.into_iter().map(|sub| (ipt_id, sub.0)).collect(),
                ));

                Ok(().into())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T: Config> Pallet<T> {
        pub fn create(
            owner: T::AccountId,
            ipt_id: T::IptId,
            endowed_accounts: Vec<(T::AccountId, <T as pallet::Config>::Balance)>,
            sub_assets: BoundedBTreeMap<T::IptId, SubIptInfo<T::IptId>, T::MaxSubAssets>,
        ) {
            Ipt::<T>::insert(
                ipt_id,
                IptInfo {
                    owner,
                    supply: endowed_accounts
                        .clone()
                        .into_iter()
                        .map(|(_, balance)| balance)
                        .sum(),
                    sub_assets,
                },
            );

            let id: (T::IptId, Option<T::IptId>) = (ipt_id, None);
            endowed_accounts
                .iter()
                .for_each(|(account, balance)| Balance::<T>::insert(id, account, balance));
        }

        pub fn internal_mint(
            ipt_id: (T::IptId, Option<T::IptId>),
            target: T::AccountId,
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            Ipt::<T>::try_mutate(ipt_id.0, |ipt| -> DispatchResult {
                Balance::<T>::try_mutate(ipt_id, target, |balance| -> DispatchResult {
                    let old_balance = balance.take().unwrap_or_default();
                    *balance = Some(old_balance + amount);

                    let mut old_ipt = ipt.take().ok_or(Error::<T>::IptDoesntExist)?;

                    if let None = ipt_id.1 {
                        old_ipt.supply += amount;
                    }

                    *ipt = Some(old_ipt);

                    Ok(())
                })
            })
        }

        pub fn internal_burn(
            target: T::AccountId,
            ipt_id: (T::IptId, Option<T::IptId>),
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            Ipt::<T>::try_mutate(ipt_id.0, |ipt| -> DispatchResult {
                Balance::<T>::try_mutate(ipt_id, target, |balance| -> DispatchResult {
                    let old_balance = balance.take().ok_or(Error::<T>::IptDoesntExist)?;
                    *balance = Some(
                        old_balance
                            .checked_sub(&amount)
                            .ok_or(Error::<T>::NotEnoughAmount)?,
                    );

                    let mut old_ipt = ipt.take().ok_or(Error::<T>::IptDoesntExist)?;

                    if let None = ipt_id.1 {
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
}
