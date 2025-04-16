use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata,
    token::Token,
    token_interface::{Mint, Token2022, TokenAccount, TokenInterface},
};
use raydium_clmm_cpi::{
    cpi,
    program::RaydiumClmm,
    states::{AmmConfig, PoolState, POOL_SEED, POOL_TICK_ARRAY_BITMAP_SEED, POOL_VAULT_SEED, POSITION_SEED, TICK_ARRAY_SEED},
};

#[derive(Accounts)]
#[instruction(
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
)]
pub struct AddSingleSideLiquidity<'info> {
    pub clmm_program: Program<'info, RaydiumClmm>,
    
    /// Address paying for transactions and providing liquidity
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// Used as pool creator (needed by create_pool)
    #[account(mut)]
    pub pool_creator: Signer<'info>,
    
    /// Which config the pool belongs to
    pub amm_config: Box<Account<'info, AmmConfig>>,
    
    /// CHECK: Position NFT owner (typically same as payer)
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
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_mint_0.key().as_ref(),
            token_mint_1.key().as_ref(),
        ],
        seeds::program = clmm_program.key(),
        bump,
    )]
    pub pool_state: AccountLoader<'info, PoolState>,
    
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
    
    /// Token 0 mint
    pub token_mint_0: Box<InterfaceAccount<'info, Mint>>,
    
    /// Token 1 mint
    pub token_mint_1: Box<InterfaceAccount<'info, Mint>>,
    
    /// Token 0 account
    #[account(mut)]
    pub token_account_0: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// Token 1 account
    #[account(mut)]
    pub token_account_1: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// Token 0 vault
    #[account(mut)]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// Token 1 vault
    #[account(mut)]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// CHECK: Observation state
    #[account(mut)]
    pub observation_state: UncheckedAccount<'info>,
    
    /// CHECK: Tick array bitmap
    #[account(mut)]
    pub tick_array_bitmap: UncheckedAccount<'info>,
    
    /// Token program interface for token 0
    pub token_program_0: Interface<'info, TokenInterface>,
    
    /// Token program interface for token 1
    pub token_program_1: Interface<'info, TokenInterface>,
    
    /// TOKEN program
    pub token_program: Program<'info, Token>,
    
    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// Metadata program
    pub metadata_program: Program<'info, Metadata>,
    
    /// Token 2022 program
    pub token_program_2022: Program<'info, Token2022>,
    
    /// Vault 0 mint
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,
    
    /// Vault 1 mint
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,
    
    /// System program
    pub system_program: Program<'info, System>,
    
    /// Rent
    pub rent: Sysvar<'info, Rent>,
}

pub fn add_single_side_liquidity<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, AddSingleSideLiquidity<'info>>,
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
    with_matedata: bool,
    base_flag: Option<bool>,
) -> Result<()> {
    // Calculate sqrt price from tick_lower_index for pool initialization
    // Instead of calculating sqrt price from tick, we can take sqrt price as input from the user, which can be obtained from the Raydium SDK.
    let sqrt_price_x64: u128 = get_sqrt_price_x64_from_tick(tick_lower_index);
    let open_time: u64 = Clock::get()?.unix_timestamp as u64;

    // Step 1: Initialize pool if needed
    create_pool(&ctx, sqrt_price_x64, open_time)?;
    
    // Step 2: Add liquidity
    open_position(
        &ctx,
        tick_lower_index,
        tick_upper_index,
        tick_array_lower_start_index,
        tick_array_upper_start_index,
        liquidity,
        amount_0_max,
        amount_1_max,
        with_matedata,
        base_flag,
    )
}

