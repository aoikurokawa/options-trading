use anchor_lang::prelude::*;
use anchor_spl::dex::serum_dex::{
    instruction::SelfTradeBehavior as SerumSelfTradeBehavior,
    matching::{OrderType as SerumOrderType, Side as SerumSide},
};
use anchor_spl::dex::*;
use anchor_spl::token::*;
use std::num::NonZeroU64;

use crate::errors as CpiExampleErrors;

#[derive(Debug, AnchorSerialize, AnchorDeserialize)]
pub enum SelfTradeBehavior {
    DecrementTake = 0,
    CancelProvide = 1,
    AbortTransaction = 2,
}

impl From<SelfTradeBehavior> for SerumSelfTradeBehavior {
    fn from(self_trade_behave: SelfTradeBehavior) -> SerumSelfTradeBehavior {
        match self_trade_behave {
            SelfTradeBehavior::DecrementTake => SerumSelfTradeBehavior::DecrementTake,
            SelfTradeBehavior::CancelProvide => SerumSelfTradeBehavior::CancelProvide,
            SelfTradeBehavior::AbortTransaction => SerumSelfTradeBehavior::AbortTransaction,
        }
    }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize)]
pub enum OrderType {
    Limit = 0,
    ImmediateOrCancel = 1,
    PostOnly = 2,
}

impl From<OrderType> for SerumOrderType {
    fn from(order_type: OrderType) -> SerumOrderType {
        match order_type {
            OrderType::Limit => SerumOrderType::Limit,
            OrderType::ImmediateOrCancel => SerumOrderType::ImmediateOrCancel,
            OrderType::PostOnly => SerumOrderType::PostOnly,
        }
    }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize)]
pub enum NewSide {
    Bid,
    Ask,
}

impl From<NewSide> for SerumSide {
    fn from(side: NewSide) -> SerumSide {
        match side {
            NewSide::Bid => SerumSide::Bid,
            NewSide::Ask => SerumSide::Ask,
        }
    }
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    pub user_authority: Signer<'info>,
    /// CHECK: TODO
    pub option_trading_program: AccountInfo<'info>,
    pub dex_program: Program<'info, Dex>,
    /// CHECK: TODO
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub market: AccountInfo<'info>,
    /// CHECK: TODO
    pub market_authority: AccountInfo<'info>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: TODO
    #[account(mut)]
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<PlaceOrder>,
    vault_authority_bump: u8,
    open_order_bump: u8,
    open_order_bump_init: u8,
    side: NewSide,
    limit_price: u64,
    max_coin_qty: u64,
    order_type: OrderType,
    client_order_id: u64,
    self_trade_behavior: SelfTradeBehavior,
    limit: u16,
    max_native_pc_qty_including_fees: u64,
) -> Result<()> {
    let cpi_program = ctx.accounts.option_trading_program.clone();
    if ctx.accounts.open_orders.data_is_empty() {
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                &ctx.accounts.user_authority.key,
                &ctx.accounts.vault_authority.key,
                23347760,
            ),
            &[
                ctx.accounts.user_authority.to_account_info(),
                ctx.accounts.vault_authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let mut ix = serum_dex::instruction::init_open_orders(
            &ctx.accounts.dex_program.key,
            ctx.accounts.open_orders.key,
            ctx.accounts.vault_authority.key,
            ctx.accounts.market.key,
            Some(ctx.accounts.market_authority.key),
        )
        .map_err(|_x| CpiExampleErrors::ErrorCode::DexIxError)?;

        ix.program_id = *cpi_program.key;
        ix.accounts[0].pubkey = ctx.accounts.open_orders.key();
        ix.accounts[4].pubkey = ctx.accounts.market_authority.key();
        ix.accounts[4].is_signer = false;
        ix.accounts[1].is_writable = true;
        ix.accounts.insert(
            0,
            ctx.accounts.system_program.to_account_metas(Some(false))[0].clone(),
        );
        ix.accounts.insert(
            0,
            ctx.accounts.dex_program.to_account_metas(Some(false))[0].clone(),
        );

        ix.data.insert(0, open_order_bump_init);
        ix.data.insert(0, open_order_bump);
        ix.data.insert(0, 0 as u8);
        ix.data.insert(0, 0 as u8);

        ix.accounts.insert(
            0,
            ctx.accounts.dex_program.to_account_metas(Some(false))[0].clone(),
        );

        let vault_key = ctx.accounts.vault.key();
        let vault_authority_seeds = &[
            vault_key.as_ref(),
            b"vaultAuthority",
            &[vault_authority_bump],
        ];
        solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.option_trading_program.to_account_info(),
                ctx.accounts.dex_program.to_account_info(),
                ctx.accounts.open_orders.to_account_info(),
                ctx.accounts.vault_authority.to_account_info(),
                ctx.accounts.market.to_account_info(),
                ctx.accounts.market_authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            &[vault_authority_seeds],
        )?;
    }

    let mut new_order_ix = serum_dex::instruction::new_order(
        ctx.accounts.market.key,
        ctx.accounts.open_orders.key,
        ctx.accounts.request_queue.key,
        ctx.accounts.event_queue.key,
        ctx.accounts.market_bids.key,
        ctx.accounts.market_asks.key,
        &ctx.accounts.vault.key(),
        ctx.accounts.vault_authority.key,
        ctx.accounts.coin_vault.key,
        ctx.accounts.pc_vault.key,
        ctx.accounts.token_program.key,
        &ctx.accounts.rent.key(),
        None,
        ctx.accounts.dex_program.key,
        side.into(),
        NonZeroU64::new(limit_price).unwrap(),
        NonZeroU64::new(max_coin_qty).unwrap(),
        order_type.into(),
        client_order_id,
        self_trade_behavior.into(),
        limit,
        NonZeroU64::new(max_native_pc_qty_including_fees).unwrap(),
    )
    .map_err(|_x| CpiExampleErrors::ErrorCode::DexIxError)?;
    new_order_ix.program_id = *cpi_program.key;
    new_order_ix.data.insert(0, 1 as u8);
    new_order_ix.data.insert(0, 1 as u8);
    new_order_ix.accounts.insert(
        0,
        ctx.accounts.dex_program.to_account_metas(Some(false))[0].clone(),
    );

    let vault_key = ctx.accounts.vault.key();
    let vault_authority_seeds = &[
        vault_key.as_ref(),
        b"vaultAuthority",
        &[vault_authority_bump],
    ];

    solana_program::program::invoke_signed(
        &new_order_ix,
        &[
            ctx.accounts.market.to_account_info(),
            ctx.accounts.open_orders.to_account_info(),
            ctx.accounts.request_queue.to_account_info(),
            ctx.accounts.event_queue.to_account_info(),
            ctx.accounts.market_bids.to_account_info(),
            ctx.accounts.market_asks.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.vault_authority.to_account_info(),
            ctx.accounts.coin_vault.to_account_info(),
            ctx.accounts.pc_vault.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        &[vault_authority_seeds],
    )?;

    Ok(())
}
