[![Compatible with Substrate v3.0.0](https://img.shields.io/badge/Substrate-v3.0.0-E6007A)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)

# DEV Pallet: Decentralized Entrepreneurial Ventures for Substrate

This is a Substrate [Pallet](https://substrate.dev/docs/en/knowledgebase/runtime/pallets) that defines basic functions
to manage the creation and formation of DEV partnerships, which are stored as [non-fungible tokens (NFTs)](https://en.wikipedia.org/wiki/Non-fungible_token). 

# DEVs : Establishing Terms & Recording Interactions.

The following **components** are defined:
* `DEV` + Metadata

The following **functions** are possible:
* `create` - Create a new DEV agreement
* `post` - Post a DEV as joinable
* `add` - Add a user to a DEV
* `remove` - Remove a user from a DEV
* `update` - Update a DEV to include a new interaction
* `freeze` - Freeze a DEV and its metadata

# DEV

This standard defines how decentralized entrepreneurial ventures are structured and established.

A DEV is an agreement between 2 or more parties to work together in order to actualize an IP Set. 

The `Pallet_dev` is responsible for linking a venture to an IP Set, establishing the roles, terms, milestones, 
IPO allocations, tracking interactions with an IP Set (including the interview process(es), and then freezing 
this information and storing it as an NFT using IPFS.

DEVs would be considered multi-attribute NFTs.

<div align=center>
  <img src="https://i.ibb.co/6myQDwD/Screen-Shot-2021-10-07-at-4-07-39-PM.png">
</div>
