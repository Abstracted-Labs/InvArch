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

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetDetails<Balance, AccountId> {
    owner: AccountId,
    /// The total supply across all accounts.
    supply: Balance,
    /// The balance deposited for this asset. This pays for the data stored here.
    deposit: Balance,
}

type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::Call>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct MultisigOperation<AccountId, Signers, Call> {
    signers: Signers,
    include_original_caller: Option<AccountId>,
    actual_call: Call,
    call_weight: Weight,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use core::iter::Sum;
    use frame_support::{
        dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
        weights::WeightToFeePolynomial,
    };
    use frame_system::RawOrigin;
    use primitives::utils::multi_account_id;
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
        AssetDetails<<T as pallet::Config>::Balance, T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn balance)]
    /// The holdings of a specific account for a specific asset.
    pub type Balance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::IptId,
        Blake2_128Concat,
        T::AccountId,
        <T as pallet::Config>::Balance,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        Minted(T::IptId, T::AccountId, <T as pallet::Config>::Balance),
        Burned(T::IptId, T::AccountId, <T as pallet::Config>::Balance),
        MultisigVoteStarted(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            OpaqueCall<T>,
        ),
        MultisigVoteAdded(
            T::AccountId,
            <T as pallet::Config>::Balance,
            <T as pallet::Config>::Balance,
            OpaqueCall<T>,
        ),
        MultisigExecuted(T::AccountId, OpaqueCall<T>),
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
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn mint(
            owner: OriginFor<T>,
            ips_id: T::IptId,
            amount: <T as pallet::Config>::Balance,
            target: T::AccountId,
        ) -> DispatchResult {
            let owner = ensure_signed(owner)?;

            let ipt = Ipt::<T>::get(ips_id).ok_or(Error::<T>::IptDoesntExist)?;

            ensure!(owner == ipt.owner, Error::<T>::NoPermission);

            Pallet::<T>::internal_mint(target.clone(), ips_id, amount)?;

            Self::deposit_event(Event::Minted(ips_id, target, amount));

            Ok(())
        }

        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn burn(
            owner: OriginFor<T>,
            ips_id: T::IptId,
            amount: <T as pallet::Config>::Balance,
            target: T::AccountId,
        ) -> DispatchResult {
            let owner = ensure_signed(owner)?;

            let ipt = Ipt::<T>::get(ips_id).ok_or(Error::<T>::IptDoesntExist)?;

            ensure!(owner == ipt.owner, Error::<T>::NoPermission);

            Pallet::<T>::internal_burn(target.clone(), ips_id, amount)?;

            Self::deposit_event(Event::Burned(ips_id, target, amount));

            Ok(())
        }

        #[pallet::weight(100_000)]
        pub fn as_multi(
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

            let owner_balance =
                Balance::<T>::get(ips_id, owner.clone()).ok_or(Error::<T>::NoPermission)?;

            let opaque_call: OpaqueCall<T> = WrapperKeepOpaque::from_encoded(call.encode());

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

                call.dispatch(
                    RawOrigin::Signed(multi_account_id::<T, T::IptId>(
                        ips_id,
                        if include_caller {
                            Some(owner.clone())
                        } else {
                            None
                        },
                    ))
                    .into(),
                )?;

                Self::deposit_event(Event::MultisigExecuted(
                    multi_account_id::<T, T::IptId>(
                        ips_id,
                        if include_caller { Some(owner) } else { None },
                    ),
                    opaque_call,
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
                    (ips_id, blake2_256(&call.encode())),
                    MultisigOperation {
                        signers: vec![owner.clone()]
                            .try_into()
                            .map_err(|_| Error::<T>::TooManySignatories)?,
                        include_original_caller: if include_caller {
                            Some(owner.clone())
                        } else {
                            None
                        },
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
                    opaque_call,
                ));
            }

            Ok(().into())
        }

        #[pallet::weight(100_000)]
        pub fn approve_as_multi(
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

                let voter_balance =
                    Balance::<T>::get(ips_id, owner.clone()).ok_or(Error::<T>::NoPermission)?;

                let total_in_operation: <T as pallet::Config>::Balance = old_data
                    .signers
                    .clone()
                    .into_iter()
                    .map(|voter| -> Option<<T as pallet::Config>::Balance> {
                        Balance::<T>::get(ips_id, voter)
                    })
                    .collect::<Option<Vec<<T as pallet::Config>::Balance>>>()
                    .ok_or(Error::<T>::NoPermission)?
                    .into_iter()
                    .sum();

                let total_per_2 = ipt.supply / 2u32.into();

                let fee: <T as pallet::Config>::Balance =
                    T::WeightToFeePolynomial::calc(&old_data.call_weight).into();

                if (total_in_operation + voter_balance) > total_per_2 {
                    pallet_balances::Pallet::<T>::transfer(
                        caller,
                        <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                            multi_account_id::<T, T::IptId>(ips_id, None),
                        ),
                        fee.checked_sub(&total_in_operation)
                            .ok_or(Error::<T>::NotEnoughAmount)
                            .unwrap()
                            .into(),
                    )?;

                    old_data
                        .actual_call
                        .try_decode()
                        .ok_or(Error::<T>::CouldntDecodeCall)?
                        .dispatch(
                            RawOrigin::Signed(multi_account_id::<T, T::IptId>(
                                ips_id,
                                old_data.include_original_caller.clone(),
                            ))
                            .into(),
                        )?;

                    Self::deposit_event(Event::MultisigExecuted(
                        multi_account_id::<T, T::IptId>(ips_id, old_data.include_original_caller),
                        old_data.actual_call,
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
                        multi_account_id::<T, T::IptId>(ips_id, old_data.include_original_caller),
                        voter_balance,
                        ipt.supply,
                        old_data.actual_call,
                    ));
                }

                Ok(().into())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T: Config> Pallet<T> {
        pub fn create(
            owner: T::AccountId,
            ips_id: T::IptId,
            endowed_accounts: Vec<(T::AccountId, <T as pallet::Config>::Balance)>,
        ) {
            Ipt::<T>::insert(
                ips_id,
                AssetDetails {
                    owner,
                    supply: endowed_accounts
                        .clone()
                        .into_iter()
                        .map(|(_, balance)| balance)
                        .sum(),
                    deposit: Default::default(),
                },
            );

            endowed_accounts
                .iter()
                .for_each(|(account, balance)| Balance::<T>::insert(ips_id, account, balance));
        }

        pub fn internal_mint(
            target: T::AccountId,
            ips_id: T::IptId,
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            Ipt::<T>::try_mutate(ips_id, |ipt| -> DispatchResult {
                Balance::<T>::try_mutate(ips_id, target, |balance| -> DispatchResult {
                    let old_balance = balance.take().unwrap_or_default();
                    *balance = Some(old_balance + amount);

                    let mut old_ipt = ipt.take().ok_or(Error::<T>::IptDoesntExist)?;
                    old_ipt.supply += amount;
                    *ipt = Some(old_ipt);

                    Ok(())
                })
            })
        }

        pub fn internal_burn(
            target: T::AccountId,
            ips_id: T::IptId,
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            Ipt::<T>::try_mutate(ips_id, |ipt| -> DispatchResult {
                Balance::<T>::try_mutate(ips_id, target, |balance| -> DispatchResult {
                    let old_balance = balance.take().ok_or(Error::<T>::IptDoesntExist)?;
                    *balance = Some(
                        old_balance
                            .checked_sub(&amount)
                            .ok_or(Error::<T>::NotEnoughAmount)?,
                    );

                    let mut old_ipt = ipt.take().ok_or(Error::<T>::IptDoesntExist)?;
                    old_ipt.supply = old_ipt
                        .supply
                        .checked_sub(&amount)
                        .ok_or(Error::<T>::NotEnoughAmount)?;
                    *ipt = Some(old_ipt);

                    Ok(())
                })
            })
        }
    }
}
