pub mod currency {
    use crate::Balance;

    pub const UNIT: Balance = 1_000_000_000_000;
    pub const MILLIUNIT: Balance = 1_000_000_000;
    pub const MICROUNIT: Balance = 1_000_000;

    pub const CENTS: Balance = UNIT / 10_000;
    pub const MILLICENTS: Balance = CENTS / 1_000;

    // Almost same as Kusama
    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 2_000 * CENTS + (bytes as Balance) * 100 * MILLICENTS
    }
}

/// The IpId
pub type CommonId = u32;
