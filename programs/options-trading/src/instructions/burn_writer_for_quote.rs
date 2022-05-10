use crate::errors;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::*;
// use solana_program::{program_error::ProgramError, system_program};

#[derive(Accounts)]
pub struct BurnWriterForQuote<'info> {
    pub user_authority: Signer<'info>,
    pub option_market: Box<Account<'info, OptionMarket>>,
    #[account(mut)]
    pub writer_token_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub writer_token_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub writer_quote_dest: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> BurnWriterForQuote<'info> {
    pub fn accounts(ctx: &Context<BurnWriterForQuote>) -> Result<()> {
        // Validate the Quote asset pool matches the OptionMarket
        if ctx.accounts.quote_asset_pool.key() != ctx.accounts.option_market.quote_asset_pool {
            return Err(errors::ErrorCode::QuotePoolAccountDoesNotMatchMarket.into());
        }

        // Validate WriteToken mint matches the OptionMarket
        if ctx.accounts.writer_token_mint.key() != ctx.accounts.option_market.writer_token_mint {
            return Err(errors::ErrorCode::WriterTokenMintDoesNotMatchMarket.into());
        }

        Ok(())
    }

    // Validate there is enough quote assets in the pool
    pub fn quotes_in_pool(ctx: &Context<BurnWriterForQuote>, size: u64) -> Result<()> {
        if ctx.accounts.quote_asset_pool.amount
            < size
                .checked_mul(ctx.accounts.option_market.quote_amount_per_contract)
                .unwrap()
        {
            return Err(errors::ErrorCode::NotEnoughQuoteAssetsInPool.into());
        }

        Ok(())
    }
}

pub fn handler(ctx: Context<BurnWriterForQuote>, size: u64) -> Result<()> {
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

    // Transfer the quote assets to the writer's account
    let cpi_accounts = Transfer {
        from: ctx.accounts.quote_asset_pool.to_account_info(),
        to: ctx.accounts.writer_quote_dest.to_account_info(),
        authority: ctx.accounts.option_market.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx =
        CpiContext::new_with_signer(cpi_token_program.to_account_info(), cpi_accounts, signer);
    let quote_transfer_amount = option_market
        .quote_amount_per_contract
        .checked_mul(size)
        .unwrap();
    transfer(cpi_ctx, quote_transfer_amount)?;

    Ok(())
}
