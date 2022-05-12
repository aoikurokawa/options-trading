use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use options_trading::cpi::accounts::MintOptionV2;
use options_trading::state::OptionMarket;

#[derive(Accounts)]
pub struct MintCtx<'info> {
    /// CHECK: TODO
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    /// CHECK: TODO
    pub option_trading_program: AccountInfo<'info>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: TODO
    #[account(mut)]
    pub vault_authority: AccountInfo<'info>,

    /// CHECK: TODO
    pub underlying_asset_mint: AccountInfo<'info>,
    #[account(mut)]
    pub underlying_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub minted_option_dest: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub writer_token_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub minted_writer_token_dest: Box<Account<'info, TokenAccount>>,
    pub option_market: Box<Account<'info, OptionMarket>>,
    /// CHECK: TODO
    #[account(mut)]
    pub fee_owner: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK: TODO
    pub associated_token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, MintCtx<'info>>,
    size: u64,
    vault_authority_bump: u8,
) -> Result<()> {
    let cpi_program = ctx.accounts.option_trading_program.clone();
    let cpi_accounts = MintOptionV2 {
        user_authority: ctx.accounts.vault_authority.to_account_info(),
        underlying_asset_mint: ctx.accounts.underlying_asset_mint.to_account_info(),
        underlying_asset_pool: ctx.accounts.underlying_asset_pool.to_account_info(),
        underlying_asset_src: ctx.accounts.vault.to_account_info(),
        option_mint: ctx.accounts.option_mint.to_account_info(),
        minted_option_dest: ctx.accounts.minted_option_dest.to_account_info(),
        writer_token_mint: ctx.accounts.writer_token_mint.to_account_info(),
        minted_writer_token_dest: ctx.accounts.minted_writer_token_dest.to_account_info(),
        option_market: ctx.accounts.option_market.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };
    let key = ctx.accounts.underlying_asset_mint.key();

    let seeds = &[key.as_ref(), b"vaultAuthority", &[vault_authority_bump]];
    let signer = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

    options_trading::cpi::mint_option_v2(cpi_ctx, size)
}
