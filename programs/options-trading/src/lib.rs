pub mod errors;
pub mod fees;
pub mod instructions;
pub mod serum_proxy;
pub mod state;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("6xUQFHLbbfhayBwLBNSMfZfCmHNJJujWsS88qCtGfWdn");

#[program]
pub mod options_trading {
    use super::*;

    #[access_control(InitializeMarket::accounts(&ctx))]
    pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
        Ok(())
    }
}
