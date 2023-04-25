<div align="center">
<img src="https://github.com/InvArch/brand/blob/main/InvArch-logo-dark/cover.png">
</div>

<div align="Center">
<h1>InvArch FRAME Pallet Library</h1>
<h2> Multichain MPC & DAO Infrastructure </h2>

<br>
Official Repository for the InvArch FRAME Pallet Library
💡Built on Substrate

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

InvArch is a blockchain network & cross-consensus operating system for DAOs. InvArch revolves around on multi-party ownership & computation with a focus on non-custodial asset management, intellectual property rights facilitation, & DAO operations.

Currently, InvArch features a multichain multisignature solution & DAO staking protocol.

## Features

- `Multichain Multisig` - Please see the `Saturn SDK` below.
- `DAO Staking` - https://www.tinker.network/staking

### Resources

- `Saturn SDK` - https://github.com/InvArch/saturn-sdk

### Custom Protocols & Pallets

- `INV4` - Account structure & ownership ontology protocol
- `Rules` - layer for defining custom account permissions
- `Rings` - XCM abstraction layer
- `OCIF` - DAO Staking & Farming protocol

## Testing Documentation

- [INV4 Testing Documentation](https://gist.github.com/arrudagates/877d6d7b56d06ea1a941b73573a28d3f)
- [OCIF Testing Documentation](https://github.com/InvArch/InvArch-Frames)

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
