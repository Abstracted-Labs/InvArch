# Checked Inflation Pallet

## Overview

The Checked Inflation Pallet is designed to facilitate the inflationary aspect of a blockchain's economy. 
It automatically mints new tokens at the start of every era, with the amount determined by a configurable inflation method. 
This functionality is crucial for maintaining a controlled expansion of the token supply, aligning with economic models or rewarding network participants.

### Key Features

- **Configurable Inflation**: The amount and method of inflation can be tailored to suit the blockchain's economic model.
- **Automatic Token Minting**: New tokens are minted automatically at the beginning of each era.
- **Yearly and Era-Based Inflation**: Supports fixed yearly, fixed per era, or rate-based inflation calculations.

## Functionality

The pallet's core functionality revolves around the `on_initialize` hook, which triggers at the beginning of each block. 
If conditions align (start of a new era or year), the pallet calculates the amount to mint based on the configured inflation method and mints the tokens.

## Inflation Methods

Inflation can be configured in one of three ways, as defined in the `InflationMethod` enum:

- **Rate**: A percentage of the current supply.
- **FixedYearly**: A fixed amount distributed evenly across all eras in a year.
- **FixedPerEra**: A fixed amount minted at the start of each era.

The choice of method allows for flexibility in how the token supply expands over time, catering to different economic strategies.

## Dispatchable Functions

### `set_first_year_supply`

Configures the initial token supply at the year's start, preparing the system for accurate inflation calculation.

- **Access Control**: Root

### `halt_unhalt_pallet`

Toggles the inflation process, allowing it to be halted or resumed based on network needs.

- **Parameters**:
  - `halt`: A boolean indicating whether to halt (`true`) or resume (`false`) the inflation process.
- **Access Control**: Root


## Events

- **NewYear**: Marks the beginning of a new year, resetting era counts and updating the starting issuance for inflation calculations.
- **NewEra**: Signifies the start of a new era, triggering token minting according to the configured inflation rate.
- **InflationMinted**: Indicates that tokens have been minted due to inflation, detailing the amounts involved.
- **OverInflationDetected**: Warns of excess token minting beyond expected amounts, prompting corrective measures.
- **HaltChanged**: Reports changes in the inflation process's halt status.

## Errors

- **NoHaltChange**: Triggered when attempting to change the halt status to its current value, indicating no action is needed.

## Conclusion

The Checked Inflation Pallet offers a flexible and automated way to manage token supply expansion through inflation. 
By configuring the inflation method to match your blockchain's economic model, you can ensure a controlled and predictable increase in token supply, 
this pallet is an essential tool for managing network growth and stability through controlled inflation in response to evolving economic conditions, 
ensuring long-term sustainability.

