<div align="center">
<img src="https://github.com/InvArch/InvArch-Frames/blob/56560bb81d4678d6e2e6a00cf3b79ab79cf42cbd/logo_colored.svg?raw=true" width="175" height="175" />
</div>

<div align="Center">
<h1>InvArch FRAME Pallet Library</h1>
<h2> Multichain MPC & DAO Infrastructure </h2>


Official Repository for the InvArch FRAME Pallet Library
💡Built on Substrate



[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2FInvArch)](https://twitter.com/InvArchNetwork)
[![Discord](https://img.shields.io/badge/Discord-gray?logo=discord)](https://discord.gg/invarch)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/InvArch)
[![Knowledge Hub](https://img.shields.io/badge/🧠_Knwoledge_hub-gray)](https://abstracted.notion.site/Knowledge-Hub-eec0071f36364d6aa8138f0004ac8d85)
<br />
[![Substrate version](https://img.shields.io/badge/Substrate-v3.0.0-E6007A?logo=Parity%20Substrate)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)
[![Medium](https://img.shields.io/badge/Medium-InvArch-E6007A?logo=medium)](https://invarch.medium.com/)
[![License](https://img.shields.io/github/license/InvArch/InvArch?color=E6007A)](https://github.com/InvArch/InvArch/blob/main/LICENSE)
[![Library Docs](https://img.shields.io/badge/Library-Docs%2Ers-E6007A?logo=docsdotrs)](https://invarch.github.io/InvArch-Frames/)

</div>  

---

## Intro

This repository should contain the Substrate FRAME Pallets used in the InvArch blockchain, and reviews their relationships and functions. At the current stage, the goal of creating this document and repository is centered around getting feedback while we continue to write the code and develop InvArch. This is a WIP.

Check out the [Knowledge Hub](https://abstracted.notion.site/Knowledge-Hub-eec0071f36364d6aa8138f0004ac8d85), it is the perfect place to dive into all things InvArch

## Overview

InvArch is a blockchain network & cross-consensus operating system for DAOs. InvArch revolves around on multi-party ownership & computation with a focus on non-custodial asset management, intellectual property rights facilitation, & DAO operations.

Currently, InvArch features a multichain multisignature solution & DAO staking protocol.

---

# Pallet Library

 ## [INV4](./INV4/pallet-inv4/)
 - The INV4 pallet is designed to manage advanced virtual multisigs, internally referred to as cores.
    - [`Docs.rs`](https://invarch.github.io/InvArch-Frames/pallet_inv4/index.html)
 - Articles:
    - [`The Saturn SDK.`](https://invarch.medium.com/the-saturn-sdk-c46b4e40f46e)
    - [`The INV4 Protocol: The Core of the Creator Economy.`](https://invarch.medium.com/the-inv4-protocol-the-core-of-the-creator-economy-1af59fdbc943)
    - [`🪐 Saturn: The Future of Multi-Party Ownership.`](https://invarch.medium.com/saturn-the-future-of-multi-party-ownership-ac7190f86a7b)
  
 ## [OCIF](./OCIF/staking/)
 - The OCIF Staking Pallet is a pallet designed to facilitate staking towards INV-Cores within a blockchain network.
    - [`Docs.rs`](https://invarch.github.io/InvArch-Frames/pallet_ocif_staking/index.html)
 - Articles:
    - [`The OCIF Protocol: Permissionless Funding for DAOs & Creators.`](https://invarch.medium.com/the-ocif-protocol-permissionless-funding-for-daos-creators-505aa18098f1)
 - DAO Staking is live on [InvArch](https://portal.invarch.network/staking) and [Tinkernet](https://www.tinker.network/staking).

 ## [Rings](./pallet-rings)
 - The Rings pallet provides a cross-consensus message (XCM) abstraction layer for INV4 Cores.
    - [`Docs.rs`](https://invarch.github.io/InvArch-Frames/pallet_rings/index.html)

 ## [Checked Inflation](./pallet-checked-inflation)
 - The Checked Inflation pallet is designed to facilitate the inflationary aspect of a blockchain's economy.
    - [`Docs.rs`](https://invarch.github.io/InvArch-Frames/pallet_checked_inflation/index.html)

---

## How to contribute

We need volunteer developers to help this idea become a reality!

If you haven't already, come find us on the [#InvArch Discord](https://discord.gg/invarch). We want you working on things you're excited about!

### Submitting changes

Please send a [GitHub Pull Request to InvArch](https://github.com/InvArch/InvArch/pull/new/master) with a clear list of what you've done (read more about [pull requests](http://help.github.com/pull-requests/)). Please make sure all of your commits are atomic (one feature per commit).

Always write a clear log message for your commits. One-line messages are fine for small changes, but bigger changes should look like this:

    $ git commit -m "A brief summary of the commit
    >
    > A paragraph describing what changed and its impact."

Please make sure to update tests as appropriate.


### License

[GPLv3.0](https://github.com/InvArch/InvArch/blob/main/LICENSE)