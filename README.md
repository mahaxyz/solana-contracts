# Solana Tax Token Launchpad

A Solana program intended for creating tax tokens with automated liquidity pool setup on Raydium.

## Overview

This program aims to enable users to:

1. Create a new token with transfer fee capabilities (tax token)
2. Pay a creation fee in SOL
3. Automatically create a Raydium CLMM pool (concentrated liquidity) for the tax token and WSOL
4. Add initial liquidity to the pool

The project is designed to demonstrate the implementation of a token with transfer fee (tax) functionality on Solana. It should utilize the SPL Token 2022 program's TransferFee extension to automatically collect fees on each token transfer.

## How It Works

The main functionality is intended to be exposed through the `create_and_buy` instruction, which should handle the entire process in one transaction:

```rust
pub fn create_and_buy(ctx: Context<CreateAndBuy>, params: CreateTokenParams) -> Result<()>
```

### Intended Process Flow

1. **Fee Collection**: Should collect SOL creation fee from the user
2. **Token Creation**: Should create a new SPL Token-2022 token with transfer fee extension
3. **Pool Creation**: Should create a Raydium CLMM pool for the tax token and WSOL
4. **Liquidity Addition**: Should add single-sided liquidity to the pool

### Intended Code Flow

The `create_and_buy` function should execute the following steps in sequence:

1. **Handle Creation Fees**
   ```rust
   handle_creation_fees(&ctx, params.creation_fee)?;
   ```
   - Should transfer SOL from the payer to the fee receiver using the System Program's transfer instruction
   - This fee is intended for the service of creating the token and setting up the liquidity pool

2. **Create Tax Token**
   ```rust
   create_token(&ctx, params.transfer_fee_basis_points, params.maximum_fee)?;
   ```
   - Should calculate required space for the mint account with TransferFeeConfig extension
   - Should create the mint account with proper space allocation
   - Should initialize the TransferFeeConfig extension with specified fee parameters:
     - transfer_fee_basis_points: percentage of each transfer to collect as fee
     - maximum_fee: maximum amount to collect per transfer
   - Should initialize the mint with standard parameters (decimals, authorities)

3. **Add Single-Sided Liquidity**
   ```rust
   handle_add_single_side_liquidity(
       &ctx,
       params.tick_lower_index,
       params.tick_upper_index,
       params.tick_array_lower_start_index,
       params.tick_array_upper_start_index,
       params.liquidity,
       params.amount_tax_token_max,
       params.amount_sol_max,
   )?;
   ```
   - Should create a Raydium CLMM pool for the tax token and WSOL pair if it doesn't exist
   - Should calculate sqrt_price_x64 from tick_lower_index for pool initialization
   - Should create the pool with the calculated price and current timestamp
   - Should open a new position in the pool with specified tick ranges
   - Should add liquidity to the position with the specified amounts of tax token and SOL
   - This is intended to establish initial trading liquidity for the newly created token

The process aims to create a token with tax capabilities and immediate trading liquidity, which should allow users to trade the token on Raydium's CLMM DEX right after creation.

### Required Parameters

The `CreateTokenParams` struct should contain all necessary parameters:

```rust
pub struct CreateTokenParams {
    pub transfer_fee_basis_points: u16,  // The tax rate (in basis points)
    pub maximum_fee: u64,                // Maximum fee per transaction
    pub creation_fee: u64,               // SOL fee to create the token
    pub tick_lower_index: i32,           // CLMM lower tick index
    pub tick_upper_index: i32,           // CLMM upper tick index
    pub tick_array_lower_start_index: i32, // CLMM lower tick array start
    pub tick_array_upper_start_index: i32, // CLMM upper tick array start
    pub liquidity: u128,                 // Initial liquidity amount
    pub amount_tax_token_max: u64,       // Max tax tokens to add as liquidity
    pub amount_sol_max: u64,             // Max SOL to add as liquidity
}
```

## Expected Account Structure

To execute the `create_and_buy` instruction, the following accounts should be provided:

- **payer**: User paying for the transaction and creation fee
- **mint_account**: New token's mint account (signer)
- **fee_receiver**: Account that should receive the creation fee
- **token_program_2022**: SPL Token-2022 program
- **clmm_program**: Raydium CLMM program
- **amm_config**: Raydium AMM configuration
- Various position accounts for the CLMM pool
- Token accounts for the tax token and WSOL

## Planned Features

- Token creation with configurable transfer fee
- Automatic fee collection on token transfers
- Automatic pool creation with the tax token and WSOL
- Single-sided liquidity provision

## Transfer Fee Configuration

When initializing a new tax token, users should be able to configure:
- `transfer_fee_basis_points`: The fee percentage in basis points (e.g., 100 = 1%)
- `maximum_fee`: The maximum fee amount in token units

## Expected Fee Mechanism

Fees should be automatically withheld in the recipient account during transfers. The fee amount should be calculated as:
```
fee = min(transfer_amount * transfer_fee_basis_points / 10000, maximum_fee)
```

## Prerequisites

- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://project-serum.github.io/anchor/getting-started/installation.html)
- [Node.js](https://nodejs.org/en/)

## Setup

1. Clone the repository:
```sh
git clone <repository-url>
cd maha-solana-contracts
```

2. Install dependencies:
```sh
yarn install  # or npm install
```

3. Build the program:
```sh
anchor build
```

4. Deploy:
```sh
anchor deploy
```

## Development

### Building

```bash
anchor build
```

### Testing

```bash
anchor test
```

## Intended Usage Example

```typescript
// Client-side example of how creating a tax token and pool should work
const createAndBuyIx = await program.methods
  .createAndBuy({
    transferFeeBasisPoints: 100, // 1% fee
    maximumFee: new BN(1000000), // 0.01 tokens max fee
    creationFee: new BN(100000000), // 0.1 SOL
    tickLowerIndex: -10, 
    tickUpperIndex: 10,
    tickArrayLowerStartIndex: -10,
    tickArrayUpperStartIndex: 10,
    liquidity: new BN("1000000000"),
    amountTaxTokenMax: new BN(1000000000),
    amountSolMax: new BN(100000000),
  })
  .accounts({
    // All required accounts...
  })
  .signers([user, mintKeypair])
  .rpc();
```

## License

MIT License