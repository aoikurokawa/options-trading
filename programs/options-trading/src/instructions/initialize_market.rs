use solana_program::program_pack::Pack;
use anchor_lang::{prelude::*, Key};
use crate::{errors, fees};
use crate::state::option_market::{OptionMarket};
use anchor_spl::token::{Mint, Token, TokenAccount};
use spl_token::state::Account as SPLTokenAccount;

#[derive(Accounts)]
#[instruction(
    underlying_amount_per_contract: u64,
    quote_amount_per_contract: u64,
    expiration_unix_timestamp: i64,
    bump_seed: u8,
)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub underlying_asset_mint: Box<Account<'info, Mint>>,
    pub quote_asset_mint: Box<Account<'info, Mint>>,
    #[account(init, 
        seeds = [&option_market.key().to_bytes()[..], b"optionToken"],
        bump,
        payer = authority,
        mint::decimals = 0,
        mint::authority = option_market
    )]
    pub option_mint: Box<Account<'info, Mint>>,
    #[account(init, 
        seeds = [&option_market.key().to_bytes()[..], b"writerToken"],
        bump,
        payer = authority,
        mint::decimals = 0,
        mint::authority = option_market
    )]
    pub writer_token_mint: Box<Account<'info, Mint>>,
    #[account(init,
        seeds = [&option_market.key().to_bytes()[..], b"quoteAssetPool"],
        bump,
        payer = authority,
        token::mint = quote_asset_mint,
        token::authority = option_market,
    )]
    pub quote_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(init, 
        seeds = [&option_market.key().to_bytes()[..], b"underlyingAssetPool"],
        bump,
        payer = authority,
        token::mint = underlying_asset_mint,
        token::authority = option_market,
    )]
    pub underlying_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(init,
        seeds = [
            underlying_asset_mint.key().as_ref(),
            quote_asset_mint.key().as_ref(),
            &underlying_amount_per_contract.to_le_bytes(),
            &quote_amount_per_contract.to_le_bytes(),
            &expiration_unix_timestamp.to_le_bytes()
        ],
        bump, 
        payer = authority,
        space = 8 + std::mem::size_of::<OptionMarket>() + 300
    )]
    pub option_market: Box<Account<'info, OptionMarket>>,
    /// The V1 fee owner
    /// CHECK: Handled
    pub fee_owner: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Unnecessary account, but left for backwards compatibility
    pub associated_token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> InitializeMarket<'info> {
    pub fn accounts(ctx: &Context<InitializeMarket<'info>>) -> Result<()> {
        if ctx.accounts.option_mint.mint_authority.unwrap() != *ctx.accounts.option_market.to_account_info().key {
            return Err(errors::ErrorCode::OptionMarketMustBeMintAuthority.into());
        }

        if ctx.accounts.writer_token_mint.mint_authority.unwrap() != *ctx.accounts.option_market.to_account_info().key {
            return Err(errors::ErrorCode::OptionMarketMustBeMintAuthority.into());
        }

        if ctx.accounts.underlying_asset_pool.owner != *ctx.accounts.option_market.to_account_info().key {
            return Err(errors::ErrorCode::OptionMarketMustOwnQuoteAssetPool.into());
        }

        if ctx.accounts.underlying_asset_mint.to_account_info().key == ctx.accounts.quote_asset_mint.to_account_info().key {
            return Err(errors::ErrorCode::QuoteAndUnderlyingAssetMustDiffer.into());
        }

        Ok(())
    }
}

