#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
use sp_std::convert::TryInto;

mod traits;

pub use pallet::*;
pub use traits::ParachainList;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::OriginFor;
    use pallet_inv4::origin::{ensure_multisig, INV4Origin};
    use sp_std::{vec, vec::Vec};
    use xcm::{latest::prelude::*, DoubleEncoded};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_inv4::Config + pallet_xcm::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Parachains: ParachainList;

        #[pallet::constant]
        type ParaId: Get<u32>;
    }

    #[pallet::error]
    pub enum Error<T> {
        SendingFailed,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        CallSent {
            sender: <T as pallet_inv4::Config>::IpId,
            destination: <T as pallet::Config>::Parachains,
            call: Vec<u8>,
        },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        Result<
            INV4Origin<T, <T as pallet_inv4::Config>::IpId, <T as frame_system::Config>::AccountId>,
            <T as frame_system::Config>::Origin,
        >: From<<T as frame_system::Config>::Origin>,

        <T as pallet_inv4::Config>::IpId: Into<u32>,
    {
        #[pallet::weight(100_000_000)]
        pub fn send_call(
            origin: OriginFor<T>,
            destination: <T as pallet::Config>::Parachains,
            weight: u64,
            call: Vec<u8>,
        ) -> DispatchResult {
            let core = ensure_multisig::<T, OriginFor<T>>(origin)?;
            let core_id = core.id.into();

            let interior = Junctions::X2(
                Junction::Parachain(<T as pallet::Config>::ParaId::get()),
                Junction::Plurality {
                    id: BodyId::Index(core_id),
                    part: BodyPart::Voice,
                },
            );

            let dest = destination.get_location();
            let dest_asset = destination.get_asset();
            let weight_to_fee = destination.get_weight_to_fee();

            let fee = weight as u128 * weight_to_fee;

            let fee_multiasset = MultiAsset {
                id: dest_asset,
                fun: Fungibility::Fungible(fee),
            };

            let message = Xcm(vec![
                Instruction::WithdrawAsset(fee_multiasset.clone().into()),
                Instruction::BuyExecution {
                    fees: fee_multiasset,
                    weight_limit: WeightLimit::Unlimited,
                },
                Instruction::Transact {
                    origin_type: OriginKind::Native,
                    require_weight_at_most: weight * 2,
                    call: <DoubleEncoded<_> as From<Vec<u8>>>::from(call.clone()),
                },
                Instruction::RefundSurplus,
                Instruction::DepositAsset {
                    assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                    max_assets: 1,
                    beneficiary: interior.clone().into(),
                },
            ]);

            pallet_xcm::Pallet::<T>::send_xcm(interior, dest, message)
                .map_err(|_| Error::<T>::SendingFailed)?;

            Self::deposit_event(Event::CallSent {
                sender: core.id,
                destination,
                call,
            });

            Ok(())
        }
    }
}
