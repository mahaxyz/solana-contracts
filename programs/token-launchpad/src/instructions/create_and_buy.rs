use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata,
    token::Token,
    token_2022::{
        spl_token_2022::{
            extension::{
                transfer_fee::TransferFeeConfig, BaseStateWithExtensions, ExtensionType,
                StateWithExtensions,
            },
            pod::PodMint,
            state::Mint as MintState,
        },
        initialize_mint2, transfer_fee_initialize, InitializeMint2, Token2022, TransferFeeInitialize,
    },
    token_interface::{Mint, TokenAccount, TokenInterface, spl_pod::optional_keys::OptionalNonZeroPubkey},
};
use anchor_lang::system_program::{create_account, CreateAccount};

// Import this to use the add_single_side_liquidity CPI
use crate::cpi_interfaces::clmm;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateTokenParams {
    pub transfer_fee_basis_points: u16,
    pub maximum_fee: u64,
    pub creation_fee: u64,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
    pub liquidity: u128,
    pub amount_tax_token_max: u64,
    pub amount_sol_max: u64,
}

#[derive(Accounts)]
pub struct CreateAndBuy<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub mint_account: Signer<'info>,
    
    // Fee collector account
    /// CHECK: This is just a receiver
    #[account(mut)]
    pub fee_receiver: UncheckedAccount<'info>,

    // Tax token program accounts
    pub token_program_2022: Program<'info, Token2022>,
    
    // CLMM-related accounts
    pub clmm_program: Program<'info, clmm::RaydiumClmm>,
    pub amm_config: Box<Account<'info, clmm::AmmConfig>>,
    
    /// CHECK: Position NFT owner
    pub position_nft_owner: UncheckedAccount<'info>,
    
    /// Position NFT mint
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,
    
    /// CHECK: Token account for position NFT
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,
    
    /// CHECK: Metadata account
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
    
    /// Pool state account
    #[account(mut)]
    pub pool_state: AccountLoader<'info, clmm::PoolState>,
    
    /// CHECK: Protocol position
    #[account(mut)]
    pub protocol_position: UncheckedAccount<'info>,
    
    /// CHECK: Lower tick array
    #[account(mut)]
    pub tick_array_lower: UncheckedAccount<'info>,
    
    /// CHECK: Upper tick array
    #[account(mut)]
    pub tick_array_upper: UncheckedAccount<'info>,
    
    /// CHECK: Personal position
    #[account(mut)]
    pub personal_position: UncheckedAccount<'info>,
    
    // WSOL mint
    pub wsol_mint: Box<InterfaceAccount<'info, Mint>>,
    
    /// Tax token account
    #[account(mut)]
    pub tax_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// WSOL account
    #[account(mut)]
    pub wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// Tax token vault
    #[account(mut)]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// WSOL vault
    #[account(mut)]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// CHECK: Observation state
    #[account(mut)]
    pub observation_state: UncheckedAccount<'info>,
    
    /// CHECK: Tick array bitmap
    #[account(mut)]
    pub tick_array_bitmap: UncheckedAccount<'info>,
    
    // Other required programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn process_create_and_buy(ctx: Context<CreateAndBuy>, params: CreateTokenParams) -> Result<()> {
    // 1. Handle creation fees - charge SOL from the user
    handle_creation_fees(&ctx, params.creation_fee)?;
    
    // 2. Create the tax token
    create_token(
        &ctx,
        params.transfer_fee_basis_points,
        params.maximum_fee,
    )?;
    
    // 3. Add single-sided liquidity
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
    
    Ok(())
}

fn handle_creation_fees(ctx: &Context<CreateAndBuy>, fee: u64) -> Result<()> {
    // Transfer SOL from payer to fee_receiver
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.fee_receiver.to_account_info(),
            },
        ),
        fee,
    )?;
    
    Ok(())
}

