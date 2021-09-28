//! # IPO
//! IP Ownership
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to create and manage IP Ownership tokens, which reflect ownership over an IP Set and governing weight in a DEV's governance.
//!
//! ### Pallet Functions
//!
//! `issue` - Issues the total supply of a new fungible asset to the account of the caller of the function
//! `transfer` - Transfer some liquid free balance to another account
//! `set_balance` - Set the balances to a given account. The origin of this call mus be root
//! `get_balance` - Get the asset `id` balance of `who`
//! `total_supply` - Get the total supply of an asset `id`
//! `bind` - Bind some `amount` of unit of fungible asset `id` from the ballance of the function caller's account (`origin`) to a specific `IPSet`
//! account to claim some portion of fractionalized ownership of that particular `IPset`
//! `unbind` - Unbind some `amount` of unit of fungible asset `id` from a specific `IPSet` account to unclaim some portion of fractionalized ownership
//! to the ballance of the function caller's account.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::no_effect)]

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, Get, WithdrawReasons},
    BoundedVec, Parameter,
};

use frame_system::{ensure_signed, pallet_prelude::*};

use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, CheckedAdd, MaybeSerializeDeserialize, Member, One, Saturating, Zero,
    },
    DispatchError,
};

use scale_info::TypeInfo;
use sp_std::{convert::TryInto, ops::BitOr, vec::Vec};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

use codec::{Codec, MaxEncodedLen};
use sp_std::{fmt::Debug, prelude::*};

