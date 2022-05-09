pub mod errors;
pub mod fees;
pub mod instructions;
pub mod serum_proxy;
pub mod state;

// use crate::state::option_market::OptionMarket;
use anchor_lang::{prelude::*, AccountsExit, Key};
use anchor_spl::dex::{
    initialize_market as init_serum_market_instruction, InitializeMarket as SerumInitMarket,
};
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};
use psyfi_serum_dex_permissioned::{MarketProxy, OpenOrdersPda, ReferralFees};
use solana_program::{
    program::invoke, program_error::ProgramError, program_pack::Pack, system_instruction,
    system_program,
};
use spl_token::state::Account as SPLTokenAccount;

declare_id!("6xUQFHLbbfhayBwLBNSMfZfCmHNJJujWsS88qCtGfWdn");

#[program]
pub mod options_trading {
    use super::*;

    #[access_control(InitializeMarket::accounts(&ctx))]
    pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
        Ok(())
    }
}
