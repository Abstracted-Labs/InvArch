<div align="center">
<img src="https://github.com/InvArch/brand/blob/main/InvArch-logo-dark/cover.png">
</div>

<div align="Center">
<h1>InvArch</h1>
<h2> The Future of Innovation </h2>
The world’s first intellectual property tokenization & networking platform.
<br>
Official Repository for the InvArch platform 💡
Built on Substrate 

<br>  
<br>

[![Substrate version](https://img.shields.io/badge/Substrate-v3.0.0-E6007A?logo=Parity%20Substrate)](https://github.com/paritytech/substrate/releases/tag/v3.0.0)
[![Medium](https://img.shields.io/badge/Medium-InvArch-E6007A?logo=medium)](https://invarch.medium.com/)
[![License](https://img.shields.io/github/license/InvArch/InvArch?color=E6007A)](https://github.com/InvArch/InvArch/blob/main/LICENSE)
 <br />
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2FInvArch)](https://twitter.com/InvArchNetwork)
[![Discord](https://img.shields.io/badge/Discord-gray?logo=discord)](https://discord.gg/J3hapvrpZJ)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)]()
</div>

<!-- TOC -->

- [Introduction](##-Overview)
  - [Philosophy](###-InvArch-approaches-ideas-(IP)-as-a-set-of-non-fungible-components)
- [Features](##-Features)
  - [Intellectual Property Set (IPS)](###-Intellectual-Property-Set-IPS)
  - [Intellectual Property Token (IPT)](###-Intellectual-Property-Token-IPT)
  - [Decentralized Entrepreneurial Ventures (DEVs)](###-Decentralized-Entrepreneurial-Ventures-DEVs)
  - [IP Ownership (IPO)](###-IP-Ownership-IPO)
- [Components](##-Components)
  - [IP Protocol & Pallets](###-IP-Protocol-&-Pallets)
  - [DEV Protocol & Pallets](###-DEV-Protocol-&-Pallets)
- [How to Contribute](##-How-to-contribute)
  - [Submitting changes](###-Submitting-changes)
  - [License](###-License)
  - [Substrate Node](###-Substrate-Node)

<!-- /TOC -->
---
## Overview

InvArch is short for the "Invention, Involvement, & Investment Arch" platform.

The focus of the InvArch project is to develop a platform where individuals can mint & store their ideas and concepts for innovations as NFTs called
intellectual property tokens (IPTs). 

Users can form partnerships, call decentralized entrepreneurial ventures (DEVs), between the author of an IPT and 
users whoo have the skills and/or resources to actualize the idea. 

DEVs are formed by leveraging fractional & fungible ownership tokens that are pegged to an IPT and built into the DEV. IPTO can also be used 
in exchange for startup capital, and provides governance participation in a DEV. 

When a DEV is complete, which is determined through the consensus of a DEV's governing community, its related IPTO is liquidated at a proportionate
ratio to either cryptocurrency tokens or company shares. 🚀

<div align="center">
<img src="https://i.ibb.co/hFM47Qh/Screen-Shot-2021-09-11-at-4-39-30-PM.png" style="align-center">
</div>

### InvArch approaches ideas (IP) as a set of non-fungible components 
* IP Set = Idea
* IP Tokens  = components of their idea. 
* An IP Set can have built-in IP Ownership tokens. 
* You can,`list`,`sell`,`buy`,`transfer`, and `destroy` an IP Set, but not individual IP Tokens, since they're only components. 
* A new IP set can be created (`create`) and new IPT can be minted (`mint`) and added to a Set.
* Existing IPT can be burned (`burn`) or amended (`amend`). 
* Subsequently, an entire IP Set could be destroyed (`destroy`) as well, burning all of its contents.

## Features

### Intellectual Property Set (IPS)

⚙️   Think of an IPS as an idea/innovation that consists of one or more components that help define that idea/innovation.

⚙️   Multi-layer IPS: an IPS can own and inherit the metadata of another IPS. (Future Release)

### Intellectual Property Token (IPT)

⚙️   Think of an IPT as a component of an idea/innovation.

⚙️   Multi-attribute IPTs: code, diagrams, 3D models, images, and other docs can be stored within the metadate of an IPT.

### Decentralized Entrepreneurial Ventures (DEVs)

⚙️   IPS Governed as DAOs: Pegged & fungible IP Ownership (IPO) allow decentralized governance of the development of an IPS.

⚙️   Professional Networking: IPO is leveraged to form partnerships with individuals with the skills and/or resources to actualize.

### IP Ownership (IPO)

⚙️   IP Ownership: Fractional ownership tokens are minted in quantities of 10k, each representing a 0.01% stake over a IPS.

⚙️   Leveraging IPO: IPO can be leveraged in exchange for not just skills (i.e. partnerships), but also in exchange for capital.

<div align="center">
<img src="https://i.ibb.co/7NKWDM6/Screen-Shot-2021-08-28-at-5-41-35-PM.png" style="align-center">
</div>

## Testing

Clone:

`git clone https://github.com/InvArch/InvArch-Pallet-Library`

Test:

`cd InvArch-Pallet-Library
cargo test`

## Components

### IP Protocol & Pallets
* `Pallet_ips` - Provides basic functionality for creating and managing an `IPSet`. You can think of an `IPSet` as an idea, which is basically a collection of components (intellectual property tokens) that define and describe that idea.
* `Pallet_ipt` - Provides basic functionality for creating and managing an `IPToken`. You can think of an `IPToken` as a component of an idea. For example, a business summary PDF file, or even a 3D rendering of a prototype mold. When combined and stored in an `IPSet`, that collection forms the foundtion for an idea. The more detailed and/or comprehensive an `IPSet` is, the stronger the idea.
* `Pallet_ipo` - Provides basic functionality for creating and managing a `IPOwnership` tokens. You can think of `IPOwnership` tokens as a form of fungible and fractionalized ownership that are built-in to every `IPSet`. 

### DEV Protocol & Pallets
* `Pallet_dev` - Provides basic functionality for creating and managing a `DEV`(Decentralized Entrepreneurial Venture). You can think of a `DEV` as an agreement between multiple parties to come together as cofounders over a project in order to contribute towards an `IPSet`'s actualization.
* `Pallet_dao` - Provides basic functionality for creating and managing a `DAO` that helps govern a `DEV`. You can think of a `DAO` as a `DEV`'s governance mechanism. It helps regulate the and ensure the integrity and prudence of participants within a `DEV`.
* `Pallet_worklog` - Provides basic functionality for creating and managing a `WorkLog` within a `DEV`. You can think of a `Worklog` as a `DEV`'s method of recording and storing milestone/deliverables progressions and completions.
* `Pallet_deliverables` - Provides basic functionality for creating and managing a `Deliverables` distribution mechainism for `IPOwnership` throughout a `DEV`. You can think of `Deliverables` as a mechanism for automatically distributing `IPOwnership` tokens to participants in a `DEV` as milestones/deliverables are met and confirmed by its `Worklog`.
* `Pallet_listings` - Provides basic functionality for creating and managing a `Listing` for a `DEV`'s `IPOwnership` tokens. `Listings` allows for public listings of `IPOwnership` to be purchased by outside participants/investors.

## How to contribute

I'm really glad you're reading this, because we need volunteer developers to help this idea become a reality!

If you haven't already, come find us on the [#InvArch Discord](https://discord.gg/J3hapvrpZJ). We want you working on things you're excited about!

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
