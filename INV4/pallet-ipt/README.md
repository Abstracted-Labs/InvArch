[![Compatible with Substrate v3.0.0](https://img.shields.io/badge/Substrate-v3.0.0-E6007A)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)

# IPT Pallet: IP Tokens for Substrate

This is a Substrate [Pallet](https://substrate.dev/docs/en/knowledgebase/runtime/pallets) that defines basic functions
to create and manage Intellect Property Tokens (IPT), which are fungible assets pegged to a non-fungible asset called 
an Intellectual Property Set (IPS). IPT can feature SubAssets, which are multiple layers of fungible assets pegged to a
single IPS.

# IP Tokens : Fungible Assets & SubAssets, Pegged to an IPS

The following **components** are defined:
* `IPToken` + Metadata

The following **callable functions** are possible:

* `mint` - Create a new IP File and add to an IP Set
* `burn` - Burn an IP File from an IP Set
* `operate_multisig` - Set an IPT as a multisig 
* `vote_multisig` - Give vote for a multisig IPT
* `withdraw_vote_multisig` - Remove a vote from a voted multisig IPT
* `create_sub_asset` - Create Sub-Asset of an IPT


# IP Token

An IP Token (IPT) is a programmable fungible tokens that can be pegged to an IP Set. Similar to ERC20 fungible tokens, IP Tokens have a property that makes each token exactly the same (in type and value). As a result, IP Sets can deploy IPTs in a very similar manner to how dApps utilize their utility tokens. IP Tokens realize an unrestricted possibility of use-cases such as assigning (exclusive or fractional) ownership rights, seamless royalty allocations, providing access rights & authorization tiers over data, deciding voting weight in a DAO or community governing IP, extending exclusive functionality, providing native currencies for IP-based dApps, & streamlining copyright licensing agreements.

## IPT Standard

```json
{
  "name": {
    "type": "string",
    "description": "Name of the IPT. E.g. IPT0, IPT1."
  },
  "iptId": {
    "type": "u64",
    "description": "The ID of an existing IPT owned by the current caller, can be found in events after minting or in storage"
  },
  "metadata?": {
    "type": "Vec<u8>",
    "description": "Free to use any value as this won't affect logic and is intended to be used by dApp developers"
  },
  "data?": {
    "type": "H256",
    "description": "An IPFS CID hash, intended to be the content identifier of the actual file, can be taken from CID here: https://cid.ipfs.io by copying the Digest (Hex) field"
  }
}
```

## Testing Documentation

[IPT Testing Documentation](https://gist.github.com/arrudagates/877d6d7b56d06ea1a941b73573a28d3f)