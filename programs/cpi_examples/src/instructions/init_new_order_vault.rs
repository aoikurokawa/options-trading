use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitNewOrderVault<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        seeds = [&usdc_mint.key().to_bytes()[..], b"vault"],
        bump,
        payer = authority,
        token::mint = usdc_mint,
        token::authority = vault_authority,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub vault_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(_ctx: Context<InitNewOrderVault>) -> Result<()> {
    Ok(())
}
