# Solana Tax Token Anchor Program

An implementation of a token with automatic transfer fee/tax functionality built on Solana using SPL Token 2022 extensions and the Anchor framework.

## Overview

This program leverages SPL Token 2022's TransferFee extension to create tokens that automatically deduct a fee on each transfer. The program provides functionality for:

1. Creating tokens with a configurable transfer fee
2. Transferring tokens (with automatic fee deduction)
3. Harvesting fees from token accounts to the mint
4. Withdrawing fees to a designated wallet
5. Updating fee configuration parameters

## Technical Implementation

### Architecture

The program is organized into the following components:

- **Instructions**: Individual operations supported by the contract
- **Accounts**: Structs defining the accounts required for each instruction
- **State**: Token state with TransferFee extension

### Instructions

#### initialize

Creates a new token mint with the transfer fee extension. This process involves:

1. Creating the mint account with appropriate space for extensions
2. Initializing the TransferFee extension
3. Initializing the standard mint data

```rust
pub fn process_initialize(
    ctx: Context<Initialize>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<()>
```

#### transfer

Transfers tokens between accounts using the SPL Token 2022 program, with fee automatically deducted.

```rust
pub fn process_transfer(ctx: Context<Transfer>, amount: u64) -> Result<()>
```

#### harvest

Collects withheld fees from token accounts into the mint. This allows fees that have been collected in various token accounts to be consolidated in the mint.

```rust
pub fn process_harvest(ctx: Context<Harvest>, sources: Vec<Pubkey>) -> Result<()>
```

#### withdraw

Withdraws collected fees from the mint to a designated wallet.

```rust
pub fn process_withdraw(ctx: Context<Withdraw>) -> Result<()>
```

#### update_fee

Updates the transfer fee configuration parameters.

```rust
pub fn process_update_fee(
    ctx: Context<UpdateFee>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<()>
```

### Accounts

Each instruction requires specific accounts:

#### Initialize
- `payer`: The account that pays for the initialization
- `mint_account`: The mint account to be created
- `token_program`: SPL Token 2022 program
- `system_program`: Solana System program

#### Transfer
- `sender`: Token sender
- `recipient`: Token recipient
- `mint_account`: The token mint
- `sender_token_account`: Sender's token account
- `recipient_token_account`: Recipient's token account
- `token_program`: SPL Token 2022 program
- `associated_token_program`: Associated Token program
- `system_program`: Solana System program

#### Harvest
- `mint_account`: The token mint
- `token_program`: SPL Token 2022 program
- (Sources are passed as remaining accounts)

#### Withdraw
- `authority`: The fee withdrawal authority
- `mint_account`: The token mint
- `token_account`: Account to receive the withdrawn fees
- `token_program`: SPL Token 2022 program

#### UpdateFee
- `authority`: The fee update authority
- `mint_account`: The token mint
- `token_program`: SPL Token 2022 program

## Fee Calculation

The fee amount is calculated as:
```
fee = min(transfer_amount * transfer_fee_basis_points / 10000, maximum_fee)
```

Where:
- `transfer_fee_basis_points` is the fee percentage in basis points (e.g., 100 = 1%)
- `maximum_fee` is the maximum fee amount in token units

## Usage Notes

1. **Fee Collection**: Fees are automatically withheld in the recipient's token account during transfers
2. **Fee Harvesting**: Withheld fees need to be harvested to the mint before withdrawal
3. **Fee Withdrawal**: After harvesting, fees can be withdrawn to the designated wallet
4. **Fee Update**: Fee configurations can be updated, but changes take effect after a delay (2 epochs)

## Integration with SPL Token 2022

This program extensively uses SPL Token 2022's CPI (Cross-Program Invocation) calls to interact with the token system, including:

- Creating token mints with extensions
- Initializing transfer fee configuration
- Transferring tokens with fees
- Harvesting withheld fees
- Withdrawing fees to token accounts
- Updating fee parameters

## Security Considerations

1. Only the authority designated during initialization can withdraw fees or update fee parameters
2. The program validates that the mint has the TransferFee extension before operations
3. Proper error handling for all operations
4. Account validation to prevent unauthorized operations 