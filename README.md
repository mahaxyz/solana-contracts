# Solana Tax Token Contract

A Solana smart contract implementing a token with tax/fee functionality using SPL Token 2022 extensions, built with Anchor framework.

## Overview

This project demonstrates the implementation of a token with transfer fee (tax) functionality on Solana. It uses the SPL Token 2022 program's TransferFee extension to automatically collect fees on each token transfer.

Key features:
- Token creation with configurable transfer fee
- Automatic fee collection on token transfers
- Fee harvesting from token accounts to the mint
- Fee withdrawal to a designated wallet
- Fee configuration updates

## Prerequisites

- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) v1.16.0 or later
- [Anchor](https://project-serum.github.io/anchor/getting-started/installation.html) v0.29.0 or later
- [Node.js](https://nodejs.org/en/) v16 or later
- [Yarn](https://yarnpkg.com/getting-started/install) or npm

## Setup

1. Clone the repository:
```sh
git clone https://github.com/yourusername/maha-solana-contracts.git
cd maha-solana-contracts
```

2. Install dependencies:
```sh
yarn install
```

3. Build the program:
```sh
anchor build
```

4. Start a local test validator:
```sh
solana-test-validator --reset
```

5. Deploy the program:
```sh
anchor deploy
```

## Contracts

### solana-tax-token-anchor

The main contract that implements the tax token functionality with the following instructions:

- `initialize`: Creates a new token mint with transfer fee configuration
- `transfer`: Transfers tokens between accounts (with automatic fee deduction)
- `harvest`: Collects withheld fees from token accounts into the mint
- `withdraw`: Withdraws collected fees from the mint to a designated wallet
- `update_fee`: Updates the transfer fee configuration

## Test

Run the tests with:

```sh
ANCHOR_PROVIDER_URL=http://localhost:8899 ANCHOR_WALLET=/path/to/your/wallet.json npx ts-mocha -p ./tsconfig.json -t 1000000 tests/tax-token.ts
```

The tests demonstrate:
1. Creating a token with transfer fee
2. Transferring tokens with fee deduction
3. Harvesting fees from token accounts
4. Withdrawing fees to the fee authority's wallet
5. Updating fee configuration

## Implementation Details

### Transfer Fee Configuration

When initializing a new tax token, you can configure:
- `transfer_fee_basis_points`: The fee percentage in basis points (e.g., 100 = 1%)
- `maximum_fee`: The maximum fee amount in token units

### Fee Mechanism

Fees are automatically withheld in the recipient account during transfers. The fee amount is calculated as:
```
fee = min(transfer_amount * transfer_fee_basis_points / 10000, maximum_fee)
```

### Harvesting Fees

Fees withheld in token accounts need to be harvested to the mint before withdrawal. This is done with the `harvest` instruction.

### Withdrawing Fees

After harvesting, fees can be withdrawn from the mint to a designated wallet using the `withdraw` instruction.

## Usage Example

```typescript
// Create a tax token with 1% fee
await program.methods
  .initialize(100, new anchor.BN(1_000_000))
  .accounts({
    payer: wallet.publicKey,
    mint_account: mintKeypair.publicKey,
    token_program: TOKEN_2022_PROGRAM_ID,
    system_program: anchor.web3.SystemProgram.programId,
  })
  .signers([mintKeypair])
  .rpc();

// Transfer tokens (fee is automatically applied)
await program.methods
  .transfer(new anchor.BN(100_000_000))
  .accounts({
    sender: wallet.publicKey,
    recipient: recipient.publicKey,
    mint_account: mintKeypair.publicKey,
    sender_token_account: senderATA,
    recipient_token_account: recipientATA,
    token_program: TOKEN_2022_PROGRAM_ID,
    associated_token_program: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    system_program: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

## License

This project is licensed under the MIT License.