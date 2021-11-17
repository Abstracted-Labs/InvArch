pub mod currency {
    use node_primitives::Balance;

    /// The existential deposit. Set to 1/10 of its parent Relay Chain (v9010).
    pub const EXISTENTIAL_DEPOSIT: Balance = 10 * CENTS;

    pub const UNITS: Balance = 10_000_000_000;
    pub const DOLLARS: Balance = UNITS;
    pub const CENTS: Balance = UNITS / 100; // 100_000_000
    pub const MILLICENTS: Balance = CENTS / 1_000; // 100_000

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        // 1/10 of Polkadot v9010
        (items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS) / 10
    }
}
