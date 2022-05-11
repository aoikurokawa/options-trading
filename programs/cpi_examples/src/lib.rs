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
pub mod instructions;

use errors as CpiExampleErrors;
use instructions::*;

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
        ctx: Context<'a, 'b, 'c, 'info, InitOptionMarket<'info>>,
        underlying_amount_per_contract: u64,
        quote_amount_per_contract: u64,
        expiration_unix_timestamp: i64,
        bump_seed: u8,
    ) -> Result<()> {
        instructions::init_option_market::handler(
            ctx,
            underlying_amount_per_contract,
            quote_amount_per_contract,
            expiration_unix_timestamp,
            bump_seed,
        )
    }

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        instructions::initialize::handler(ctx, amount)
    }
}