fn create_token(
    ctx: &Context<CreateAndBuy>,
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> Result<()> {
    // Calculate space required for mint and extension data
    let mint_size =
        ExtensionType::try_calculate_account_len::<PodMint>(&[ExtensionType::TransferFeeConfig])?;

    // Calculate minimum lamports required for size of mint account with extensions
    let lamports = (Rent::get()?).minimum_balance(mint_size);

    // Invoke System Program to create new account with space for mint and extension data
    create_account(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.mint_account.to_account_info(),
            },
        ),
        lamports,                          // Lamports
        mint_size as u64,                  // Space
        &ctx.accounts.token_program_2022.key(), // Owner Program
    )?;

    // Initialize the transfer fee extension data
    // This instruction must come before the instruction to initialize the mint data
    transfer_fee_initialize(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            TransferFeeInitialize {
                token_program_id: ctx.accounts.token_program_2022.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
            },
        ),
        Some(&ctx.accounts.payer.key()), // transfer fee config authority (update fee)
        Some(&ctx.accounts.payer.key()), // withdraw authority (withdraw fees)
        transfer_fee_basis_points,       // transfer fee basis points (% fee per transfer)
        maximum_fee,                     // maximum fee (maximum units of token per transfer)
    )?;

    // Initialize the standard mint account data
    initialize_mint2(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            InitializeMint2 {
                mint: ctx.accounts.mint_account.to_account_info(),
            },
        ),
        2,                               // decimals
        &ctx.accounts.payer.key(),       // mint authority
        Some(&ctx.accounts.payer.key()), // freeze authority
    )?;

    // Verify mint data
    let mint = &ctx.accounts.mint_account.to_account_info();
    let mint_data = mint.data.borrow();
    let mint_with_extension = StateWithExtensions::<MintState>::unpack(&mint_data)?;
    let extension_data = mint_with_extension.get_extension::<TransferFeeConfig>()?;

    assert_eq!(
        extension_data.transfer_fee_config_authority,
        OptionalNonZeroPubkey::try_from(Some(ctx.accounts.payer.key()))?
    );

    assert_eq!(
        extension_data.withdraw_withheld_authority,
        OptionalNonZeroPubkey::try_from(Some(ctx.accounts.payer.key()))?
    );

    msg!("Tax token created successfully");
    
    Ok(())
}

fn handle_add_single_side_liquidity(
    ctx: &Context<CreateAndBuy>,
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
    liquidity: u128,
    amount_tax_token_max: u64,
    amount_sol_max: u64,
) -> Result<()> {
    // Use CPI to call the add_single_side_liquidity on Raydium CLMM program
    clmm::add_single_side_liquidity(
        CpiContext::new(
            ctx.accounts.clmm_program.to_account_info(),
            clmm::AddSingleSideLiquidity {
                payer: ctx.accounts.payer.to_account_info(),
                position_nft_owner: ctx.accounts.position_nft_owner.to_account_info(),
                position_nft_mint: ctx.accounts.position_nft_mint.to_account_info(),
                position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
                metadata_account: ctx.accounts.metadata_account.to_account_info(),
                amm_config: ctx.accounts.amm_config.to_account_info(),
                pool_state: ctx.accounts.pool_state.to_account_info(),
                protocol_position: ctx.accounts.protocol_position.to_account_info(),
                tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
                tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
                personal_position: ctx.accounts.personal_position.to_account_info(),
                token_mint_0: ctx.accounts.mint_account.to_account_info(),
                token_mint_1: ctx.accounts.wsol_mint.to_account_info(),
                token_account_0: ctx.accounts.tax_token_account.to_account_info(),
                token_account_1: ctx.accounts.wsol_account.to_account_info(),
                token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
                token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
                observation_state: ctx.accounts.observation_state.to_account_info(),
                tick_array_bitmap: ctx.accounts.tick_array_bitmap.to_account_info(),
                token_program_0: ctx.accounts.token_program_2022.to_account_info(),
                token_program_1: ctx.accounts.token_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                metadata_program: ctx.accounts.metadata_program.to_account_info(),
                token_program_2022: ctx.accounts.token_program_2022.to_account_info(),
                vault_0_mint: ctx.accounts.mint_account.to_account_info(),
                vault_1_mint: ctx.accounts.wsol_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        tick_lower_index,
        tick_upper_index,
        tick_array_lower_start_index,
        tick_array_upper_start_index,
        liquidity,
        amount_tax_token_max,
        amount_sol_max,
        true, // with_metadata
        Some(true), // base_flag
    )?;

    msg!("Single-sided liquidity added successfully");
    
    Ok(())
}
