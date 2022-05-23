//! Benchmarks for IPF Pallet
#![cfg(feature = "runtime-benchmarks")]

pub use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_system::{Config, RawOrigin};
use sp_core::H256;

const MOCK_DATA: [u8; 32] = [
    12, 47, 182, 72, 140, 51, 139, 219, 171, 74, 247, 18, 123, 28, 200, 236, 221, 85, 25, 12, 218,
    0, 230, 247, 32, 73, 152, 66, 243, 27, 92, 95,
];

runtime_benchmarks! {
    where_clause {
        where T: pallet::Config + frame_system::Config<Hash = H256>
    }
    // Mint IPF
    mint {
        let caller: T::AccountId = whitelisted_caller();
        let metadata: Vec<u8> = vec![1];
        let data = H256::from(MOCK_DATA);
    }: _(RawOrigin::Signed(caller), metadata, data)

    // Burn IPF
    burn {
        let caller: T::AccountId = whitelisted_caller();
        let metadata: Vec<u8> = vec![1];
        let data = H256::from(MOCK_DATA);
        let ipf_id: u32 = 0;

        Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), metadata, data)?;
    }: _(RawOrigin::Signed(caller), T::IpfId::from(ipf_id))
}

impl_benchmark_test_suite!(Ipf, crate::mock::new_test_ext(), crate::mock::Test);
