use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::dex::serum_dex;

use anchor_spl::dex::Dex;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use options_trading::instructions::{ExerciseOption, MintOptionV2};
use options_trading::state::OptionMarket;
use solana_program::msg;


pub mod errors;
pub mod instructions;

use errors as CpiExampleErrors;
use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

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

    pub fn exercise<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Exercise<'info>>,
        vault_authority_bump: u8,
    ) -> Result<()> {
        instructions::exercise::exercise(ctx, vault_authority_bump)
    }

    pub fn init_mint_vault(_ctx: Context<InitMintVault>) -> Result<()> {
        instructions::init_mint_vault::handler(_ctx)
    }

    pub fn mint<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, MintCtx<'info>>,
        size: u64,
        vault_authority_bump: u8,
    ) -> Result<()> {
        instructions::mint::handler(ctx, size, vault_authority_bump)
    }

    pub fn init_new_order_vault(_ctx: Context<InitNewOrderVault>) -> Result<()> {
        instructions::init_new_order_vault::handler(_ctx)
    }
}
