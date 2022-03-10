//! # Pallet IPS
//! Intellectual Property Sets
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to create and manage IP Sets, which are sets of tokenized IP components, or IP Tokens.
//!
//! ### Pallet Functions
//!
//! - `create` - Create a new IP Set
//! - `send` - Transfer IP Set owner account address
//! - `list` - List an IP Set for sale
//! - `buy` - Buy an IP Set
//! - `destroy` - Delete an IP Set and all of its contents

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
    pallet_prelude::*,
    traits::{Currency as FSCurrency, Get},
    BoundedVec, Parameter,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, Member, One};
use sp_std::{convert::TryInto, vec::Vec};

/// Import the primitives crate
use primitives::IpsInfo;

pub use pallet::*;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetDetails<Balance, AccountId> {
    owner: AccountId,
    /// The total supply across all accounts.
    supply: Balance,
    /// The balance deposited for this asset. This pays for the data stored here.
    deposit: Balance,
}

#[frame_support::pallet]
pub mod pallet {
    use core::iter::Sum;

    use super::*;
    use primitives::utils::multi_account_id;
    use primitives::{AnyId, Parentage};
    use scale_info::prelude::fmt::Display;
    use scale_info::prelude::format;
    use sp_runtime::traits::{CheckedSub, StaticLookup};

    #[pallet::config]
    pub trait Config: frame_system::Config {
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
            + Sum<Self::Balance>;

        /// The IPS ID type
        type IptId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen;

        #[pallet::constant]
        type ExistentialDeposit: Get<Self::Balance>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as FSCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

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
    }

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        IptDoesntExist,
        NoPermission,
        NotEnoughAmount,
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

            Pallet::<T>::internal_mint(target, ips_id, amount)?;

            Self::deposit_event(Event::Minted(ips_id, owner, amount));

            Ok(().into())
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

                    let old_ipt = ipt.take().ok_or(Error::<T>::IptDoesntExist)?;
                    old_ipt
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
