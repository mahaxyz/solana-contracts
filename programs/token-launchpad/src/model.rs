use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateTokenParams {
    pub token_name: String,
    pub token_symbol: String,
    pub token_metadata_uri: String,
    pub token_supply: u64,
    pub buy_amount: u64,
    pub bump: u8,
}

#[account]
#[derive(Default)]
pub struct LaunchpadConfig {
    pub authority: Pubkey,
    pub creation_fee: u64,
    pub bump: u8,
} 