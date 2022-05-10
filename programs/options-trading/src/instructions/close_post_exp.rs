use crate::errors;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::*;
// use solana_program::{program_error::ProgramError, system_program};

#[derive(Accounts)]
pub struct ClosePostExp<'info> {
    pub user_authority: Signer<'info>,
    pub option_market: Box<Account<'info, OptionMarket>>,
    #[account(mut)]
    pub writer_token_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub writer_token_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub underlying_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub underlying_asset_dest: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> ClosePostExp<'info> {
    pub fn accounts(ctx: &Context<ClosePostExp>) -> Result<()> {
        // Validate the underlying asset pool is the same as on the OptionMarket
        if *ctx.accounts.underlying_asset_pool.to_account_info().key
            != ctx.accounts.option_market.underlying_asset_pool
        {
            return Err(errors::ErrorCode::UnderlyingPoolAccountDoesNotMatchMarket.into());
        }

        // Validate the writer mint is the same as on the OptionMarket
        if *ctx.accounts.writer_token_mint.to_account_info().key
            != ctx.accounts.option_market.writer_token_mint
        {
            return Err(errors::ErrorCode::WriterTokenMintDoesNotMatchMarket.into());
        }

        // Validate the underlying destination has the same mint as the option underlying
        if ctx.accounts.underlying_asset_dest.mint
            != ctx.accounts.option_market.underlying_asset_mint
        {
            return Err(errors::ErrorCode::UnderlyingDestMintDoesNotMatchUnderlyingAsset.into());
        }

        Ok(())
    }

    pub fn expired_market(ctx: &Context<ClosePostExp>) -> Result<()> {
        // Validate the market is expired
        if ctx.accounts.option_market.expiration_unix_timestamp >= ctx.accounts.clock.unix_timestamp
        {
            return Err(errors::ErrorCode::OptionMarketNotExpirationCantClose.into());
        }

        Ok(())
    }
}

pub fn handler(ctx: Context<ClosePostExp>, size: u64) -> Result<()> {
    let option_market = &ctx.accounts.option_market;
    let seeds = &[
        option_market.underlying_asset_mint.as_ref(),
        option_market.quote_asset_mint.as_ref(),
        &option_market.underlying_amount_per_contract.to_le_bytes(),
        &option_market.quote_amount_per_contract.to_le_bytes(),
        &option_market.expiration_unix_timestamp.to_le_bytes(),
        &[option_market.bump_seed],
    ];
    let signer = &[&seeds[..]];

    // Burn the size of WriterTokens
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info().clone(),
        Burn {
            mint: ctx.accounts.writer_token_mint.to_account_info(),
            from: ctx.accounts.writer_token_src.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        },
        signer,
    );
    burn(cpi_ctx, size)?;

    // Transfer the underlying from the pool to the user
    let cpi_accounts = Transfer {
        from: ctx.accounts.underlying_asset_pool.to_account_info(),
        to: ctx.accounts.underlying_asset_dest.to_account_info(),
        authority: ctx.accounts.option_market.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx =
        CpiContext::new_with_signer(cpi_token_program.to_account_info(), cpi_accounts, signer);
    let underlying_transfer_amount = option_market
        .underlying_amount_per_contract
        .checked_mul(size)
        .unwrap();
    transfer(cpi_ctx, underlying_transfer_amount)?;

    Ok(())
}
