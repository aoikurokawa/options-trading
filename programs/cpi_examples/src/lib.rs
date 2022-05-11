use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::dex::serum_dex;
use anchor_spl::dex::serum_dex::{
    instruction::SelfTradeBehavior as SerumSelfTradeBehavior,
    matching::{OrderType as SerumOrderType, Side as SerumSide},
};
use anchor_spl::dex::Dex;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use options_trading::instructions::{ExerciseOption, MintOptionV2};
use options_trading::state::OptionMarket;
use solana_program::msg;
use std::num::NonZeroU64;

pub mod errors;

use errors as CpiExampleErrors;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

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

#[program]
pub mod cpi_examples {
    use super::*;

    pub fn initialize_option_market<'a, 'b, 'c, 'info>(
        ctx: Context<InitOptionMarket>,
    ) -> Result<()> {
        Ok(())
    }
}

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
