[![Compatible with Substrate v3.0.0](https://img.shields.io/badge/Substrate-v3.0.0-E6007A)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)

# IPO FRAME Pallet: IP Ownership for Substrate

This is a [Pallet](https://substrate.dev/docs/en/knowledgebase/runtime/pallets) that defines basic functions to create and manage ownership of [intellectual property (IP)](https://en.wikipedia.org/wiki/Intellectual_property) stored as [fungible] and [fractionalized] ownership that are built-in to every [IPSet](../pallet_ips/pallet_ips.md). 

# IP Ownership : Fungible and fractionalized ownership of IP Sets

The following **components** are defined:
* `IPOwnership` + Metadata

The following **functions** are possible following the [balances pallet](https://github.com/paritytech/substrate/tree/master/frame/balances) and [asset pallet](https://github.com/paritytech/substrate/tree/master/frame/assets):
* `issue` - Issues the total supply of a new fungible asset to the account of the caller of the function
* `transfer` - Transfer some liquid free balance to another account
* `set_balance` - Set the balances to a given account. The origin of this call mus be root
* `get_balance` - Get the asset `id` balance of `who`
* `total_supply` - Get the total supply of an asset `id`
* `bind` - Bind some `amount` of unit of fungible asset `id` from the ballance of the function caller's account (`origin`) to a specific `IPSet` account to claim some portion of fractionalized ownership of that particular `IPset`
* `unbind` - Unbind some `amount` of unit of fungible asset `id` from a specific `IPSet` account to unclaim some portion of fractionalized ownership to the ballance of the function caller's account'
