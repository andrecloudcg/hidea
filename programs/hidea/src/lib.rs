use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod hidea {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Hello Anchor!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
