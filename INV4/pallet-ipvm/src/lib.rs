#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, traits::Currency as FSCurrency};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;

//#[cfg(test)]
//mod mock;
//#[cfg(test)]
//mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_core::crypto::UncheckedFrom;
    use sp_runtime::traits::{CheckedAdd, Hash, StaticLookup};
    use sp_std::vec;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + ips::Config
        + ipf::Config
        + pallet_contracts::Config
        + pallet_balances::Config
    {
        /// The IPS Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Currency
        type Currency: FSCurrency<Self::AccountId>;

        /// The minimum amount required to keep an account open.
        #[pallet::constant]
        type ExistentialDeposit: Get<
            <<Self as Config>::Currency as FSCurrency<<Self as frame_system::Config>::AccountId>>::Balance,
        >;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as FSCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type ContractsBalanceOf<T> = <<T as pallet_contracts::Config>::Currency as FSCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    pub type BalancesBalanceOf<T> = <<T as pallet_contracts::Config>::Currency as FSCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        Created(T::AccountId, T::IpsId),
    }

    /// Errors for SmartIP pallet
    #[pallet::error]
    pub enum Error<T> {
        BalanceOverflow,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: UncheckedFrom<T::Hash>,
        T::AccountId: AsRef<[u8]>,

        <T as pallet_balances::Config>::Balance: From<
            <<T as pallet::Config>::Currency as FSCurrency<
                <T as frame_system::Config>::AccountId,
            >>::Balance,
        >,
        <<T as pallet_contracts::Config>::Currency as FSCurrency<
            <T as frame_system::Config>::AccountId,
        >>::Balance: From<
            <<T as pallet::Config>::Currency as FSCurrency<
                <T as frame_system::Config>::AccountId,
            >>::Balance,
        >,
    {
        /// Create IP (Intellectual Property) Set (IPS)
        #[pallet::weight(10000)] // TODO
        pub fn create(
            owner: OriginFor<T>,
            code: Vec<u8>,
            data: Vec<u8>,
            endowment: BalanceOf<T>,
            gas_limit: Weight,
            allow_replica: bool,
        ) -> DispatchResultWithPostInfo
        where
            <T as pallet_balances::Config>::Balance: From<
                <<T as pallet::Config>::Currency as FSCurrency<
                    <T as frame_system::Config>::AccountId,
                >>::Balance,
            >,
            <<T as pallet_contracts::Config>::Currency as FSCurrency<
                <T as frame_system::Config>::AccountId,
            >>::Balance: From<
                <<T as pallet::Config>::Currency as FSCurrency<
                    <T as frame_system::Config>::AccountId,
                >>::Balance,
            >,
        {
            let ips_id: <T as ips::Config>::IpsId = ips::NextIpsId::<T>::get();
            let ipf_id: <T as ipf::Config>::IpfId = ipf::NextIpfId::<T>::get();

            ipf::Pallet::<T>::mint(owner.clone(), vec![], T::Hashing::hash(&code))?;

            // TODO: WASM to WAT
            // TODO: Mint WAT IPF

            let ips_account: <T as frame_system::Config>::AccountId =
                primitives::utils::multi_account_id::<T, <T as ips::Config>::IpsId>(ips_id, None);

            ips::Pallet::<T>::create_ips(owner.clone(), vec![], vec![ipf_id], allow_replica)?;

            pallet_balances::Pallet::<T>::transfer(
                owner,
                <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                    ips_account.clone(),
                ),
                endowment
                    .checked_add(&<T as pallet::Config>::ExistentialDeposit::get())
                    .ok_or(Error::<T>::BalanceOverflow)?
                    .into(),
            )?;

            pallet_contracts::Pallet::<T>::bare_instantiate(
                ips_account.clone(),
                endowment.into(),
                gas_limit,
                Some(endowment.into()),
                pallet_contracts_primitives::Code::Existing(
                    pallet_contracts::Pallet::<T>::bare_upload_code(
                        ips_account.clone(),
                        code,
                        Some(endowment.into()),
                    )?
                    .code_hash,
                ),
                data,
                vec![],
                true,
            )
            .result?;

            Self::deposit_event(Event::Created(ips_account, ips_id));

            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
