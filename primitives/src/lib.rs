#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};

/// IPS Id type
pub type IpsId = u64;
/// IPT Id type
pub type IptId = u64;
/// IPO id type
pub type IpoId = u64;
/// DEV id type
pub type DevId = u64;

/// IPS info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug)]
pub struct IpsInfo<AccountId, Data, IpsMetadataOf> {
    /// IPS owner
    pub owner: AccountId,
    /// IPS metadata
    pub metadata: IpsMetadataOf,
    /// IPS Properties
    pub data: Data,
}

/// IPT Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug)]
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
pub struct IpoInfo<AccountId, Data, IpoMetadataOf> {
    /// IPO metadata
    pub metadata: IpoMetadataOf,
    /// Total issuance for the IPO
    pub total_issuance: u64,
    /// IPO owner
    pub owner: AccountId,
    /// IPO Properties
    pub data: Data,
    /// Binding Properties
    pub is_bond: bool,
}

/// DEV Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug)]
pub struct DevInfo<AccountId, Data, DevMetadataOf> {
    /// DEV owner
    pub owner: AccountId,
    /// DEV metadata
    pub metadata: DevMetadataOf,
    /// DEV data
    pub data: Data,
    /// IPO allocations for DEV
    pub ipo_allocations: u8,
    /// DEV interactions
    pub interactions: u8,
    /// DEV post as joinable
    pub is_joinable: bool,
}
