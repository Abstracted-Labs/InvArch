//! Track configurations for governance.

use super::*;

const fn percent(x: i32) -> sp_arithmetic::FixedI64 {
    sp_arithmetic::FixedI64::from_rational(x as u128, 100)
}
use pallet_referenda::Curve;
const APP_ROOT: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_ROOT: Curve = Curve::make_linear(28, 28, percent(0), percent(50));
const APP_COUNCIL_ADMIN: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_COUNCIL_ADMIN: Curve =
    Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_GENERAL_MANAGEMENT: Curve =
    Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_GENERAL_MANAGEMENT: Curve =
    Curve::make_reciprocal(7, 28, percent(10), percent(0), percent(50));
const APP_REFERENDUM_CANCELLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_CANCELLER: Curve =
    Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_REFERENDUM_KILLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_KILLER: Curve =
    Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_SMALL_SPENDER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_SMALL_SPENDER: Curve =
    Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_BIG_SPENDER: Curve = Curve::make_linear(28, 28, percent(50), percent(100));
const SUP_BIG_SPENDER: Curve = Curve::make_reciprocal(20, 28, percent(1), percent(0), percent(50));
const APP_WHITELISTED_CALLER: Curve =
    Curve::make_reciprocal(16, 28 * 24, percent(96), percent(50), percent(100));
const SUP_WHITELISTED_CALLER: Curve =
    Curve::make_reciprocal(1, 28, percent(20), percent(5), percent(50));
const TRACKS_DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 8] = [
    (
        0,
        pallet_referenda::TrackInfo {
            name: "root",
            max_deciding: 1,
            decision_deposit: 100 * GRAND,
            #[cfg(not(feature = "on-chain-release-build"))]
            prepare_period: 4 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            prepare_period: 2 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            decision_period: 10 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            decision_period: 28 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            confirm_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            confirm_period: 7 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            min_enactment_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            min_enactment_period: 24 * HOURS,
            min_approval: APP_ROOT,
            min_support: SUP_ROOT,
        },
    ),
    (
        1,
        pallet_referenda::TrackInfo {
            name: "whitelisted_caller",
            max_deciding: 100,
            decision_deposit: 10 * GRAND,
            #[cfg(not(feature = "on-chain-release-build"))]
            prepare_period: 2 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            prepare_period: 30 * MINUTES,
            #[cfg(not(feature = "on-chain-release-build"))]
            decision_period: 10 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            decision_period: 28 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            confirm_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            confirm_period: 10 * MINUTES,
            #[cfg(not(feature = "on-chain-release-build"))]
            min_enactment_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_WHITELISTED_CALLER,
            min_support: SUP_WHITELISTED_CALLER,
        },
    ),
    (
        2,
        pallet_referenda::TrackInfo {
            name: "general_management",
            max_deciding: 10,
            decision_deposit: 10 * GRAND,
            #[cfg(not(feature = "on-chain-release-build"))]
            prepare_period: 4 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            prepare_period: 2 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            decision_period: 10 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            decision_period: 28 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            confirm_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            confirm_period: 24 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            min_enactment_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_GENERAL_MANAGEMENT,
            min_support: SUP_GENERAL_MANAGEMENT,
        },
    ),
    (
        13,
        pallet_referenda::TrackInfo {
            name: "council_admin",
            max_deciding: 10,
            decision_deposit: 10 * GRAND,
            #[cfg(not(feature = "on-chain-release-build"))]
            prepare_period: 4 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            prepare_period: 2 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            decision_period: 10 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            decision_period: 28 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            confirm_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            confirm_period: 3 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            min_enactment_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_COUNCIL_ADMIN,
            min_support: SUP_COUNCIL_ADMIN,
        },
    ),
    (
        20,
        pallet_referenda::TrackInfo {
            name: "referendum_canceller",
            max_deciding: 1_000,
            decision_deposit: 10 * GRAND,
            #[cfg(not(feature = "on-chain-release-build"))]
            prepare_period: 4 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            prepare_period: 2 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            decision_period: 10 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            decision_period: 7 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            confirm_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            confirm_period: 3 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            min_enactment_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_REFERENDUM_CANCELLER,
            min_support: SUP_REFERENDUM_CANCELLER,
        },
    ),
    (
        21,
        pallet_referenda::TrackInfo {
            name: "referendum_killer",
            max_deciding: 1_000,
            decision_deposit: 50 * GRAND,
            #[cfg(not(feature = "on-chain-release-build"))]
            prepare_period: 4 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            prepare_period: 2 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            decision_period: 10 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            decision_period: 28 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            confirm_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            confirm_period: 3 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            min_enactment_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_REFERENDUM_KILLER,
            min_support: SUP_REFERENDUM_KILLER,
        },
    ),
    (
        32,
        pallet_referenda::TrackInfo {
            name: "small_spender",
            max_deciding: 50,
            decision_deposit: 100 * UNIT,
            #[cfg(not(feature = "on-chain-release-build"))]
            prepare_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            prepare_period: 4 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            decision_period: 10 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            decision_period: 28 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            confirm_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            confirm_period: 2 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            min_enactment_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            min_enactment_period: 24 * HOURS,
            min_approval: APP_SMALL_SPENDER,
            min_support: SUP_SMALL_SPENDER,
        },
    ),
    (
        34,
        pallet_referenda::TrackInfo {
            name: "big_spender",
            max_deciding: 50,
            decision_deposit: 400 * UNIT,
            #[cfg(not(feature = "on-chain-release-build"))]
            prepare_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            prepare_period: 4 * HOURS,
            #[cfg(not(feature = "on-chain-release-build"))]
            decision_period: 10 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            decision_period: 28 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            confirm_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            confirm_period: 7 * DAYS,
            #[cfg(not(feature = "on-chain-release-build"))]
            min_enactment_period: 5 * MINUTES,
            #[cfg(feature = "on-chain-release-build")]
            min_enactment_period: 24 * HOURS,
            min_approval: APP_BIG_SPENDER,
            min_support: SUP_BIG_SPENDER,
        },
    ),
];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
    type Id = u16;
    type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
    fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
        &TRACKS_DATA[..]
    }
    fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
        if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
            match system_origin {
                frame_system::RawOrigin::Root => Ok(0),
                _ => Err(()),
            }
        } else if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
            match custom_origin {
                origins::Origin::WhitelistedCaller => Ok(1),
                origins::Origin::GeneralManagement => Ok(2),
                // General admin
                origins::Origin::CouncilAdmin => Ok(13),
                // Referendum admins
                origins::Origin::ReferendumCanceller => Ok(20),
                origins::Origin::ReferendumKiller => Ok(21),
                // Limited treasury spenders
                origins::Origin::SmallSpender => Ok(32),
                origins::Origin::BigSpender => Ok(34),
            }
        } else {
            Err(())
        }
    }
}
pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);
