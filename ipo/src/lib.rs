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
    traits::{Currency, Get, StoredMap, WithdrawReasons},
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
use primitives::{IpoInfo, IpsInfo};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The IPO ID type
        type IpoId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy; // TODO: WIP
        /// The IPO properties type
        type IpoData: Parameter + Member + MaybeSerializeDeserialize; // TODO: WIP
        /// The IPS ID type
        type IpsId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// IPT properties type
        type IpsData: Parameter + Member + MaybeSerializeDeserialize; // TODO: WIP
        /// The maximum size of an IPS's metadata
        type MaxIpoMetadata: Get<u32>; // TODO: WIP
        /// The maximum size of an IPT's metadata
        type MaxIpsMetadata: Get<u32>;
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
        /// The means of storing the balances of an account.
        type AccountStore: StoredMap<Self::AccountId, AccountData<Self::Balance>>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type IpoIndexOf<T> = <T as Config>::IpoId;
    pub type IpoMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpoMetadata>;
    pub type IpsMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIpsMetadata>;
    pub type IpoInfoOf<T> = IpoInfo<
        <T as Config>::IpsId,
        <T as frame_system::Config>::AccountId,
        <T as Config>::IpoData,
        IpoMetadataOf<T>,
    >;
    pub type IpsInfoOf<T> = IpsInfo<
        <T as frame_system::Config>::AccountId,
        <T as Config>::IpsData,
        IpoMetadataOf<T>,
        IpsMetadataOf<T>,
    >;

    pub type GenesisIpsData<T> = (
        <T as frame_system::Config>::AccountId, // IPS owner
        Vec<u8>,                                // IPS metadata
        <T as Config>::IpsData,                 // IPS data
    );
    pub type GenesisIpo<T> = (
        <T as frame_system::Config>::AccountId, // IPO owner
        Vec<u8>,                                // IPO metadata
        <T as Config>::IpoData,                 // IPO data
        Vec<GenesisIpsData<T>>,                 // Vector of IPSs belong to this IPO
    );

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Next available IPO ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ipo_id)]
    pub type NextIpoId<T: Config> = StorageValue<_, T::IpoId, ValueQuery>;

    /// Next available IPS ID
    #[pallet::storage]
    #[pallet::getter(fn next_ips_id)]
    pub type NextIpsId<T: Config> = StorageMap<_, Blake2_128Concat, T::IpoId, T::IpsId, ValueQuery>;

    /// Store IPO info
    ///
    /// Return `None` if IPO info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ipo_storage)]
    pub type IpoStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IpoId, IpoInfoOf<T>>;

    /// Store IPS info
    ///
    /// Returns `None` if IPS info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ips_storage)]
    pub type IpsStorage<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::IpoId, Blake2_128Concat, T::IpsId, IpsInfoOf<T>>;

    /// IPS existence check by owner and IPO ID
    #[pallet::storage]
    #[pallet::getter(fn get_balance)]
    pub type BalanceToAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

    /// Get IPO price. None means not for sale.
    #[pallet::storage]
    #[pallet::getter(fn ipo_prices)]
    pub type IpoPrices<T: Config> =
        StorageMap<_, Blake2_128Concat, IpoInfoOf<T>, BalanceOf<T>, OptionQuery>;

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

    /// All balance information for an account.
    #[derive(
        Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
    )]
    pub struct AccountData<Balance> {
        /// Non-reserved part of the balance. There may still be restrictions on this, but it is the
        /// total pool what may in principle be transferred, reserved and used for tipping.
        ///
        /// This is the only balance that matters in terms of most operations on tokens. It
        /// alone is used to determine the balance when in the contract execution environment.
        pub free: Balance,
        /// Balance which is reserved and may not be used at all.
        ///
        /// This can still get slashed, but gets slashed last of all.
        ///
        /// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
        /// that are still 'owned' by the account holder, but which are suspendable.
        /// This includes named reserve and unnamed reserve.
        pub reserved: Balance,
        /// The amount that `free` may not drop below when withdrawing for *anything except transaction
        /// fee payment*.
        pub misc_frozen: Balance,
        /// The amount that `free` may not drop below when withdrawing specifically for transaction
        /// fee payment.
        pub fee_frozen: Balance,
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
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    /// Create IP (Intellectual Property) Ownership (IPO)
    pub fn issue_ipo(
        // TODO: WIP
        owner: &T::AccountId,
        metadata: Vec<u8>,
        data: T::IpoData,
    ) -> Result<T::IpoId, DispatchError> {
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
            owner: owner.clone(),
            data,
        };
        IpoStorage::<T>::insert(ipo_id, info);

        Ok(ipo_id)
    }

    /// Transfer some liquid free IPO balance to another account
    /// Is a no-op if value to be transferred is zero or the `from` is the same as `to`.
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
        Ok(().into())
    }

    pub fn set_balance(
        origin: OriginFor<T>,
        new_free: T::Balance,
        new_reserved: T::Balance,
    ) -> DispatchResultWithPostInfo {
        ensure_root(origin)?;

        let existential_deposit = T::ExistentialDeposit::get();

        let wipeout = new_free + new_reserved < existential_deposit;
        let new_free = if wipeout { Zero::zero() } else { new_free };
        let new_reserved = if wipeout { Zero::zero() } else { new_reserved };

        // TODO : WIP [need help]
        // - Add more logic for free and reserved balance

        (new_free, new_reserved);

        Ok(().into())
    }

    /// Get the free balance of an account.
    pub fn free_balance(who: impl sp_std::borrow::Borrow<T::AccountId>) -> T::Balance {
        Self::account(who.borrow()).free
    }

    /// Get the reserved balance of an account.
    pub fn reserved_balance(who: impl sp_std::borrow::Borrow<T::AccountId>) -> T::Balance {
        Self::account(who.borrow()).reserved
    }

    /// Get both the free and reserved balances of an account.
    fn account(who: &T::AccountId) -> AccountData<T::Balance> {
        T::AccountStore::get(who)
    }

    // TODO: WIP
    // Redundancy with get_balance from storage in line #156
    // Should we remove get free and reserved balance and just use get_balance from storage above?
}

// TODO: WIP

// - Add total_supply function
// - Add bind function
// - Add unbind function
