use anchor_lang::prelude::*;

declare_id!("AJBmAnT7A7RBxyK2USQUU1deZj6H3TLYM6RWuG3yaxoJ");

#[program]
pub mod solana_token_launchpad {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
