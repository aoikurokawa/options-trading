use anchor_lang::prelude::*;
use instructions::*;

pub mod errors;
pub mod fees;
pub mod instructions;
pub mod serum_proxy;
pub mod state;

declare_id!("6xUQFHLbbfhayBwLBNSMfZfCmHNJJujWsS88qCtGfWdn");

#[program]
pub mod options_trading {
    use super::*;

    #[access_control(InitializeMarket::accounts(&ctx))]
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        underlying_amount_per_contract: u64,
        quote_amount_per_contract: u64,
        expiration_unix_timestamp: i64,
        bump_seed: u8,
    ) -> Result<()> {
        instructions::initialize_market::handler(
            ctx,
            underlying_amount_per_contract,
            quote_amount_per_contract,
            expiration_unix_timestamp,
            bump_seed,
        )
    }
}
