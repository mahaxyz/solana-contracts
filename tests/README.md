# Solana Tax Token Tests

This directory contains tests for the Solana Tax Token contract. The tests verify that all tax token functionality works correctly, including token creation, transfers with fees, fee harvesting, and fee withdrawal.

## Test Files

- **tax-token.ts**: Tests for the tax token contract using SPL Token 2022 extensions

## Running Tests

To run the tests, you need to have a local Solana validator running:

```sh
# Start a local validator
solana-test-validator --reset --quiet &

# Deploy the program
anchor deploy

# Run the tests
ANCHOR_PROVIDER_URL=http://localhost:8899 ANCHOR_WALLET=/path/to/your/wallet.json npx ts-mocha -p ./tsconfig.json -t 1000000 tests/tax-token.ts
```

## Test Coverage

The tests cover the following functionality:

### 1. Token Creation with Transfer Fee

Tests the creation of a new token mint with transfer fee extension. This includes:
- Creating the mint account with appropriate space for extensions
- Initializing the TransferFee extension with specified parameters
- Initializing the standard mint data
- Verifying the fee parameters

### 2. Token Transfer with Fee

Tests transferring tokens between accounts with automatic fee deduction. This includes:
- Transferring tokens from one account to another
- Verifying the correct amount is received by the recipient
- Verifying the correct fee is withheld in the recipient's account

### 3. Fee Harvesting

Tests harvesting withheld fees from token accounts into the mint. This includes:
- Checking withheld fees in the token account before harvesting
- Harvesting fees to the mint
- Verifying the withheld fees are correctly transferred to the mint
- Verifying the token account's withheld fees are reset to zero

### 4. Fee Withdrawal

Tests withdrawing collected fees from the mint to a designated wallet. This includes:
- Checking the mint's withheld fees before withdrawal
- Withdrawing fees to the specified token account
- Verifying the correct amount is received by the token account
- Verifying the mint's withheld fees are reset to zero

### 5. Fee Configuration Update

Demonstrates how to update the fee configuration parameters. Currently, this test is partially implemented due to limitations in the current version of the SPL Token library.

## Test Implementation

The tests are implemented using:
- **@coral-xyz/anchor**: For Anchor framework integration
- **@solana/spl-token**: For SPL Token 2022 functionality
- **@solana/web3.js**: For Solana web3 functionality
- **mocha**: For test structure

## Extending the Tests

To add new tests:

1. **Add a new test file**: Create a new .ts file in the tests directory
2. **Import the required dependencies**:
   ```typescript
   import * as anchor from "@coral-xyz/anchor";
   import { TOKEN_2022_PROGRAM_ID, ... } from "@solana/spl-token";
   import { Keypair, ... } from '@solana/web3.js';
   ```
3. **Set up the test environment**:
   ```typescript
   describe("your-test-description", () => {
     const provider = anchor.AnchorProvider.env();
     anchor.setProvider(provider);
     
     const programId = new PublicKey("YOUR_PROGRAM_ID");
     const idl = require("../target/idl/your_idl.json");
     const program = new anchor.Program(idl, programId, provider);
     
     // Add your test cases here
   });
   ```
4. **Add test cases**:
   ```typescript
   it("your test case description", async () => {
     // Your test code here
   });
   ```

## Testing Tips

1. **Test in Isolation**: Each test should be independent or clearly dependent on previous tests
2. **Account Management**: Properly manage keypairs and accounts for testing
3. **Error Handling**: Use try/catch blocks to properly handle and report errors
4. **Logging**: Use console.log for visibility into test execution
5. **Verification**: Always verify results by querying account state after operations 