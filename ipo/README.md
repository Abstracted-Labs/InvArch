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

### Key Info Regarding `IPOwnership`
To ensure that no single actor can have a 51% hold over a project, IPO can be distributed within the following ranges:
<div align=center>
  <img src="https://i.ibb.co/7NKWDM6/Screen-Shot-2021-08-28-at-5-41-35-PM.png">
</div>
<div align=center>
  𝑓 + 𝑡 = 10000 | 0 ≤ 𝑓 ≤ 6600 | 3400 ≤ 𝑡 ≤ 10000
</div>
Among the Founders, out of however much IPO is decided to be allocated, no single
participant can have more than 50% (Max. 3300) of the allocated IPO. No single
co-founder can have a higher stake than the founder. The distribution algorithm for the
founder’s distribution is:<br>
<div align=center>
  𝑓(𝑂) / 𝑝(𝑛) ≥ 𝑝(𝑂)<br>
</div>
Where 𝑓(𝑂)represents the founder’s total IPOwnership tokens, 𝑝(𝑛)represents the number of
co-founders, and 𝑝(𝑂)represents a co-founder’s IPOwnership tokens. This statement must
pass to form a DEV, and changes that break this statement cannot be implemented.
* Voting Weight
IPO acts as a governance token over a DEV. Holders have the right to propose
development changes, financing strategies, report misconduct, and vote on status consensus reports. Every DEV has 10,000 votes, with an IPO representing a single vote.
The more IPO a participant has, the more voting weight they have.
