#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};

/// IPS Id type
pub type IpsId = u64;
/// IPT Id type
pub type IptId = u64;
/// IPO id type
pub type IpoId = u64;

/// IPS info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen)]
pub struct IpsInfo<IptId, AccountId, Data, IpsMetadataOf> {
    // TODO: WIP
    /// IPS metadata
    pub metadata: IpsMetadataOf,
    /// Total issuance for the IPS
    pub total_issuance: IptId,
    /// IPS owner
    pub owner: AccountId,
    /// IPS Properties
    pub data: Data,
}

/// IPT Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen)]
pub struct IptInfo<AccountId, Data, IptMetadataOf> {
    /// IPT owner
    pub owner: AccountId,
    /// IPT metadata
    pub metadata: IptMetadataOf,
    /// IPT data
    pub data: Data,
}

/// IPO Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen)]
pub struct IpoInfo<IpsId, AccountId, Data, IpoMetadataOf> {
    // TODO: WIP
    /// IPO metadata
    pub metadata: IpoMetadataOf,
    /// Total issuance for the IPO
    pub total_issuance: IpsId,
    /// IPO owner
    pub owner: AccountId,
    /// IPO Properties
    pub data: Data,
    /// Binding Properties
    pub is_bond: bool,
}
