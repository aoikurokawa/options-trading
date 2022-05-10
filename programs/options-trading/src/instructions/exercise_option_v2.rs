use crate::errors;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::*;
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
    pub fn accounts(ctx: &Context<ExerciseOptionV2>) -> Result<()> {
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

    pub fn unexpired_market(ctx: &Context<ExerciseOptionV2>) -> Result<()> {
        // Validate the market is not expired
        if ctx.accounts.option_market.expiration_unix_timestamp < Clock::get()?.unix_timestamp {
            return Err(errors::ErrorCode::OptionMarketExpiredCantExercise.into());
        }

        Ok(())
    }
}

pub fn exercise_option_v2<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExerciseOptionV2<'info>>,
    size: u64,
) -> Result<()> {
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
    // Burn the size of option tokens
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.quote_asset_src.to_account_info().clone(),
        Burn {
            mint: ctx.accounts.option_mint.to_account_info(),
            from: ctx.accounts.exerciser_option_token_src.to_account_info(),
            authority: ctx.accounts.option_authority.to_account_info(),
        },
        signer,
    );
    burn(cpi_ctx, size)?;

    // Transfer the quote assets to the pool
    let cpi_accounts = Transfer {
        from: ctx.accounts.quote_asset_src.to_account_info(),
        to: ctx.accounts.quote_asset_pool.to_account_info(),
        authority: ctx.accounts.user_authority.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_token_program.to_account_info(), cpi_accounts);
    let quote_transfer_amount = option_market
        .quote_amount_per_contract
        .checked_mul(size)
        .unwrap();
    transfer(cpi_ctx, quote_transfer_amount)?;

    // Transfer the underlying assets from the pool to the exerciser
    let cpi_accounts = Transfer {
        from: ctx.accounts.underlying_asset_pool.to_account_info(),
        to: ctx.accounts.underlying_asset_dest.to_account_info(),
        authority: ctx.accounts.option_market.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_token_program.to_account_info(), cpi_accounts);
    let quote_transfer_amount = option_market
        .quote_amount_per_contract
        .checked_mul(size)
        .unwrap();
    transfer(cpi_ctx, quote_transfer_amount)?;

    // Transfer the underlying assets from the pool to the exerciser
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
