use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::{program_error::ProgramError, system_program};

use crate::errors;
use crate::fees;

#[derive(Accounts)]
pub struct MintOption<'info> {
    /// The user authority must be the authority that has ownership of the `underlying_asset_src`
    #[account(mut)]
    pub user_authority: Signer<'info>,
    /// CHECK: Handled
    pub underlying_asset_mint: AccountInfo<'info>,
    #[account(mut)]
    pub underlying_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub underlying_asset_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub minted_option_dest: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub writer_token_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub minted_writer_token_dest: Box<Account<'info, TokenAccount>>,
    pub option_market: Box<Account<'info, OptionMarket>>,
    /// CHECK: Handled
    #[account(mut)]
    pub fee_owner: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK Unnecessary account, but left for backwards compatibility
    pub associated_token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintOption<'info> {
    fn accounts(ctx: &Context<MintOption<'info>>) -> Result<()> {
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

        // Validate the writer token mint is the same as on the OptionMarket
        if *ctx.accounts.writer_token_mint.to_account_info().key
            != ctx.accounts.option_market.writer_token_mint
        {
            return Err(errors::ErrorCode::FeeOwnerDoesNotMatchProgram.into());
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

    fn unexpired_market(ctx: &Context<MintOption<'info>>) -> Result<()> {
        // Validate the market is not expired
        if ctx.accounts.option_market.expiration_unix_timestamp < ctx.accounts.clock.unix_timestamp
        {
            return Err(errors::ErrorCode::OptionMarketExpiredCantMint.into());
        }

        Ok(())
    }
}
