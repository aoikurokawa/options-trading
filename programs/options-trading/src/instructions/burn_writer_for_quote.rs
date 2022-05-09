use crate::errors;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
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
    fn accounts(ctx: &Context<BurnWriterForQuote>) -> Result<()> {
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
    fn quotes_in_pool(ctx: &Context<BurnWriterForQuote>, size: u64) -> Result<()> {
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