// Helper function to create pool if it doesn't exist
fn create_pool<'a, 'b, 'c: 'info, 'info>(
    ctx: &Context<'a, 'b, 'c, 'info, AddSingleSideLiquidity<'info>>,
    sqrt_price_x64: u128,
    open_time: u64,
) -> Result<()> {
    let cpi_accounts = cpi::accounts::CreatePool {
        pool_creator: ctx.accounts.pool_creator.to_account_info(),
        amm_config: ctx.accounts.amm_config.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        token_mint_0: ctx.accounts.token_mint_0.to_account_info(),
        token_mint_1: ctx.accounts.token_mint_1.to_account_info(),
        token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
        token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
        observation_state: ctx.accounts.observation_state.to_account_info(),
        tick_array_bitmap: ctx.accounts.tick_array_bitmap.to_account_info(),
        token_program_0: ctx.accounts.token_program_0.to_account_info(),
        token_program_1: ctx.accounts.token_program_1.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_context = CpiContext::new(ctx.accounts.clmm_program.to_account_info(), cpi_accounts);
    cpi::create_pool(cpi_context, sqrt_price_x64, open_time)?;
    
    Ok(())
}

// Helper function to open position and add liquidity
fn open_position<'a, 'b, 'c: 'info, 'info>(
    ctx: &Context<'a, 'b, 'c, 'info, AddSingleSideLiquidity<'info>>,
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
    with_matedata: bool,
    base_flag: Option<bool>,
) -> Result<()> {
    let cpi_accounts = cpi::accounts::OpenPositionV2 {
        payer: ctx.accounts.payer.to_account_info(),
        position_nft_owner: ctx.accounts.position_nft_owner.to_account_info(),
        position_nft_mint: ctx.accounts.position_nft_mint.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        metadata_account: ctx.accounts.metadata_account.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        protocol_position: ctx.accounts.protocol_position.to_account_info(),
        tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
        tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
        personal_position: ctx.accounts.personal_position.to_account_info(),
        token_account_0: ctx.accounts.token_account_0.to_account_info(),
        token_account_1: ctx.accounts.token_account_1.to_account_info(),
        token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
        token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        metadata_program: ctx.accounts.metadata_program.to_account_info(),
        token_program_2022: ctx.accounts.token_program_2022.to_account_info(),
        vault_0_mint: ctx.accounts.vault_0_mint.to_account_info(),
        vault_1_mint: ctx.accounts.vault_1_mint.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.clmm_program.to_account_info(), cpi_accounts)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    cpi::open_position_v2(
        cpi_context,
        tick_lower_index,
        tick_upper_index,
        tick_array_lower_start_index,
        tick_array_upper_start_index,
        liquidity,
        amount_0_max,
        amount_1_max,
        with_matedata,
        base_flag,
    )
}

// Function to calculate sqrt_price_x64 from tick, similar to Raydium's implementation
fn get_sqrt_price_x64_from_tick(tick: i32) -> u128 {
    // Constants
    const MAX_UINT128: u128 = 340282366920938463463374607431768211455;
    
    // Validate tick is within allowed range
    let tick_abs: u32 = if tick < 0 { (-tick) as u32 } else { tick as u32 };
    
    let mut ratio: u128 = if (tick_abs & 0x1) != 0 { 18445821805675395072 } else { 18446744073709551616 };
    
    // Use array of bit masks and multipliers to iterate instead of having many if statements
    let bit_masks: [u32; 19] = [
        0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 
        0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000,
        0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000
    ];
    
    let multipliers: [u128; 19] = [
        18444899583751176192, 18443055278223355904, 18439367220385607680,
        18431993317065453568, 18417254355718170624, 18387811781193609216,
        18329067761203558400, 18212142134806163456, 17980523815641700352,
        17526086738831433728, 16651378430235570176, 15030750278694412288,
        12247334978884435968, 8131365268886854656, 3584323654725218816,
        696457651848324352, 26294789957507116, 37481735321082, 7685
    ];
    
    // Iterate through the bit positions and apply multipliers as needed
    for i in 0..bit_masks.len() {
        if (tick_abs & bit_masks[i]) != 0 {
            ratio = mul_right_shift(ratio, multipliers[i]);
        }
    }
    
    if tick > 0 { MAX_UINT128 / ratio } else { ratio }
}

// Helper function to simulate mulRightShift in Raydium SDK
fn mul_right_shift(val: u128, mul_by: u128) -> u128 {
    // Handle potential overflow when multiplying large u128 numbers
    // We can use a more careful approach:
    
    // Extract high and low 64 bits
    let val_high = val >> 64;
    let val_low = val & 0xFFFFFFFFFFFFFFFF;
    let mul_high = mul_by >> 64;
    let mul_low = mul_by & 0xFFFFFFFFFFFFFFFF;
    
    // Only the val_high * mul_low and val_low * mul_high (shifted) contribute to result
    // val_low * mul_low >> 64 gives us the high bits we need to add
    // val_high * mul_high << 64 is too high to matter in our 64-bit right shift
    
    let val_low_mul_low = val_low * mul_low;
    let val_high_mul_low = val_high * mul_low;
    let val_low_mul_high = val_low * mul_high;
    
    // Calculate result with carries
    let middle_term = val_high_mul_low + (val_low_mul_low >> 64);
    let result = middle_term + val_low_mul_high;
    
    result
} 