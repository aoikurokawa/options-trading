use anchor_lang::{prelude::*, Key};
use crate::errors;
use crate::state::option_market::{OptionMarket};
use anchor_spl::token::{Mint, Token, TokenAccount};

pub fn initialize_market(ctx: Context<InitializeMarket>, underlying_amount_per_contract: u64, quote_amount_per_contract: u64, expiration_unix_timestamp: i64, bump_seed: u8) -> Result<()> {



    Ok(())
}

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
    pub fee_owner: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
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