[![Compatible with Substrate v3.0.0](https://img.shields.io/badge/Substrate-v3.0.0-E6007A)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)

# IPS Pallet: IP Sets for Substrate

This is a Substrate [Pallet](https://substrate.dev/docs/en/knowledgebase/runtime/pallets) that defines basic functions
to create and manage sets of [intellectual property (IP)](https://en.wikipedia.org/wiki/Intellectual_property) stored as [non-fungible tokens (NFTs)](https://en.wikipedia.org/wiki/Non-fungible_token). 

# IPSets : Storing sets of components that define an idea.

The following **components** are defined:
* `IPSet` + Metadata

The following **functions** are possible:
* `create` - Create a new IP Set
* `send` - Transfer IP Set owner account address
* `list` - List an IP Set for sale
* `buy` - Buy an IP Set
* `destroy` - Delete an IP Set and all of its contents

# IP Set

This standard defines how **Sets** of related IP Tokens are minted.

In context an IP Set is viewed as an idea, which consists of one or more components (IP Tokens) that help to strenghthen and describe that idea. 

For example, a 3D rendering of a flux capacitor prototype could be stored as an IP Token representing an STL file.
Additionally, an XML file explaining the relation between flux capacitors different components could also be stored as an IP Token.
in the "Flux Capacitor" IP Set, these two files exists and help to strengethen and expand on the idea for building an flux capacitor.
Every IP Token must have a parent IP Set it belongs to.

## IP Set Standard

An IP Set MUST adhere to the following standard.

```json
{
  "name": {
    "type": "string",
    "description": "Name of the IP Set. Name must be limited to alphanumeric characters. Underscore is allowed as word separator. E.g. HOVER-CRAFT is NOT allowed. HOVER_CRAFT is allowed."
  },
  "account": {
    "type": "string",
    "description": "Account's address, e.g. CpjsLDC1JFydhtd64jn2ThY7qnd28UhwTwqjqGTRegbeaq1. Can be address different from minter."
  },
  "symbol": {
    "type": "string",
    "description": "Ticker symbol by which to represent the IPO token in wallets and UIs, e.g. HOVER"
  },
  "id": {
    "type": "string",
    "description": "An IP Set is uniquely identified by at least the first four and last four bytes of the original issuer's pubkey, combined with the symbol. This prevents anyone but the issuer from reusing the symbol, Example ID: 0aff6865bed5g76b-HOVER."
  },
  "metadata?": {
    "type": "string",
    "description": "HTTP(s) or IPFS URI. If IPFS, MUST be in the format of ipfs://ipfs/HASH"
  },
  "data?": {
    "type": "object",
    "description": "See Data"
  }
}
```

When either metadata or [data](#data) is present, the other is optional. Data takes precedence
always. Note that because metadata contains description, attributes, third party URLs, etc. it is
still recommended to include it alongside `data`.

### Data

The `data` object is composed of:

- protocol (strict, see Protocols below)
- data
- type (mime type)

#### Protocols

| Protocol  | Mime default           | Description                                                                                                                                    |
| --------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `ipfs`    | image/png              | Points to a directly interpretable resource, be it audio, video, code, or something else                                                       |
| `http(s)` | image/html             | Points to a directly interpretable resource, be it audio, video, code, or something else (not recommended for use)                             |
| `p5`      | application/javascript | Processing.js code                                                                                                                             |
| `js`      | application/javascript | Plain JS code                                                                                                                                  |
| `html`    | text/html              | HTML code, no need for `<html>` and `<body>`, can support dependencies but it's up to the author to prevent the dependencies from disappearing |
| `svg`     | image/svg+xml          | SVG image data                                                                                                                                 |
| `bin`     | n/a                    | binary, directly interpretable                                                                                                                 |

## Metadata

A collection SHOULD have metadata to describe it and help visualization on various platforms.

```json
{
  "description": {
    "type": "string",
    "description": "Description of the IP Set/idea as a whole. Markdown is supported."
  },
  "industry": {
    "type": "string",
    "description": "A string citing the idea's related industry. Markdown is supported."
  },
  "attributes": {
    "type": "array",
    "description": "An Array of JSON objects, matching OpenSea's: https://docs.opensea.io/docs/metadata-standards#section-attributes"
  },
  "external_url": {
    "type": "string",
    "description": "HTTP or IPFS URL for finding out more about this idea. If IPFS, MUST be in the format of ipfs://ipfs/HASH"
  },
  "image": {
    "type": "string",
    "description": "HTTP or IPFS URL to idea's main image, in the vein of og:image. If IPFS, MUST be in the format of ipfs://ipfs/HASH"
  },
  "image_data": {
    "type": "string?",
    "description": "[OPTIONAL] Use only if you don't have the image field (they are mutually exclusive and image takes precedence). Raw base64 or SVG data for the image. If SVG, MUST start with <svg, if base64, MUST start with base64:"
  }
}
```

## Examples

Collection:

```json
{
  "name": "Hover Craft",
  "account": "CpjsLDC1JFydhtd64jn2ThY7qnd28UhwTwqjqGTRegbeaq1",
  "symbol": "HOVER",
  "id": "0aff6865bed5g76b-HOVER",
  "metadata": "ipfs://ipfs/QmVgs8P4awhZpFXhkkgnCwBp4AdKRj3F9K56dbYwu3q"
}
```

Metadata:

```json
{
  "description": "An idea for a fully functioning street-legal hover craft!",
  "industry": "Mechanical Engineering",
  "attributes": [],
  "external_url": "https://invarch.app/registry/0aff6865bed5g76b-HOVER",
  "image": "ipfs://ipfs/QmYcWFQCY1bAZ7ffRggt367McqwrgfeqrZjXtribj5hzzeCWQ"
}
```
