use crate::errors;
use crate::fees;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::{program_error::ProgramError, system_program};

#[derive(Accounts)]
pub struct ExerciseOption<'info> {
    /// The user_authority must be the authority that has ownership of the `quote_asset_src` account
    pub user_authority: Signer<'info>,
    /// The owner of the `exercise_option_token_src` account
    /// CHECK: Handled
    #[account(mut, signer)]
    pub option_authority: AccountInfo<'info>,
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
    /// CHECK: Handled
    #[account(mut)]
    pub fee_owner: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> ExerciseOption<'info> {
    fn accounts(ctx: &Context<ExerciseOption>) -> Result<()> {
        // Validate the quote asset pool is the same as on the OptionMarket
        if *ctx.accounts.quote_asset_pool.to_account_info().key
            != ctx.accounts.option_market.quote_asset_pool
        {
            return Err(errors::ErrorCode::QuotePoolAccountDoesNotMatchMarket.into());
        }

        // Validate the underlying asset pool is the same as on the OptionMarket
        if *ctx.accounts.underlying_asset_pool.to_account_info().key
            != ctx.accounts.option_market.underlying_asset_pool
        {
            return Err(errors::ErrorCode::UnderlyingPoolAccountDoesNotMatchMarket.into());
        }

        // Validate the option mint is the same as on the OptionMarket
        if *ctx.accounts.option_mint.to_account_info().key != ctx.accounts.option_market.option_mint
        {
            return Err(errors::ErrorCode::OptionTokenMintDoesNotMatchMarket.into());
        }

        // Validate the underlying destination has the same mint as the pool
        if ctx.accounts.underlying_asset_dest.mint
            != ctx.accounts.option_market.underlying_asset_mint
        {
            return Err(errors::ErrorCode::UnderlyingDestMintDoesNotMatchUnderlyingAsset.into());
        }

        // Validate the fee owner is correct
        if *ctx.accounts.fee_owner.key != fees::fee_owner_key::ID {
            return Err(errors::ErrorCode::FeeOwnerDoesNotMatchProgram.into());
        }

        // Validate the system program account passed in is correct
        if !system_program::check_id(ctx.accounts.system_program.key) {
            return Err(ProgramError::InvalidAccountData.into());
        }

        Ok(())
    }
}