pub fn handler(
    ctx: Context<InitializeMarket>, 
    underlying_amount_per_contract: u64, 
    quote_amount_per_contract: u64, 
    expiration_unix_timestamp: i64, 
    bump_seed: u8,
) -> Result<()> {
    if expiration_unix_timestamp < ctx.accounts.clock.unix_timestamp {
        return Err(errors::ErrorCode::ExpirationIsInThePast.into());
    }

    // check that underlying_amount_per_contract and quote_amount_per_contract are not 0
    if underlying_amount_per_contract <= 0 || quote_amount_per_contract <= 0 {
        return Err(errors::ErrorCode::QuoteOrUnderlyingAmountCannotBe0.into());
    }

    let fee_accounts = validate_fee_accounts(
        &ctx.remaining_accounts,
        &ctx.accounts.underlying_asset_mint.key(),
        &ctx.accounts.quote_asset_mint.key(),
        underlying_amount_per_contract,
        quote_amount_per_contract,
    )?;

    // write the data to the OptionMarket account
    let option_market = &mut ctx.accounts.option_market;
    option_market.option_mint = *ctx.accounts.writer_token_mint.to_account_info().key;
    option_market.writer_token_mint = *ctx.accounts.writer_token_mint.to_account_info().key;
    option_market.underlying_asset_mint = *ctx.accounts.underlying_asset_mint.to_account_info().key;
    option_market.quote_asset_mint = *ctx.accounts.quote_asset_mint.to_account_info().key;
    option_market.underlying_amount_per_contract = underlying_amount_per_contract;
    option_market.quote_amount_per_contract = quote_amount_per_contract;
    option_market.expiration_unix_timestamp = expiration_unix_timestamp;
    option_market.underlying_asset_pool = *ctx.accounts.underlying_asset_pool.to_account_info().key;
    option_market.quote_asset_pool = *ctx.accounts.quote_asset_pool.to_account_info().key;
    option_market.mint_fee_account = fee_accounts.mint_fee_key;
    option_market.exercise_fee_account = fee_accounts.exercise_fee_key;
    option_market.expired = false;

    match ctx.bumps.get("option_market") {
        Some(bump) => {
            option_market.bump_seed = *bump;
        }
        None => {
            msg!("Wrong bump key. Available keys are {:?}", ctx.bumps.keys());
            panic!("Wrong bump key")
        }
    }

    Ok(())
}

struct FeeAccounts {
    mint_fee_key: Pubkey,
    exercise_fee_key: Pubkey,
}

fn validate_fee_accounts<'info> (
    remaining_accounts: &[AccountInfo],
    underlying_asset_mint: &Pubkey,
    quote_asset_mint: &Pubkey,
    underlying_amount_per_contract: u64,
    quote_amount_per_contract: u64,
) -> Result<FeeAccounts> {
    let account_info_iter = &mut remaining_accounts.iter();
    let mut fee_accounts = FeeAccounts {
        mint_fee_key: fees::fee_owner_key::ID,
        exercise_fee_key: fees::fee_owner_key::ID,
    };

    // if the mint fee account is required, check that it exists and has the proper owner
    if fees::fee_amount(underlying_amount_per_contract) > 0 {
        let mint_fee_recipient = next_account_info(account_info_iter)?;
        if mint_fee_recipient.owner != &spl_token::ID {
            return Err(errors::ErrorCode::ExpectedSPLTokenProgramId.into());
        }
        let mint_fee_account = SPLTokenAccount::unpack_from_slice(&mint_fee_recipient.try_borrow_data()?)?;
        if mint_fee_account.owner != fees::fee_owner_key::ID {
            return Err(errors::ErrorCode::MintFeeMustBeOwnedByFeeOwner.into());
        }
        // Check that the mint fee recipient account's mint is also the underlying mint
        if mint_fee_account.mint != *underlying_asset_mint {
            return Err(errors::ErrorCode::MintFeeTokenMustMatchUnderlyingAsset.into());
        }

        fee_accounts.mint_fee_key = *mint_fee_recipient.key;
    }

    // if the exercise fee account is required, check that it exists and has the proper owner
    if fees::fee_amount(quote_amount_per_contract) > 0 {
        let exercise_fee_recipient = next_account_info(account_info_iter)?;
        if exercise_fee_recipient.owner != &spl_token::ID {
            return Err(errors::ErrorCode::ExpectedSPLTokenProgramId.into());
        }
        let exercise_fee_account = SPLTokenAccount::unpack_from_slice(&exercise_fee_recipient.try_borrow_data()?)?;
        if exercise_fee_account.owner != fees::fee_owner_key::ID {
            return Err(errors::ErrorCode::ExerciseFeeMustBeOwnedByFeeOwner.into());
        }
        // check that the exercise fee recipient account's mint is also the quote mint
        if exercise_fee_account.mint != *quote_asset_mint {
            return Err(errors::ErrorCode::ExerciseFeeTokenMustMatchQuoteAsset.into());
        }

        fee_accounts.exercise_fee_key = *exercise_fee_recipient.key;
    }

    Ok(fee_accounts)

}


