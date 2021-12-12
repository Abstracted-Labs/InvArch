[![Compatible with Substrate v3.0.0](https://img.shields.io/badge/Substrate-v3.0.0-E6007A)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)

# IPF Pallet: IP Tokens for Substrate

This is a [Pallet](https://substrate.dev/docs/en/knowledgebase/runtime/pallets) that defines basic functions
to create and manage [intellectual property (IP)](https://en.wikipedia.org/wiki/Intellectual_property) stored as [non-fungible tokens (NFTs)](https://en.wikipedia.org/wiki/Non-fungible_token). 

# IPFokens : Non-fungible components that define an idea.

The following **components** are defined:
* `IPFoken` + Metadata

The following **functions** are possible:
* `mint` - Create a new IP Token and add to an IP Set
* `burn` - Burn an IP Token from an IP Set
* `amend` - Amend the data stored inside an IP Token


# IP Token

An IP Token (IPF) is a part of a set, and can be thought of as a component of an idea. Either by itself or in combination with other IP Tokens, it serves to strengethen the foundation for an innovation. IP Tokens represent a unique digital asset.

## IPF Standard

```json
{
  "ips": {
    "type": "string",
    "description": "Collection ID, e.g. 0aff6865bed3a66b-HOVER"
  },
  "name": {
    "type": "string",
    "description": "Name of the IPF. E.g. Hover Craft Schematics, Hover Craft PoC."
  },
  "sn": {
    "type": "string",
    "description": "Serial number or issuance number of the IPF, padded so that its total length is 16, e.g. 0000000000000123"
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

### Computed fields

Computed fields are fields that are used in interactions, but are not explicitly set on their
entities. Computed fields are the result of applying a standardized calculation or merger formula to
specific fields. The IPF entity has the following computed fields, to be provided by
implementations:

```json
{
  "id": {
    "type": "computed",
    "description": "An IPF is uniquely identified by the combination of its minting block number, set ID, its instance ID, and its serial number, e.g. 4110010-0aff6865bed5g76b-HOVER-0000000000000123"
  }
}
```

Example id: `4110010-0aff6865bed5g76b-HOVER-0000000000000123`.

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

#### Example

#### A binary video

```json
data: {
  "protocol": "bin",
  "data": "AAAAIGZ0eXBpc29tAAACAGlzb21pc28yYXZjMW1wNDEAAAAIZnJlZQAQC0ttZGF0AQIUGRQmM...",
  "type": "video/mp4"
}
```

## Metadata Standard

```json
{
  "external_url": {
    "type": "string",
    "description": "HTTP or IPFS URL for finding out more about this token. If IPFS, MUST be in the format of ipfs://ipfs/HASH"
  },
  "image": {
    "type": "string",
    "description": "HTTP or IPFS URL to project's main image, in the vein of og:image. If IPFS, MUST be in the format of ipfs://ipfs/HASH"
  },
  "image_data": {
    "type": "string?",
    "description": "[OPTIONAL] Use only if you don't have the image field (they are mutually exclusive and image takes precedence). Raw base64 or SVG data for the image. If SVG, MUST start with <svg, if base64, MUST start with base64:"
  },
  "description": {
    "type": "string",
    "description": "Description of the IP Token. Markdown is supported."
  },
  "name": {
    "type": "string",
    "description": "Name of the IP Token."
  },
  "attributes": {
    "type": "array",
    "description": "An Array of JSON objects, matching OpenSea's: https://docs.opensea.io/docs/metadata-standards#section-attributes"
  },
  "animation_url": {
    "type": "string",
    "description": "[OPTIONAL] HTTP or IPFS URL (format MUST be ipfs://ipfs/HASH) for an animated image of the item. GLTF, GLB, WEBM, MP4, M4V, and OGG are supported, and when using IPFS type MUST be appended, separated by colon, e.g. ipfs://ipfs/SOMEHASH:webm."
  },
}
```

## Example

IPF:

```json
{
  "ips": "0aff6865bed5g76b-HOVER",
  "name": "PoC STL file",
  "sn": "0000000000000001",
  "metadata": "ipfs://ipfs/QmYcWFQCY1bAZ7ffRggt36fhdbvfeb7Tu5hzzeCWQ"
}
```

Metadata:

```json
{
  "external_url": "https://invarch.app/registry/0aff6865bed5g76b-HOVER",
  "image": "ipfs://ipfs/QmVgs8P4awhZpFXhkkgnCwBp4AdKRj3F9K56dbYwu3q",
  "description": "A Proof-of-Concept for a hover craft shown in a 3d rendering",
  "name": "Hover Craft PoC STL file",
  "attributes": [],
}
```
