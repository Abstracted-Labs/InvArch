#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::Percent;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum OneOrPercent {
    One,
    ZeroPoint(Percent),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Parentage<AccountId, IpsId> {
    /// Parent IP (Account Id of itself)
    Parent(AccountId),
    /// Child IP (Id of the immediate parent, Account Id of the topmost parent)
    Child(IpsId, AccountId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum IpsType<IpsId> {
    /// Normal IPS
    Normal,
    /// IP Replica (Id of the original IP)
    Replica(IpsId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub struct IpInfo<AccountId, Data, IpsMetadataOf, IpId, Balance, LicenseMetadata, Hash> {
    /// IPS parentage
    pub parentage: Parentage<AccountId, IpId>,
    /// IPS metadata
    pub metadata: IpsMetadataOf,
    /// IPS Properties
    pub data: Data,
    /// IPS Type
    pub ips_type: IpsType<IpId>,
    /// If this IPS allows replicas
    pub allow_replica: bool,

    pub supply: Balance,

    pub license: (LicenseMetadata, Hash),
    pub execution_threshold: OneOrPercent,
    pub default_asset_weight: OneOrPercent,
    pub default_permission: bool,
}

/// IPF Info
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

// This is a struct in preparation for having more fields in the future.
#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo)]
pub struct SubIptInfo<IptId, SubAssetMetadata> {
    pub id: IptId,
    pub metadata: SubAssetMetadata,
}

#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo)]
pub struct CallInfo<Data> {
    pub pallet: Data,
    pub function: Data,
}

pub mod utils {
    use codec::{Decode, Encode};
    use sp_io::hashing::blake2_256;
    use sp_runtime::traits::TrailingZeroInput;

    pub fn multi_account_id<T: frame_system::Config, IpsId: Encode>(
        ips_id: IpsId,
        original_caller: Option<T::AccountId>,
    ) -> <T as frame_system::Config>::AccountId {
        let entropy = if let Some(original_caller) = original_caller {
            (b"modlpy/utilisuba", ips_id, original_caller).using_encoded(blake2_256)
        } else {
            (b"modlpy/utilisuba", ips_id).using_encoded(blake2_256)
        };

        Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .expect("infinite length input; no invalid inputs for type; qed")
    }
}
