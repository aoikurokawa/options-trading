pub mod errors;
pub mod fees;
pub mod serum_proxy;

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod options_trading {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[account]
#[derive(Default)]
/// Data structure that contains all the information needed to maintain an open
/// option market.
pub struct OptionMarket {
    /// The SPL Token mint address for the tokens that denote an option
    pub option_mint: Pubkey,
    /// The SPL Token mint address for Writer Tokens that denote a written option
    pub writer_token_mint: Pubkey,
    /// The SPL Token Address that is held in the program's pool when an option is written
    pub underlying_asset_mint: Pubkey,
    /// The SPL Token Address that denominates the strike price
    pub quote_asset_mint: Pubkey,
    /// The amount of the **underlying asset** that derives a single option
    pub underlying_amount_per_contract: u64,
    /// The amount of **quote asset** that must be transfered when an option is exercised
    pub quote_amount_per_contract: u64,
    /// The Unix timestamp at which the contracts in this market expire
    pub expiration_unix_timestamp: i64,
    /// Address for the luquidity pool that contains the underlying asset
    pub underlying_asset_pool: Pubkey,
    /// Address for the liquidity pool that contains the quote asset when
    /// options are exercised
    pub quote_asset_pool: Pubkey,
    /// The SPL Token account (from the Associated Token Program) that collects
    /// fee on mint.
    pub mint_fee_account: Pubkey,
    /// The SPL Token account (from the Associated Token Program) that collects
    /// fees on exercise.
    pub exercise_fee_account: Pubkey,
    /// A flag to set and use to when running a memcmp query.
    /// This will be set when Serum markets are closed and expiration is validated
    pub expired: bool,
    /// Bump seed for the market PDA
    pub bump_seed: u8,
}
