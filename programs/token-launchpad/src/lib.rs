use anchor_lang::prelude::*;

// Import the modules
pub mod instructions;
pub mod cpi_interfaces;
mod model;
mod constants;

use model::CreateTokenParams;
use instructions::create_and_buy::{CreateAndBuy, process_create_and_buy};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// Main program module
#[program]
pub mod token_launchpad {
    use super::*;

    /// Creates a new token and lets the caller buy it
    pub fn create_and_buy(ctx: Context<CreateAndBuy>, params: CreateTokenParams) -> Result<()> {
        process_create_and_buy(ctx, params)
    }
}

/// Basic initialization context
#[derive(Accounts)]
pub struct Initialize {}

