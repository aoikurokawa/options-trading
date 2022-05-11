use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
pub struct InitOptionMarket<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub options_trading_program: AccountInfo<'info>,
    pub underlying_asset_mint: Box<Account<'info, Mint>>,
    pub quote_asset_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub option_mint: AccountInfo<'info>,
    #[account(mut)]
    pub writer_token_mint: AccountInfo<'info>,
    #[account(mut)]
    pub quote_asset_pool: AccountInfo<'info>,
    #[account(mut)]
    pub underlying_asset_pool: AccountInfo<'info>,
    #[account(mut)]
    pub option_market: AccountInfo<'info>,
    pub fee_owner: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}


