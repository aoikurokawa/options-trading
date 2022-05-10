use crate::errors;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::*;
use anchor_spl::dex::{initialize_market as init_serum_market_instruction, InitializeMarket as SerumInitMarket};
// use solana_program::{program_error::ProgramError, system_program};

#[derive(Accounts)]
#[instruction(market_space: u64, vault_signer_nonce: u64, coin_lot_size: u64, pc_lot_size: u64, pc_dust_threshold: u64)]
pub struct InitSerumMarket<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,

    #[account(mut)]
    pub option_market: Box<Account<'info, OptionMarket>>,

    #[account(
        init, 
        seeds = [&option_market.key().to_bytes()[..], &pc_mint.key().to_bytes()[..], b"serumMarket"],
        bump,
        space = market_space as usize,
        payer = user_authority,
        owner = *dex_program.key
    )]
    pub serum_market: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,
    pub rent: Sysvar<'info, Rent>,
    pub pc_mint: Box<Account<'info, Mint>>,
    pub option_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
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

    #[account(
        init,
        seeds = [&option_market.key().to_bytes()[..], &pc_mint.key().to_bytes()[..], b"coinVault"],
        bump,
        payer = user_authority,
        token::mint = option_mint,
        token::authority = vault_signer,
    )]
    pub coin_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        seeds = [&option_market.key().to_bytes()[..], &pc_mint.key().to_bytes()[..], b"pcVault"],
        bump,
        payer = user_authority,
        token::mint = option_mint,
        token::authority = vault_signer,
    )]
    pub pc_vault: Box<Account<'info, TokenAccount>>,
    pub vault_signer: AccountInfo<'info>,
    pub market_authority: AccountInfo<'info>,
}

impl<'info> InitSerumMarket<'info> {
    pub fn accounts(ctx: &Context<InitSerumMarket>) -> Result<()> {

        if ctx.accounts.option_mint.key() != ctx.accounts.option_market.option_mint {
            return Err(errors::ErrorCode::CoinMintIsNotOptionMint.into());
        }

        Ok(())
    }
}

pub fn init_serum_market(
    ctx: Context<InitSerumMarket>, 
    _market_space: u64, 
    vault_signer_nonce: u64, 
    coin_lot_size: u64, 
    pc_lot_size: u64, 
    pc_dust_threshold: u64
) -> Result<()> {

    let init_market_ctx = SerumInitMarket {
        market: ctx.accounts.serum_market.to_account_info(),
        coin_mint: ctx.accounts.option_mint.to_account_info(),
        pc_mint: ctx.accounts.pc_mint.to_account_info(),
        coin_vault: ctx.accounts.coin_vault.to_account_info(),
        pc_vault: ctx.accounts.pc_vault.to_account_info(),
        bids: ctx.accounts.bids.to_account_info(),
        asks: ctx.accounts.asks.to_account_info(),
        req_q: ctx.accounts.request_queue.to_account_info(),
        event_q: ctx.accounts.event_queue.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let mut cpi_ctx = CpiContext::new(ctx.accounts.dex_program.to_account_info(), init_market_ctx);
    cpi_ctx
        .remaining_accounts
        .push(ctx.accounts.market_authority.to_account_info());
    cpi_ctx
        .remaining_accounts
        .push(ctx.accounts.market_authority.to_account_info());
    init_serum_market_instruction(
        cpi_ctx,
        coin_lot_size,
        pc_lot_size,
        vault_signer_nonce,
        pc_dust_threshold,
    )
}
