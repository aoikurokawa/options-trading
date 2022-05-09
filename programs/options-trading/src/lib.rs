pub mod errors;
pub mod fees;
pub mod instructions;
pub mod serum_proxy;
pub mod state;

use crate::state::option_market::OptionMarket;
use anchor_lang::{prelude::*, AccountsExit, Key};
use anchor_spl::dex::{
    initialize_market as init_serum_market_instruction, InitializeMarket as SerumInitMarket,
};
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};
use psyfi_serum_dex_permissioned::{MarketProxy, OpenOrdersPda, ReferralFees};
use solana_program::{
    program::invoke, program_error::ProgramError, program_pack::Pack, system_instruction,
    system_program,
};
use spl_token::state::Account as SPLTokenAccount;

declare_id!("6xUQFHLbbfhayBwLBNSMfZfCmHNJJujWsS88qCtGfWdn");

#[program]
pub mod options_trading {
    use super::*;

    #[access_control(InitializeMarket::accounts(&ctx))]
    pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
        Ok(())
    }
}

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

#[derive(Accounts)]
pub struct MintOptionV2<'info> {
    // The user authority must be the authority that has ownership of the `underlying_asset_src`
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

    pub token_program: Program<'info, Token>,
}

impl<'info> MintOptionV2<'info> {
    fn accounts(ctx: &Context<MintOptionV2<'info>>) -> Result<()> {
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
            return Err(errors::ErrorCode::WriterTokenMintDoesNotMatchMarket.into());
        }

        Ok(())
    }

    fn unexpired_market(ctx: &Context<MintOptionV2<'info>>) -> Result<()> {
        // Validate the market is not expired
        if ctx.accounts.option_market.expiration_unix_timestamp < Clock::get()?.unix_timestamp {
            return Err(errors::ErrorCode::OptionMarketExpiredCantMint.into());
        }
        Ok(())
    }
}

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
