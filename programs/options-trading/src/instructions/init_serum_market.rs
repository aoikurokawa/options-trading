use crate::errors;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
// use solana_program::{program_error::ProgramError, system_program};

#[derive(Accounts)]
#[instruction(market_space: u64, vault_signer_nonce: u64, coin_lost_size: u64, pc_lot_size: u64, pc_dust_threshold: u64)]
pub struct InitSerumMarket<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // General market accounts
    #[account(mut)]
    pub option_market: Box<Account<'info, OptionMarket>>,
    /// CHECK: Handled
    #[account(init, 
        seeds = [&option_market.key().to_bytes()[..], &pc_mint.key().to_bytes()[..],b"serumMarket"],
        bump,
        space = market_space as usize,
        payer = user_authority,
        owner = *dex_program.key
    )]
    pub serum_market: AccountInfo<'info>,
    // system accounts
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,
    pub rent: Sysvar<'info, Rent>,
    pub pc_mint: Box<Account<'info, Mint>>,
    pub option_mint: Box<Account<'info, Mint>>,
    // INIT SERUM MARKET ACCOUNTS
    /// CHECK: Handled by Serum
    #[account(init,
        seeds = [&option_market.key().to_bytes()[..], &pc_mint.key().to_bytes()[..], b"requestQueue"],
        bump,
        space = 5120 + 12,
        payer = user_authority,
        owner = *dex_program.key
    )]
    request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    #[account(init, 
        seeds = [&option_market.key().to_bytes()[..], &pc_mint.key().to_bytes()[..], b"coinVault"],
        bump,
        payer = user_authority,
        token::mint = option_mint,
        token::authority = vault_signer,
    )]
    pub coin_vault: Box<Account<'info, TokenAccount>>,
    #[account(init, 
        seeds = [&option_market.key().to_bytes()[..], &pc_mint.key().to_bytes()[..], b"pcVault"],
        bump,
        payer = user_authority,
        token::mint = pc_mint,
        token::authority = vault_signer,
    )]
    pub pc_vault: Box<Account<'info, TokenAccount>>,
    pub vault_signer: AccountInfo<'info>,
    pub market_authority: AccountInfo<'info>
}

impl<'info> InitSerumMarket<'info> {
    // Validate the coin_mint is the same as the OptionMarket.option_mint
    pub fn accounts(ctx: &Context<InitSerumMarket>) -> Result<()> {
        if ctx.accounts.option_mint.key() != ctx.accounts.option_market.option_mint {
            return Err(errors::ErrorCode::CoinMintIsNotOptionMint.into());
        }
        Ok(())
    }
}
