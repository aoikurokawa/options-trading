// use anchor_lang::prelude::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK: TODO
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub option_source: Box<Account<'info, TokenAccount>>,
    /// CHECK: TODO
    pub option_mint: AccountInfo<'info>,
    #[account(
        init,
        seeds = [&option_mint.key().to_bytes()[..], b"vault"],
        bump,
        payer = authority,
        token::mint = option_mint,
        token::authority = vault_authority
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: TODO
    pub vault_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, amount: u64) -> Result<()> {
    let cpi_accounts = Transfer {
        from: ctx.accounts.option_source.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.authority.clone(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_token_program.to_account_info(), cpi_accounts);
    transfer(cpi_ctx, amount)?;
    Ok(())
}
