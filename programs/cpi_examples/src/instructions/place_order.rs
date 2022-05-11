use anchor_lang::prelude::*;
use anchor_spl::dex::*;
use anchor_spl::token::*;
use anchor_spl::dex::serum_dex::{
    instruction::SelfTradeBehavior as SerumSelfTradeBehavior,
    matching::{OrderType as SerumOrderType, Side as SerumSide},
};

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
    pub option_trading_program: AccountInfo<'info>,
    pub dex_program: Program<'info, Dex>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub market: AccountInfo<'info>,
    pub market_authority: AccountInfo<'info>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub vault_authority: AccountInfo<'info>,

    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn place_order(
    ctx: Context<PlaceOrder>,
    vault_authority_bump: u8,
    open_order_bump: u8,
    open_order_bump_init: u8,
    side: NewSide,
    limit_price: u64,
    max_coin_qty: u64,
    order_type: OrderType,
    client_order_id: u64,
    self_trade_bahavior: SelfTradeBehavior,
    limit: u16,
    max_native_pc_qty_including_fees: u64,
) -> Result<()> {

    Ok(())
}
