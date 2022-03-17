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

This repository should contains the Pallets used in the InvArch blockchain, and reviews their relationships and functions. At the current stage, the goal of creating this document and repository is centered around getting feedback while we continue to write the code and develop InvArch. This is a WIP.

## Overview

InvArch is the world's first truly composable IP ownership, utility, & cross-chain authentication (XCA) protocol.

InvArch features the INV4 (Invention, Involvement, Inventory, Investment) Standard for minting authenticated & interoperable files or NFTs as IP Files (IPFs), truly composable IP Sets, IP Replicas (IPRs), Bridged IP (BIPs), Wrapped IP (WIPs), & pegged IP Tokens (IPTs) featuring multi-purpose & multi-level utility to Web 3.0.

InvArch also introduces the Cross-Chain Authentication (XCA) Protocol, featuring Cross-Consensus Messaging (XCM) to index, cross-reference, & certify IP asset authenticity across Web3 using various hashing methods & rounding algorithms.

<div align="center">
<img src="https://github.com/InvArch/brand/blob/main/architecture.png">
</div>

## Features

| Term                                  | Abbreviation(s)   | Description                                                                                                                  |
| ------------------------------------- | ----------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Intellectual Property File            | IP File, IPF      | Intellectual Property (IP) stored as non-fungible & authenticated files                                                      |
| Intellectual Property Set             | IP Set            | Root collections of bonded & interchangeable IP Files and/or Subsets                                                         |
| Intellectual Property Subset          | IP Subset, Subset | Child collections of bonded & interchangeable IP Files and/or additional IP Subsets                                          |
| Intellectual Property Replica         | IP Replica, IPR   | Authorized clones, or forks, of IP Sets, Subsets, and/or Files                                                               |
| Bridged Intellectual Property         | Bridged IP, BIP   | EVM or other outer-consensus-native NFTs bridged to the INV4 standard as IP Files                                            |
| Bonded Intellectual Property          | Bonded IP         | Two (2) or more bonded IP Files, Subsets, and/or Sets representating a new single IP Set and/or Subset                       |
| Intellectual Property Tokens          | IP Tokens, IPTs   | Fungible & programmable tokens that are pegged to an IP Set and/or Subset                                                    |
| Intellectual Property Sub-Tokens      | Sub-IPTs          | Multi-leveled or tiered IP Tokens representing distinctive functionality from each other                                     |
| Smart Intellectual Property           | SmartIP           | IP Sets that own themselves or are decentrally owned, and trustlessly execute functions within their IP Files and/or Subsets |
| Intellectual Property Virtual Machine | IPVM              | A distributed state machine & trustless environment for executing SmartIP contracts and maintaining canonical state          |

## Components

### INV4 Protocol & Pallets

- `Pallet_IPS` - W.I.P.
- `Pallet_IPF` - W.I.P.
- `Pallet_IPR` - W.I.P.
- `Pallet_BridgeIP` - W.I.P.
- `Pallet_BondIP` - W.I.P.
- `Pallet_IPSynth` - W.I.P.
- `Pallet_IPT` - W.I.P.
- `Pallet_MultiSig` - W.I.P.
- `Pallet_IPVM` - W.I.P.

### XCA Protocol & Pallets

- `Pallet_XCA` - W.I.P.
- `Pallet_DisputeXCA` - W.I.P.

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

[GPL](https://github.com/InvArch/InvArch/blob/main/LICENSE)

### Substrate Node

Substrate Node Template [README.md](https://github.com/substrate-developer-hub/substrate-node-template/blob/tutorials/solutions/build-a-dapp-v3%2B1/README.md)
