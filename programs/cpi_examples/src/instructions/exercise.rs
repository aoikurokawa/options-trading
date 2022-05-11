use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use options_trading::cpi::accounts::ExerciseOption;
use options_trading::state::OptionMarket;

#[derive(Accounts)]
pub struct Exercise<'info> {
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    pub option_trading_program: AccountInfo<'info>,
    #[account(mut)]
    pub vault_authority: AccountInfo<'info>,
    option_market: Box<Account<'info, OptionMarket>>,
    #[account(mut)]
    option_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    exerciser_option_token_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    underlying_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    underlying_asset_dest: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    quote_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    quote_asset_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    fee_owner: AccountInfo<'info>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    clock: Sysvar<'info, Clock>,
}

pub fn exercise<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, Exercise<'info>>,
    vault_authority_bump: u8,
) -> Result<()> {
    msg!("before cpi");
    let cpi_program = ctx.accounts.option_trading_program.clone();
    let cpi_accounts = ExerciseOption {
        user_authority: ctx.accounts.authority.to_account_info(),
        option_authority: ctx.accounts.vault_authority.to_account_info(),
        option_market: ctx.accounts.option_market.to_account_info(),
        option_mint: ctx.accounts.option_mint.to_account_info(),
        exerciser_option_token_src: ctx.accounts.exerciser_option_token_src.to_account_info(),
        underlying_asset_pool: ctx.accounts.underlying_asset_pool.to_account_info(),
        underlying_asset_dest: ctx.accounts.underlying_asset_dest.to_account_info(),
        quote_asset_pool: ctx.accounts.quote_asset_pool.to_account_info(),
        quote_asset_src: ctx.accounts.quote_asset_src.to_account_info(),
        fee_owner: ctx.accounts.fee_owner.clone(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        clock: ctx.accounts.clock.to_account_info(),
    };
    let key = ctx.accounts.option_market.key();

    let seeds = &[key.as_ref(), b"vaultAuthority", &[vault_authority_bump]];
    let signer = &[&seeds[..]];
    let mut cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    cpi_ctx.remaining_accounts = ctx.remaining_accounts.to_vec();
    options_trading::cpi::exercise_option(cpi_ctx, ctx.accounts.exerciser_option_token_src.amount)
}