/// Import from primitives pallet
use primitives::IpoInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + ips::Config {
        /// Overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The IPO ID type
        type IpoId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The IPO properties type
        type IpoData: Parameter + Member + MaybeSerializeDeserialize;
        /// The maximum size of an IPS's metadata
        type MaxIpoMetadata: Get<u32>;
        /// Currency
        type Currency: Currency<Self::AccountId>;
        /// The balance of an account
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + TypeInfo;
        /// The minimum amount required to keep an account open.
        #[pallet::constant]
        type ExistentialDeposit: Get<Self::Balance>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type IpoIndexOf<T> = <T as Config>::IpoId;

    pub type IpoMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpoMetadata>;

    pub type IpoInfoOf<T> =
        IpoInfo<<T as frame_system::Config>::AccountId, <T as Config>::IpoData, IpoMetadataOf<T>>;

    pub type GenesisIpo<T> = (
        <T as frame_system::Config>::AccountId, // IPO owner
        Vec<u8>,                                // IPO metadata
        <T as Config>::IpoData,                 // IPO data
        Vec<ips::GenesisIps<T>>,                // Vector of IPSs belong to this IPO
    );

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Next available IPO ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ipo_id)]
    pub type NextIpoId<T: Config> = StorageValue<_, T::IpoId, ValueQuery>;

    /// Store IPO info
    ///
    /// Return `None` if IPO info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ipo_storage)]
    pub type IpoStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpoId, IpoInfoOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn get_balance)]
    pub type BalanceToAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

    /// Get IPO price. None means not for sale.
    #[pallet::storage]
    #[pallet::getter(fn ipo_prices)]
    pub type IpoPrices<T: Config> =
        StorageMap<_, Blake2_128Concat, IpoInfoOf<T>, BalanceOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Some IPO were issued. \[ipo_id, owner, total_supply\]
        Issued(T::IpoId, T::AccountId, T::Balance),
        /// Some IPO wes transferred. \[ipo_id\]
        Transferred(T::AccountId, T::AccountId, T::Balance),
        /// Some IPO was bond. \[ipo_id\]
        IpoBond(T::IpoId),
        /// Some IPO was unbind. \[ipo_id\]
        IpoUnbind(T::IpoId),
    }

    /// Simplified reasons for withdrawing balance.
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub enum Reasons {
        /// Paying system transaction fees.
        Fee = 0_isize,
        /// Any reason other than paying system transaction fees.
        Misc = 1_isize,
        /// Any reason at all.
        All = 2_isize,
    }

    impl From<WithdrawReasons> for Reasons {
        fn from(r: WithdrawReasons) -> Reasons {
            if r == WithdrawReasons::TRANSACTION_PAYMENT {
                Reasons::Fee
            } else if r.contains(WithdrawReasons::TRANSACTION_PAYMENT) {
                Reasons::All
            } else {
                Reasons::Misc
            }
        }
    }

    impl BitOr for Reasons {
        type Output = Reasons;
        fn bitor(self, other: Reasons) -> Reasons {
            if self == other {
                return self;
            }
            Reasons::All
        }
    }

    /// Errors for IPO pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPO ID
        NoAvailableIpoId,
        /// No available IPS ID
        NoAvailableIpsId,
        /// IPS (IpoId, IpsId) not found
        IpsNotFound,
        /// IPO not found
        IpoNotFound,
        /// The operator is not the owner of the IPS and has no permission
        NoPermission,
        /// The IPO is already owned
        AlreadyOwned,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Buy IPO from their self
        BuyFromSelf,
        /// IPO is not for sale
        NotForSale,
        /// Buy price is too low
        PriceTooLow,
        /// Can not destroy IPO
        CannotDestroyIpo,
        /// The balance is insufficient
        InsufficientBalance,
        /// The given IPO ID is unknown
        Unknown,
        /// Balance less than existential deposit
        NotEnoughBalance,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create IP (Intellectual Property) Ownership (IPO)
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn issue_ipo(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            data: T::IpoData,
            total_issuance: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(owner)?;

            let bounded_metadata: BoundedVec<u8, T::MaxIpoMetadata> = metadata
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

            let ipo_id = NextIpoId::<T>::try_mutate(|id| -> Result<T::IpoId, DispatchError> {
                let current_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableIpoId)?;
                Ok(current_id)
            })?;

            let info = IpoInfo {
                metadata: bounded_metadata,
                total_issuance: Default::default(),
                owner: signer.clone(),
                data,
                is_bond: false,
            };
            IpoStorage::<T>::insert(ipo_id, info);
            Self::deposit_event(Event::Issued(ipo_id, signer, total_issuance));
            Ok(().into())
        }

        /// Transfer some liquid free IPO balance to another account
        /// Is a no-op if value to be transferred is zero or the `from` is the same as `to`.
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            if amount.is_zero() || sender == to {
                return Ok(().into());
            }

            BalanceToAccount::<T>::mutate(&sender, |bal| {
                *bal = bal.saturating_sub(amount);
            });
            BalanceToAccount::<T>::mutate(&to, |bal| {
                *bal = bal.saturating_add(amount);
            });
            Self::deposit_event(Event::Transferred(sender, to, amount));
            Ok(().into())
        }

        /// Set the balances to a given account. The origin of this call must be root.
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn set_balance(
            origin: OriginFor<T>,
            new_balance: T::Balance,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let existential_deposit = T::ExistentialDeposit::get();

            ensure!(
                new_balance > existential_deposit,
                Error::<T>::NotEnoughBalance
            );

            Ok(().into())
        }

        /// Bind some `amount` of unit of fungible `ipo_id` from the ballance of the function caller's account (`origin`) to a specific `IPSet` account to claim some portion of fractionalized ownership of that particular `IPset`
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn bind(origin: OriginFor<T>, ipo_id: T::IpoId) -> DispatchResult {
            let origin = ensure_signed(origin)?;

            IpoStorage::<T>::try_mutate(ipo_id, |maybe_details| {
                let d = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(origin == d.owner, Error::<T>::NoPermission);

                d.is_bond = true;

                Self::deposit_event(Event::<T>::IpoBond(ipo_id));
                Ok(())
            })
        }

        /// Unbind some `amount` of unit of fungible `ipo_id` from a specific `IPSet` account to unclaim some portion of fractionalized ownership to the ballance of the function caller's account'
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn unbind(origin: OriginFor<T>, ipo_id: T::IpoId) -> DispatchResult {
            let origin = ensure_signed(origin)?;

            IpoStorage::<T>::try_mutate(ipo_id, |maybe_details| {
                let d = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(origin == d.owner, Error::<T>::NoPermission);

                d.is_bond = false;

                Self::deposit_event(Event::<T>::IpoUnbind(ipo_id));
                Ok(())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
