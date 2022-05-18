[![Compatible with Substrate v3.0.0](https://img.shields.io/badge/Substrate-v3.0.0-E6007A)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)

# IPL Pallet: IP Licensing for Substrate

This is a Substrate [Pallet](https://substrate.dev/docs/en/knowledgebase/runtime/pallets) that defines basic functions
to select or create, and manage, licensing terms & agreements that are pegged Intellectual Property Sets (IPS). 

# IP License

The following **components** are defined:

* `IP License`

The following **callable functions** are possible:

* `set_permission` - Create a new IP File and add to an IP Set
* `set_asset_weight` - Burn an IP File from an IP Set

# IP Licenses

An IP Licenses (IPL) is an on-chain copyright, licensing, & version control management. This is designed to be customizable, internationally compliant, & attached to every root IP Set.

## IPL Standard

```json
{
  "name": {
    "type": "string",
    "description": "Name of the IPL. E.g. MIT, GPLv3.0"
  },
  "iplId": {
    "type": "u64",
    "description": "The ID of an existing IPL inside of an IP Set"
  },
  "metadata?": {
    "type": "Vec<u8>",
    "description": "Free to use any value as this won't affect logic and is intended to be used by dApp developers"
  },
  "data?": {
    "type": "H256",
    "description": "An IPFS CID hash, intended to be the content identifier of the actual file, can be taken from CID here: https://cid.ipfs.io by copying the Digest (Hex) field"
  },
  "permission": {
    "type": "bool",
    "description": "set permission to true or false"
  },
  "asset-weight": {
    "type": "Percent",
    "description": "One or one-per-cent"
  }
}
```

## Testing Documentation

[IPL Testing Documentation](https://gist.github.com/arrudagates/877d6d7b56d06ea1a941b73573a28d3f)