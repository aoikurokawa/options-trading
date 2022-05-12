use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitMintVault<'info> {
    /// CHECK: TODO
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    pub underlying_asset: Box<Account<'info, Mint>>,
    #[account(
        init,
        seeds = [&underlying_asset.key().to_bytes()[..], b"vault"],
        bump,
        payer = authority,
        token::mint = underlying_asset,
        token::authority = vault_authority,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: TODO
    pub vault_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(_ctx: Context<InitMintVault>) -> Result<()> {
    Ok(())
}
