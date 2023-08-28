use core::marker::PhantomData;

use crate::{Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::{GetDispatchInfo, Parameter};
use pallet_nft_origins::{Chain, ChainVerifier};
use scale_info::TypeInfo;
use sp_runtime::traits::Dispatchable;
use xcm::latest::Junction;
use xcm_executor::traits::CallDispatcher;

pub enum RegisteredChains {
    Moonriver,
    TinkernetTest,
}

impl ChainVerifier for RegisteredChains {
    fn get_chain_from_verifier(para_id: u32, verifier_part: Junction) -> Option<Chain> {
        match (para_id, verifier_part) {
            // Moonriver
            (
                2023,
                Junction::AccountKey20 {
                    network: None,
                    key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
            ) => Some(Chain::Parachain(2023)),

            (2126, Junction::GeneralIndex(69)) => Some(Chain::Parachain(2126)),

            _ => None,
        }
    }
}

impl pallet_nft_origins::Config for Runtime {
    type Chains = RegisteredChains;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type RuntimeOrigin = RuntimeOrigin;
}

// #[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
// pub struct TestNftDispatch<RuntimeCall>(PhantomData<RuntimeCall>);

// impl<RuntimeCall: Dispatchable> Dispatchable for TestNftDispatch<RuntimeCall> {
//     type RuntimeOrigin = <RuntimeCall as Dispatchable>::RuntimeOrigin;
//     type Config = <RuntimeCall as Dispatchable>::Config;
//     type Info = <RuntimeCall as Dispatchable>::Info;
//     type PostInfo = <RuntimeCall as Dispatchable>::PostInfo;

//     fn dispatch(
//         self,
//         origin: Self::RuntimeOrigin,
//     ) -> sp_runtime::DispatchResultWithInfo<Self::PostInfo> {
//         Ok(pallet_nft_origins::Pallet::<Runtime>::test_nft_location(origin.into())?.into())
//     }
// }

// impl<RuntimeCall: Dispatchable> GetDispatchInfo for TestNftDispatch<RuntimeCall> {
//     fn get_dispatch_info(&self) -> frame_support::dispatch::DispatchInfo {
//         pallet_nft_origins::pallet::Call::<Runtime>::test_nft_location {}.get_dispatch_info()
//     }
// }

// //#[derive(Encode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
// pub enum TryBoth<RuntimeCall> {
//     RuntimeCall(RuntimeCall),
//     TestNftDispatch(TestNftDispatch<RuntimeCall>),
// }

// impl<RuntimeCall: Decode> Decode for TryBoth<RuntimeCall> {
//     fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
//         Ok(RuntimeCall::decode(input)
//             .map(|decoded_rc| TryBoth::<RuntimeCall>::RuntimeCall(decoded_rc))
//             .unwrap_or(
//                 TestNftDispatch::decode(input)
//                     .map(|decoded_tnd| TryBoth::<RuntimeCall>::TestNftDispatch(decoded_tnd))?,
//             ))
//     }
// }

// impl<RuntimeCall> Dispatchable for TryBoth<RuntimeCall>
// where
//     RuntimeCall: Dispatchable,
// {
//     type RuntimeOrigin = <RuntimeCall as Dispatchable>::RuntimeOrigin;
//     type Config = <RuntimeCall as Dispatchable>::Config;
//     type Info = <RuntimeCall as Dispatchable>::Info;
//     type PostInfo = <RuntimeCall as Dispatchable>::PostInfo;

//     fn dispatch(
//         self,
//         origin: Self::RuntimeOrigin,
//     ) -> sp_runtime::DispatchResultWithInfo<Self::PostInfo> {
//         match self {
//             Self::RuntimeCall(rc) => rc.dispatch(origin),
//             Self::TestNftDispatch(tnd) => tnd.dispatch(origin),
//         }
//     }
// }

// impl<RuntimeCall: GetDispatchInfo> GetDispatchInfo for TryBoth<RuntimeCall> {
//     fn get_dispatch_info(&self) -> frame_support::dispatch::DispatchInfo {
//         match self {
//             Self::RuntimeCall(rc) => rc.get_dispatch_info(),
//             Self::TestNftDispatch(tnd) => tnd.get_dispatch_info(),
//         }
//     }
// }
