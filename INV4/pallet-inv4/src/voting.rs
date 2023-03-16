use crate::{origin::INV4Origin, BalanceOf, Config, CoreStorage, Multisig, Pallet};
use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::{
    pallet_prelude::Member,
    traits::{fungibles::Inspect, PollStatus, VoteTally},
    CloneNoBound, EqNoBound, Parameter, PartialEqNoBound, RuntimeDebug, RuntimeDebugNoBound,
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{One, Zero},
    DispatchError, Perbill,
};
use sp_std::vec::Vec;

pub type Votes<T> = BalanceOf<T>;
pub type Core<T> = <T as Config>::CoreId;

/// Aggregated votes for an ongoing poll by members of the ranked collective.
#[derive(
    CloneNoBound,
    PartialEqNoBound,
    EqNoBound,
    RuntimeDebugNoBound,
    TypeInfo,
    Encode,
    Decode,
    MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Tally<T: Config> {
    pub ayes: Votes<T>,
    pub nays: Votes<T>,
    dummy: PhantomData<T>,
}

impl<T: Config> Tally<T> {
    pub fn from_parts(ayes: Votes<T>, nays: Votes<T>) -> Self {
        Tally {
            ayes,
            nays,
            dummy: PhantomData,
        }
    }
}

impl<T: Config> VoteTally<Votes<T>, Core<T>> for Tally<T> {
    fn new(_: Core<T>) -> Self {
        Self {
            ayes: Zero::zero(),
            nays: Zero::zero(),
            dummy: PhantomData,
        }
    }

    fn ayes(&self, _: Core<T>) -> Votes<T> {
        self.ayes
    }

    fn support(&self, class: Core<T>) -> Perbill {
        Perbill::from_rational(self.ayes, T::AssetsProvider::total_issuance(class))
    }

    fn approval(&self, _: Core<T>) -> Perbill {
        Perbill::from_rational(
            self.ayes,
            <Votes<T> as One>::one().max(self.ayes + self.nays),
        )
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn unanimity(_: Core<T>) -> Self {
        todo!()
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn rejection(_: Core<T>) -> Self {
        todo!()
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn from_requirements(_: Perbill, _: Perbill, _: Core<T>) -> Self {
        todo!()
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn setup(_: Core<T>, _: Perbill) {
        todo!()
    }
}

pub trait CustomPolling<Tally> {
    type Index: Ord + PartialOrd + Copy + MaxEncodedLen;
    type Votes: Parameter + Ord + PartialOrd + Copy + HasCompact + MaxEncodedLen;
    type Class: Parameter + Member + Ord + PartialOrd + MaxEncodedLen;
    type Moment;

    // Provides a vec of values that `T` may take.
    fn classes() -> Vec<Self::Class>;

    /// `Some` if the referendum `index` can be voted on, along with the tally and class of
    /// referendum.
    ///
    /// Don't use this if you might mutate - use `try_access_poll` instead.
    fn as_ongoing(class: Self::Class, index: Self::Index) -> Option<(Tally, Self::Class)>;

    fn access_poll<R>(
        class: Self::Class,
        index: Self::Index,
        f: impl FnOnce(PollStatus<&mut Tally, Self::Moment, Self::Class>) -> R,
    ) -> R;

    fn try_access_poll<R>(
        class: Self::Class,
        index: Self::Index,
        f: impl FnOnce(PollStatus<&mut Tally, Self::Moment, Self::Class>) -> Result<R, DispatchError>,
    ) -> Result<R, DispatchError>;
}

impl<T: Config> CustomPolling<Tally<T>> for Pallet<T> {
    type Index = T::Hash;
    type Votes = Votes<T>;
    type Moment = T::BlockNumber;
    type Class = T::CoreId;

    fn classes() -> Vec<Self::Class> {
        CoreStorage::<T>::iter_keys().collect()
    }

    fn access_poll<R>(
        class: Self::Class,
        index: Self::Index,
        f: impl FnOnce(PollStatus<&mut Tally<T>, T::BlockNumber, T::CoreId>) -> R,
    ) -> R {
        match Multisig::<T>::get(class, index) {
            Some(mut m) => {
                let result = f(PollStatus::Ongoing(&mut m.tally, class));
                Multisig::<T>::insert(class, index, m);
                result
            }
            _ => f(PollStatus::None),
        }
    }

    fn try_access_poll<R>(
        class: Self::Class,
        index: Self::Index,
        f: impl FnOnce(PollStatus<&mut Tally<T>, T::BlockNumber, T::CoreId>) -> Result<R, DispatchError>,
    ) -> Result<R, DispatchError> {
        match Multisig::<T>::get(class, index) {
            Some(mut m) => {
                let result = f(PollStatus::Ongoing(&mut m.tally, class))?;
                Multisig::<T>::insert(class, index, m);
                Ok(result)
            }
            _ => f(PollStatus::None),
        }
    }

    fn as_ongoing(class: Self::Class, index: Self::Index) -> Option<(Tally<T>, T::CoreId)> {
        Multisig::<T>::get(class, index).map(|m| (m.tally, class))
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Vote<Votes> {
    Aye(Votes),
    Nay(Votes),
}

pub type VoteRecord<T> = Vote<Votes<T>>;

impl<T: Config> Pallet<T>
where
    Result<
        INV4Origin<T, <T as crate::pallet::Config>::CoreId, <T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::RuntimeOrigin,
    >: From<<T as frame_system::Config>::RuntimeOrigin>,
{
    pub fn minimum_support_and_required_approval(core_id: T::CoreId) -> Option<(Perbill, Perbill)> {
        CoreStorage::<T>::get(core_id).map(|core| (core.minimum_support, core.required_approval))
    }
}
