use anchor_lang::InstructionData;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::errors as CpiExampleErrors;

#[derive(Accounts)]
pub struct InitOptionMarket<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: TODO
    pub options_trading_program: AccountInfo<'info>,
    pub underlying_asset_mint: Box<Account<'info, Mint>>,
    pub quote_asset_mint: Box<Account<'info, Mint>>,

    /// CHECK: TODO
    #[account(mut)]
    pub option_mint: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub writer_token_mint: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub quote_asset_pool: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub underlying_asset_pool: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub option_market: AccountInfo<'info>,
    /// CHECK: TODO
    pub fee_owner: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK: TODO
    pub associated_token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitOptionMarket<'info>>,
    underlying_amount_per_contract: u64,
    quote_amount_per_contract: u64,
    expiration_unix_timestamp: i64,
    bump_seed: u8,
) -> Result<()> {
    let cpi_program = ctx.accounts.options_trading_program.clone();
    let init_market_args = options_trading::instruction::InitializeMarket {
        underlying_amount_per_contract,
        quote_amount_per_contract,
        expiration_unix_timestamp,
        bump_seed,
    };
    let mut cpi_accounts = vec![
        ctx.accounts.user.to_account_metas(Some(true))[0].clone(),
        ctx.accounts
            .underlying_asset_mint
            .to_account_metas(Some(false))[0]
            .clone(),
        ctx.accounts.quote_asset_mint.to_account_metas(Some(false))[0].clone(),
        ctx.accounts.option_mint.to_account_metas(Some(false))[0].clone(),
        ctx.accounts.writer_token_mint.to_account_metas(Some(false))[0].clone(),
        ctx.accounts.quote_asset_pool.to_account_metas(Some(false))[0].clone(),
        ctx.accounts
            .underlying_asset_pool
            .to_account_metas(Some(false))[0]
            .clone(),
        ctx.accounts.option_market.to_account_metas(Some(false))[0].clone(),
        ctx.accounts.fee_owner.to_account_metas(Some(false))[0].clone(),
        ctx.accounts.token_program.to_account_metas(Some(false))[0].clone(),
        ctx.accounts
            .associated_token_program
            .to_account_metas(Some(false))[0]
            .clone(),
        ctx.accounts.rent.to_account_metas(Some(false))[0].clone(),
        ctx.accounts.system_program.to_account_metas(Some(false))[0].clone(),
        ctx.accounts.clock.to_account_metas(Some(false))[0].clone(),
    ];
    let mut account_infos = vec![
        ctx.accounts.user.to_account_info().clone(),
        ctx.accounts.underlying_asset_mint.to_account_info().clone(),
        ctx.accounts.quote_asset_mint.to_account_info().clone(),
        ctx.accounts.option_mint.to_account_info().clone(),
        ctx.accounts.writer_token_mint.to_account_info().clone(),
        ctx.accounts.quote_asset_pool.to_account_info().clone(),
        ctx.accounts.underlying_asset_pool.to_account_info().clone(),
        ctx.accounts.option_market.to_account_info().clone(),
        ctx.accounts.fee_owner.to_account_info().clone(),
        ctx.accounts.token_program.to_account_info().clone(),
        ctx.accounts
            .associated_token_program
            .to_account_info()
            .clone(),
        ctx.accounts.rent.to_account_info().clone(),
        ctx.accounts.system_program.to_account_info().clone(),
        ctx.accounts.clock.to_account_info().clone(),
    ];
    for remaining_account in ctx.remaining_accounts {
        cpi_accounts.push(remaining_account.to_account_metas(Some(false))[0].clone());
        account_infos.push(remaining_account.clone());
    }

    let ix = solana_program::instruction::Instruction {
        program_id: *cpi_program.key,
        accounts: cpi_accounts,
        data: init_market_args.data(),
    };

    anchor_lang::solana_program::program::invoke(&ix, &account_infos)
        .map_err(|_x| CpiExampleErrors::ErrorCode::DexIxError.into())
}

// cpi -> cross-program instruction
//
