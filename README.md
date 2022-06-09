<div align="center">
<img src="https://github.com/InvArch/brand/blob/main/InvArch-logo-dark/cover.png">
</div>

<div align="Center">
<h1>InvArch FRAME Pallet Library</h1>
<h2> A Git Compatible IP Hosting, Management, & Cross-Chain Authentication Network for Web3 </h2>

<br>
Official Repository for the InvArch FRAME Pallet Library 💡
Built on Substrate

<br>  
<br>

[![Substrate version](https://img.shields.io/badge/Substrate-v3.0.0-E6007A?logo=Parity%20Substrate)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)
[![Medium](https://img.shields.io/badge/Medium-InvArch-E6007A?logo=medium)](https://invarch.medium.com/)
[![License](https://img.shields.io/github/license/InvArch/InvArch?color=E6007A)](https://github.com/InvArch/InvArch/blob/main/LICENSE)
<br />
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2FInvArch)](https://twitter.com/InvArchNetwork)
[![Discord](https://img.shields.io/badge/Discord-gray?logo=discord)](https://discord.gg/invarch)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/InvArch)

</div>

<!-- TOC -->

## <!-- /TOC -->

## Intro

This repository should contains the Substrate FRAME Pallets used in the InvArch blockchain, and reviews their relationships and functions. At the current stage, the goal of creating this document and repository is centered around getting feedback while we continue to write the code and develop InvArch. This is a WIP.

## Overview

InvArch A Git Compatible IP Hosting, Management, & Cross-Chain Authentication Network for Web3

InvArch features the INV4 (Invention, Involvement, Inventory, Investment), OCIF (On-Chain Innovation Funding), & XCA (Cross-Chain Authentication) Protocols.

XCM features Cross-Consensus Messaging (XCM) to index, cross-reference, & certify IP asset authenticity across Web3 using various hashing methods & rounding algorithms.

<div align="center">
<img src="https://github.com/InvArch/brand/blob/main/architecture.png">
</div>

## Features

| Term                                 | Abbreviation(s)   | Description                                                                                                                            |
| ------------------------------------ | ----------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Intellectual Property Set            | IP Set, IPS       | On-Chain Repositories & Folders. Consist of interchangeable IP Files & feature various IP Tokens.                                      |
| Intellectual Property File           | IP File, IPF      | Omni-Composable & Cross-Chain Authenticated Assets. Powered With RMRK NFTs & Piracy-Proof Files.                                       |
| Intellectual Property Tokens         | IP Tokens, IPT    | Multi-Tiered Fungible Assets Pegged To IP Sets. Realize Re-Fungible Ownership, Join Copyright, & Various Multi-Utility Purposes.       |
| Intellectual Property Licenses       | IP Licenses, IPL  | On-Chain Copyright, Licensing, & Version Control Management. Customizable, Internationally Compliant, & Attached To Every Root IP Set. |
|                                      |                   |                                                                                                                                        |
| Intellectual Property Staking        | IP Staking        | On-Chain Staking For dApps, DAOs, Smart Contracts, & Other IP Set Based Assets.                                                        |
| Intellectual Property Farming        | IP Farming        | Built-In Liquidity Tools For dApps, DAOs, & IP Tokens.                                                                                 |
| Intellectual Property Donations      | IP Donations      | Full Or Partial Donations Of Staking Rewards For dApps, DAOs, Smart Contracts, & Other IP Set Based Assets.                            |
|                                      |                   |                                                                                                                                        |
| Intellectual Property Authentication | IP Authentication | Cross-Chain Indexing, Cross-Referencing, & Authenticating For INV4 Files & NFTs.                                                       |
| Intellectual Property Disputes       | IP Disputes       | On-Chain Governance Provides A Decentralized Process For Retroactive IP Ownership Disputes.                                            |

## Components

### INV4 Protocol & Pallets

- `Pallet_IPS` - [IP Sets (IPS) Pallet](https://github.com/InvArch/InvArch-Frames/tree/main/INV4/pallet-ips)
- `Pallet_IPF` - [IP Files (IPF) Pallet](https://github.com/InvArch/InvArch-Frames/tree/main/INV4/pallet-ipf)
- `Pallet_IPT` - [IP Tokens (IPT) Pallet](https://github.com/InvArch/InvArch-Frames/tree/main/INV4/pallet-ipt)
- `Pallet_IPL` - [IP Licenses (IPL) Pallet](https://github.com/InvArch/InvArch-Frames/tree/main/INV4/pallet-ipl)

### OCIF Protocol & Pallets

- `Pallet_IPStaking` - IP Staking (W.I.P.)
- `Pallet_IPFarming` - IP Farming Pallet (W.I.P.)
- `Pallet_IPDonations` - IP Donations Pallet (W.I.P.)

### XCA Protocol & Pallets

- `Pallet_XCA` - Cross-chain IP Authentication Pallet (W.I.P.)
- `Pallet_DisputeXCA` - IP Disputes Pallet (W.I.P.)

## Testing Documentation

- [INV4 Testing Documentation](https://gist.github.com/arrudagates/877d6d7b56d06ea1a941b73573a28d3f)
- [OCIF Testing Documentation](https://github.com/InvArch/InvArch-Frames)
- [XCA Protocol Testing Documentation](https://github.com/InvArch/InvArch-Frames)

## How to contribute

I'm really glad you're reading this, because we need volunteer developers to help this idea become a reality!

If you haven't already, come find us on the [#InvArch Discord](https://discord.gg/invarch). We want you working on things you're excited about!

### Submitting changes

Please send a [GitHub Pull Request to InvArch](https://github.com/InvArch/InvArch/pull/new/master) with a clear list of what you've done (read more about [pull requests](http://help.github.com/pull-requests/)). Please make sure all of your commits are atomic (one feature per commit).

Always write a clear log message for your commits. One-line messages are fine for small changes, but bigger changes should look like this:

    $ git commit -m "A brief summary of the commit
    >
    > A paragraph describing what changed and its impact."

Please make sure to update tests as appropriate.

Thank you,<br>
Dakota Barnett, Founder

### License

[GPLv3.0](https://github.com/InvArch/InvArch/blob/main/LICENSE)

### Substrate Node

Substrate Node Template [README.md](https://github.com/substrate-developer-hub/substrate-node-template)
