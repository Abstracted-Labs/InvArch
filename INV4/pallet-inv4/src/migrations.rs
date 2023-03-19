use super::*;
use frame_support::{
    dispatch::GetStorageVersion,
    pallet_prelude::*,
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};
use log::{info, warn};

pub mod v1 {
    use core::convert::TryInto;
    use frame_support::{traits::fungibles::Mutate, BoundedVec};
    use primitives::{CoreInfo, OneOrPercent};
    use sp_core::H256;
    use sp_runtime::Perbill;

    use super::*;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    pub enum Parentage<AccountId, IpsId> {
        Parent(AccountId),
        Child(IpsId, AccountId),
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    enum IpsType<IpsId> {
        Normal,
        Replica(IpsId),
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    struct IpInfo<AccountId, Data, IpsMetadataOf, IpId, Balance, LicenseMetadata, Hash> {
        pub parentage: Parentage<AccountId, IpId>,
        pub metadata: IpsMetadataOf,
        pub data: Data,
        pub ips_type: IpsType<IpId>,
        pub allow_replica: bool,
        pub supply: Balance,
        pub license: (LicenseMetadata, Hash),
        pub execution_threshold: OneOrPercent,
        pub default_asset_weight: OneOrPercent,
        pub default_permission: bool,
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    enum AnyId<IpsId, IpfId, RmrkNftTuple, RmrkCollectionId> {
        IpfId(IpfId),
        RmrkNft(RmrkNftTuple),
        RmrkCollection(RmrkCollectionId),
        IpsId(IpsId),
    }

    type AnyIdOf = AnyId<u32, u64, (u32, u32), u32>;

    type IpsMetadataOf = BoundedVec<u8, ConstU32<10000>>;

    type IpInfoOf<T> = IpInfo<
        <T as frame_system::Config>::AccountId,
        BoundedVec<AnyIdOf, ConstU32<10000>>,
        IpsMetadataOf,
        u32,
        u128,
        BoundedVec<u8, ConstU32<10000>>,
        <T as frame_system::Config>::Hash,
    >;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    pub struct IpfInfo<AccountId, Data, IpfMetadataOf> {
        /// IPF owner
        pub owner: AccountId,
        /// Original IPF author
        pub author: AccountId,
        /// IPF metadata
        pub metadata: IpfMetadataOf,
        /// IPF data
        pub data: Data,
    }

    pub fn migrate_ip_storage_to_core_storage<T: Config>() {
        let total_ips = frame_support::migration::storage_key_iter::<
            u32,
            IpInfoOf<T>,
            Blake2_128Concat,
        >(b"INV4", b"IpStorage")
        .count();

        info!("Attempting to migrate {} IPS into Cores.", total_ips);

        let mut ips_migrated = 0;

        frame_support::migration::storage_key_iter::<u32, IpInfoOf<T>, Blake2_128Concat>(
            b"INV4",
            b"IpStorage",
        )
        .for_each(|(ips_id, ips)| {
            if let Parentage::Parent(account) = ips.parentage {
                CoreStorage::<T>::insert(
                    Into::<T::CoreId>::into(ips_id),
                    CoreInfo {
                        account: account.clone(),
                        metadata: ips
                            .metadata
                            .into_inner()
                            .try_into()
                            .expect("IPS metadata should always fit in Core metadata."),
                        minimum_support: match ips.execution_threshold {
                            OneOrPercent::One => Perbill::one(),
                            OneOrPercent::ZeroPoint(percent) => {
                                Perbill::from_percent(percent.deconstruct() as u32)
                            }
                        },
                        required_approval: Perbill::zero(),
                        frozen_tokens: true,
                    },
                );

                ips_migrated += 1;
            }
        });

        info!("Migrated {} IPS into Cores.", ips_migrated);
        info!("Extra check: {} Cores", CoreStorage::<T>::iter().count(),);
    }

    pub fn migrate_ip_owner_to_core_account<T: Config>() {
        let mut ips_migrated = 0;

        frame_support::migration::storage_key_iter::<(T::AccountId, T::CoreId), (), Blake2_128Concat>(
            b"INV4",
            b"IpsByOwner",
        )
        .for_each(|((account, ips_id), _)| {
            CoreByAccount::<T>::insert::<<T as frame_system::Config>::AccountId, T::CoreId>(
                account,
                ips_id,
            );

            ips_migrated += 1;
        });

        info!(
            "Migrated {} IPS accounts into CoreByAccount storage.",
            ips_migrated
        );
        info!("Extra check: {}", CoreByAccount::<T>::iter().count());
    }

    pub fn migrate_next_id<T: Config>() {
        let next_id =
            frame_support::migration::take_storage_value::<u32>(b"INV4", b"NextIpId", &[]).unwrap();

        NextCoreId::<T>::put::<T::CoreId>(next_id.into());

        info!("Migrated NextIpId {} into NextCoreId.", next_id);
        info!("Extra check: {}", NextCoreId::<T>::get());
    }

    pub fn migrate_balance_and_total_issuance<T: Config>() {
        let entries = frame_support::migration::storage_key_iter::<
            (u32, Option<u32>, T::AccountId),
            BalanceOf<T>,
            Blake2_128Concat,
        >(b"INV4", b"Balance")
        .count();

        info!(
            "Attempting to migrate {} entries from INV4.Balance storage.",
            entries
        );

        let mut migrated = 0;

        frame_support::migration::storage_key_iter::<
            (u32, Option<u32>, T::AccountId),
            BalanceOf<T>,
            Blake2_128Concat,
        >(b"INV4", b"Balance")
        .for_each(|((ips_id, token, account), balance)| {
            if token.is_none() {
                T::AssetsProvider::mint_into(ips_id.into(), &account, balance).unwrap();
            }

            migrated += 1;
        });

        info!("Migrated {} entries from Balance to Balances.", migrated);
    }

    pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
            frame_support::ensure!(
                Pallet::<T>::current_storage_version() == 0,
                "Required v0 before upgrading to v1"
            );

            Ok(Default::default())
        }

        fn on_runtime_upgrade() -> Weight {
            let current = Pallet::<T>::current_storage_version();

            if current == 1 {
                migrate_ip_storage_to_core_storage::<T>();
                migrate_ip_owner_to_core_account::<T>();
                migrate_next_id::<T>();
                migrate_balance_and_total_issuance::<T>();

                current.put::<Pallet<T>>();

                info!("v1 applied successfully");
                T::DbWeight::get().reads_writes(10, 10)
            } else {
                warn!("Skipping v1, should be removed");
                T::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
            frame_support::ensure!(
                Pallet::<T>::on_chain_storage_version() == 1,
                "v1 not applied"
            );

            Ok(())
        }
    }
}
