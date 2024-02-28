//! Provides supporting traits for the rings pallet.
//!
//! ## Overview
//!
//! This module contains the traits responsible for creating an abstraction layer on top of XCM [`MultiLocation`] and allows
//! easier handling of cross-chain transactions through XCM.
//!
//! The traits contained in this pallet require an appropriate runtime implementation.
//!
//! ## Traits overview:
//!
//! - [`ChainList`] - Trait used to opaquely refer to a chain, provides an interface to get the chain `MultiLocation` or the chain's main asset as `ChainAssetsList`.
//! - [`ChainAssetsList`] - Trait used to opaquely refer to a chain's asset, provides an interface to get the chain asset `MultiLocation` or the chain as `ChainList`.

use codec::MaxEncodedLen;
use frame_support::Parameter;
use xcm::latest::MultiLocation;

/// A chain [`MultiLocation`] abstraction trait.
///
/// It provides an interface for easily getting a chain's [`MultiLocation`] and to go back and forth between the chain and its assets.
///
/// This should be implemented properly in the runtime.
pub trait ChainList: Parameter + MaxEncodedLen {
    type Balance: Into<u128>;
    type ChainAssets: ChainAssetsList;

    /// Returns the chain's [`MultiLocation`].
    fn get_location(&self) -> MultiLocation;

    /// Returns the chain's main asset as `ChainAssetsList`.
    fn get_main_asset(&self) -> Self::ChainAssets;

    #[cfg(feature = "runtime-benchmarks")]
    fn benchmark_mock() -> Self;
}

/// A chain asset [`MultiLocation`] abstraction trait.
///
/// It provides an interface for easily getting a chain's asset [`MultiLocation`] and to go back and forth between the asset and its parent chain.
///
/// This should be implemented properly in the runtime.
pub trait ChainAssetsList: Parameter + MaxEncodedLen {
    type Chains: ChainList;

    /// Returns this asset parent chain.
    fn get_chain(&self) -> Self::Chains;

    /// Returns the asset's [`MultiLocation`].
    fn get_asset_location(&self) -> MultiLocation;
}
