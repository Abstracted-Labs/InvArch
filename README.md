# InvArch-Pallet-Library
## Intro ##
 This repository should contains the Pallets used in the InvArch blockchain, and reviews their relationships and functions. At the current stage, the goal of creating this document and repository is centered around getting feedback while we continue to write the code and develop InvArch. This is a **WIP.**

 ## What is InvArch? ##
  * InvArch is a next generation blockchain for intellectual property tokenization, development, & networking.
  * The InvArch platform provides utility for this new technology by allowing intellectual property tokens (IPTs) to be owned by a Decentralized Entrepreneurial Venture (DEV) contract and governed as a DAO using built-in fungible IP Ownership (IPO) tokens. These tokens may also be leveraged by participants in a DEV to raise startup funds for their projects.
  * InvArch is built using Substrate/Rust.
  * Every member of the team has an honest belief that this project will help make the world better through increased economic decentralization and by helping to catalyze future innovations, it's a belief that motivates and inspires every one of us to see this project through.

### Project Details

<div align=center>
  <img src="https://i.ibb.co/hFM47Qh/Screen-Shot-2021-09-11-at-4-39-30-PM.png">
</div>

### InvArch approaches ideas (IP) as a set of non-fungible components: 
* IP Set = Idea
* IP Tokens  = components of their idea. 
* An IP Set can have built-in IP Ownership tokens. 
* You can,`list`,`sell`,`buy`,`transfer`, and `destroy` an IP Set, but not individual IP Tokens, since they're only components. 
* A new IP set can be created (`create`) and new IPT can be minted (`mint`) and added to a Set.
* Existing IPT can be burned (`burn`) or amended (`amend`). 
* Subsequently, an entire IP Set could be destroyed (`destroy`) as well, burning all of its contents.

### Components

### 1. IP Protocol & Pallets
* `Pallet_ips` - Provides basic functionality for creating and managing an `IPSet`. You can think of an `IPSet` as an idea, which is basically a collection of components (intellectual property tokens) that define and describe that idea.
* `Pallet_ipt` - Provides basic functionality for creating and managing an `IPToken`. You can think of an `IPToken` as a component of an idea. For example, a business summary PDF file, or even a 3D rendering of a prototype mold. When combined and stored in an `IPSet`, that collection forms the foundation for an idea. The more detailed and/or comprehensive an `IPSet` is, the stronger the idea.
* `Pallet_ipo` - Provides basic functionality for creating and managing a `IPOwnership` tokens. You can think of `IPOwnership` tokens as a form of fungible and fractionalized ownership that are built-in to every `IPSet`. 

### 2. DEV Protocol & Pallets
* `Pallet_dev` - Provides basic functionality for creating and managing a `DEV`(Decentralized Entrepreneurial Venture). You can think of a `DEV` as an agreement between multiple parties to come together as cofounders over a project in order to contribute towards an `IPSet`'s actualization.
* `Pallet_dao` - Provides basic functionality for creating and managing a `DAO` that helps govern a `DEV`. You can think of a `DAO` as a `DEV`'s governance mechanism. It helps regulate the and ensure the integrity and prudence of participants within a `DEV`.
* `Pallet_worklog` - Provides basic functionality for creating and managing a `WorkLog` within a `DEV`. You can think of a `Worklog` as a `DEV`'s method of recording and storing milestone/deliverables progressions and completions.
* `Pallet_deliverables` - Provides basic functionality for creating and managing a `Deliverables` distribution mechainism for `IPOwnership` throughout a `DEV`. You can think of `Deliverables` as a mechanism for automatically distributing `IPOwnership` tokens to participants in a `DEV` as milestones/deliverables are met and confirmed by its `Worklog`.
* `Pallet_listings` - Provides basic functionality for creating and managing a `Listing` for a `DEV`'s `IPOwnership` tokens. `Listings` allows for public listings of `IPOwnership` to be purchased by outside participants/investors.

See the other pages in:
- [GitHub](https://github.com/InvArch)
- [Website](https://www.invarch.io/)
