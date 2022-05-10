use crate::errors;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::*;
// use solana_program::{program_error::ProgramError, system_program};

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
    pub fn accounts(ctx: &Context<MintOptionV2<'info>>) -> Result<()> {
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

    pub fn unexpired_market(ctx: &Context<MintOptionV2<'info>>) -> Result<()> {
        // Validate the market is not expired
        if ctx.accounts.option_market.expiration_unix_timestamp < Clock::get()?.unix_timestamp {
            return Err(errors::ErrorCode::OptionMarketExpiredCantMint.into());
        }
        Ok(())
    }
}

pub fn helper<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, MintOptionV2<'info>>,
    size: u64,
) -> Result<()> {
    let option_market = &ctx.accounts.option_market;

    // Transfer the underlying assets to the underlying assets pool
    let cpi_accounts = Transfer {
        from: ctx.accounts.underlying_asset_src.to_account_info(),
        to: ctx.accounts.underlying_asset_pool.to_account_info(),
        authority: ctx.accounts.user_authority.to_account_info().clone(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_token_program.to_account_info(), cpi_accounts);
    let underlying_transfer_amount = option_market
        .underlying_amount_per_contract
        .checked_mul(size)
        .unwrap();
    transfer(cpi_ctx, underlying_transfer_amount)?;

    let seeds = &[
        option_market.underlying_asset_mint.as_ref(),
        option_market.quote_asset_mint.as_ref(),
        &option_market.underlying_amount_per_contract.to_le_bytes(),
        &option_market.quote_amount_per_contract.to_le_bytes(),
        &option_market.expiration_unix_timestamp.to_le_bytes(),
        &[option_market.bump_seed],
    ];
    let signer = &[&seeds[..]];

    // Mint a new OptionToken(s)
    let cpi_accounts = MintTo {
        mint: ctx.accounts.option_mint.to_account_info(),
        to: ctx.accounts.minted_option_dest.to_account_info(),
        authority: ctx.accounts.option_market.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx =
        CpiContext::new_with_signer(cpi_token_program.to_account_info(), cpi_accounts, signer);
    mint_to(cpi_ctx, size)?;

    // Mint a new WriterToken(s)
    let cpi_accounts = MintTo {
        mint: ctx.accounts.writer_token_mint.to_account_info(),
        to: ctx.accounts.minted_option_dest.to_account_info(),
        authority: ctx.accounts.option_market.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx =
        CpiContext::new_with_signer(cpi_token_program.to_account_info(), cpi_accounts, signer);
    mint_to(cpi_ctx, size)?;

    Ok(())
}
