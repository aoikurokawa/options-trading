use anchor_lang::prelude::*;
use instructions::*;
use psyfi_serum_dex_permissioned::{MarketProxy, OpenOrdersPda, ReferralFees};

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
            bump_seed
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

    #[access_control(ClosePostExp::accounts(&ctx) ClosePostExp::expired_market(&ctx))]
    pub fn close_post_expiration(ctx: Context<ClosePostExp>, size: u64) -> Result<()> {
        instructions::close_post_exp::handler(ctx, size)
    }

    #[access_control(CloseOptionPosition::accounts(&ctx))]
    pub fn close_option_position(ctx: Context<CloseOptionPosition>, size: u64) -> Result<()> {
        instructions::close_option_position::handler(ctx, size)
    }

    #[access_control(BurnWriterForQuote::accounts(&ctx) BurnWriterForQuote::quotes_in_pool(&ctx, size))]
    pub fn burn_writer_for_quote(ctx: Context<BurnWriterForQuote>, size: u64) -> Result<()> {
        instructions::burn_writer_for_quote::handler(ctx, size)
    }

    #[access_control(InitSerumMarket::accounts(&ctx))]
    pub fn init_serum_market(
        ctx: Context<InitSerumMarket>,
        _market_space: u64,
        vault_signer_nonce: u64,
        coin_lot_size: u64,
        pc_lot_size: u64,
        pc_dust_threshold: u64,
    ) -> Result<()> {
        instructions::init_serum_market::handler(
            ctx,
            _market_space,
            vault_signer_nonce,
            coin_lot_size,
            pc_lot_size,
            pc_dust_threshold,
        )
    }

    pub fn entry(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> Result<()> {
        MarketProxy::new()
            .middleware(&mut serum_proxy::Validation::new())
            .middleware(&mut ReferralFees::new(serum_proxy::referral::ID))
            .middleware(&mut OpenOrdersPda::new())
            .run(program_id, accounts, data)
    }
}

fn validate_size(size: u64) -> Result<()> {
    if size <= 0 {
        return Err(errors::ErrorCode::SizeCantBeLessThanEqZero.into());
    }
    Ok(())
}
