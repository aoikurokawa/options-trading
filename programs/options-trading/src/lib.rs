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
    ) -> Result<()> {
        instructions::initialize_market::handler(
            ctx,
            underlying_amount_per_contract,
            quote_amount_per_contract,
            expiration_unix_timestamp,
        )
    }

    #[access_control(MintOption::unexpired_market(&ctx) MintOption::accounts(&ctx) validate_size(size))]
    pub fn mint_option<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, MintOption<'info>>,
        size: u64,
    ) -> Result<()> {
        instructions::mint_option::handler(ctx, size)
    }

    #[access_control(MintOptionV2::unexpired_market(&ctx) MintOptionV2::accounts(&ctx) validate_size(size))]
    pub fn mint_option_v2<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, MintOptionV2<'info>>,
        size: u64,
    ) -> Result<()> {
        instructions::mint_option_v2::handler(ctx, size)
    }

    #[access_control(ExerciseOption::accounts(&ctx) ExerciseOption::unexpired_market(&ctx))]
    pub fn exercise_option<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ExerciseOption<'info>>,
        size: u64,
    ) -> Result<()> {
        instructions::exercise_option::handler(ctx, size)
    }

    #[access_control(ExerciseOptionV2::accounts(&ctx) ExerciseOptionV2::unexpired_market(&ctx))]
    pub fn exercise_option_v2<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ExerciseOptionV2<'info>>,
        size: u64,
    ) -> Result<()> {
        instructions::exercise_option_v2::handler(ctx, size)
    }
}

fn validate_size(size: u64) -> Result<()> {
    if size <= 0 {
        return Err(errors::ErrorCode::SizeCantBeLessThanEqZero.into());
    }
    Ok(())
}
