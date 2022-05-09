use crate::errors;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
// use solana_program::{program_error::ProgramError, system_program};

#[derive(Accounts)]
pub struct ExerciseOptionV2<'info> {
    /// The user_authority must be the authority that has ownership of the `quote_asset_src` account
    pub user_authority: Signer<'info>,
    /// The owner of the `exerciser_option_token_src` account
    pub option_authority: Signer<'info>,
    pub option_market: Box<Account<'info, OptionMarket>>,
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub exerciser_option_token_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub underlying_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub underlying_asset_dest: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_asset_src: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> ExerciseOptionV2<'info> {
    fn accounts(ctx: &Context<ExerciseOptionV2>) -> Result<()> {
        // Validate the quote asset pool is the same as on the OptionMarket
        if ctx.accounts.quote_asset_pool.key() != ctx.accounts.option_market.quote_asset_pool {
            return Err(errors::ErrorCode::QuotePoolAccountDoesNotMatchMarket.into());
        }

        // Validate the underlying asset pool is the same as on the OptionMarket
        if ctx.accounts.underlying_asset_pool.key()
            != ctx.accounts.option_market.underlying_asset_pool
        {
            return Err(errors::ErrorCode::UnderlyingPoolAccountDoesNotMatchMarket.into());
        }

        // Validate the option mint is the same as on the OptionMarket
        if ctx.accounts.option_mint.key() != ctx.accounts.option_market.option_mint {
            return Err(errors::ErrorCode::OptionTokenMintDoesNotMatchMarket.into());
        }

        // Validate the underlying destination has the same mint as the pool
        if ctx.accounts.underlying_asset_dest.mint
            != ctx.accounts.option_market.underlying_asset_mint
        {
            return Err(errors::ErrorCode::UnderlyingDestMintDoesNotMatchUnderlyingAsset.into());
        }

        Ok(())
    }

    fn unexpired_market(ctx: &Context<ExerciseOptionV2>) -> Result<()> {
        // Validate the market is not expired
        if ctx.accounts.option_market.expiration_unix_timestamp < Clock::get()?.unix_timestamp {
            return Err(errors::ErrorCode::OptionMarketExpiredCantExercise.into());
        }

        Ok(())
    }
}
