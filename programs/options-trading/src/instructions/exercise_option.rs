use crate::errors;
use crate::fees;
use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::*;
use solana_program::program_pack::Pack;
use solana_program::{
    program::invoke, program_error::ProgramError, system_instruction, system_program,
};
use spl_token::state::Account as SPLTokenAccount;

#[derive(Accounts)]
pub struct ExerciseOption<'info> {
    /// The user_authority must be the authority that has ownership of the `quote_asset_src` account
    pub user_authority: Signer<'info>,
    /// The owner of the `exercise_option_token_src` account
    /// CHECK: Handled
    #[account(mut, signer)]
    pub option_authority: AccountInfo<'info>,
    pub option_market: Box<Account<'info, OptionMarket>>,
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub exerciser_option_token_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub underlying_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub underlying_asset_dest: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_asset_src: Box<Account<'info, TokenAccount>>,
    /// CHECK: Handled
    #[account(mut)]
    pub fee_owner: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> ExerciseOption<'info> {
    pub fn accounts(ctx: &Context<ExerciseOption>) -> Result<()> {
        // Validate the quote asset pool is the same as on the OptionMarket
        if *ctx.accounts.quote_asset_pool.to_account_info().key
            != ctx.accounts.option_market.quote_asset_pool
        {
            return Err(errors::ErrorCode::QuotePoolAccountDoesNotMatchMarket.into());
        }

        // Validate the underlying asset pool is the same as on the OptionMarket
        if *ctx.accounts.underlying_asset_pool.to_account_info().key
            != ctx.accounts.option_market.underlying_asset_pool
        {
            return Err(errors::ErrorCode::UnderlyingPoolAccountDoesNotMatchMarket.into());
        }

        // Validate the option mint is the same as on the OptionMarket
        if *ctx.accounts.option_mint.to_account_info().key != ctx.accounts.option_market.option_mint
        {
            return Err(errors::ErrorCode::OptionTokenMintDoesNotMatchMarket.into());
        }

        // Validate the underlying destination has the same mint as the pool
        if ctx.accounts.underlying_asset_dest.mint
            != ctx.accounts.option_market.underlying_asset_mint
        {
            return Err(errors::ErrorCode::UnderlyingDestMintDoesNotMatchUnderlyingAsset.into());
        }

        // Validate the fee owner is correct
        if *ctx.accounts.fee_owner.key != fees::fee_owner_key::ID {
            return Err(errors::ErrorCode::FeeOwnerDoesNotMatchProgram.into());
        }

        // Validate the system program account passed in is correct
        if !system_program::check_id(ctx.accounts.system_program.key) {
            return Err(ProgramError::InvalidAccountData.into());
        }

        Ok(())
    }

    pub fn unexpired_market(ctx: &Context<ExerciseOption>) -> Result<()> {
        // Validate the market is not expired
        if ctx.accounts.option_market.expiration_unix_timestamp < ctx.accounts.clock.unix_timestamp
        {
            return Err(errors::ErrorCode::OptionMarketExpiredCantExercise.into());
        }

        Ok(())
    }
}

// to_le_bytes => return the memory representation of this integer as a byte array in little-endian byte order
pub fn handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ExerciseOption<'info>>,
    size: u64,
) -> Result<()> {
    let option_market = &ctx.accounts.option_market;
    let seeds = &[
        option_market.underlying_asset_mint.as_ref(),
        option_market.quote_asset_mint.as_ref(),
        &option_market.underlying_amount_per_contract.to_le_bytes(),
        &option_market.quote_amount_per_contract.to_le_bytes(),
        &option_market.expiration_unix_timestamp.to_le_bytes(),
        &[option_market.bump_seed],
    ];
    let signer = &[&seeds[..]];

    // Burn the size of option tokens
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info().clone(),
        Burn {
            mint: ctx.accounts.option_mint.to_account_info(),
            from: ctx.accounts.exerciser_option_token_src.to_account_info(),
            authority: ctx.accounts.option_authority.to_account_info(),
        },
        signer,
    );
    burn(cpi_ctx, size)?;

    // Transfer the quote assets to the pool
    let cpi_accounts = Transfer {
        from: ctx.accounts.quote_asset_src.to_account_info(),
        to: ctx.accounts.quote_asset_pool.to_account_info(),
        authority: ctx.accounts.user_authority.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_token_program.to_account_info(), cpi_accounts);
    let quote_transfer_amount = option_market
        .quote_amount_per_contract
        .checked_mul(size)
        .unwrap();
    transfer(cpi_ctx, quote_transfer_amount)?;

    // Transfer the underlying assets from the pool to the exerciser
    let cpi_accounts = Transfer {
        from: ctx.accounts.quote_asset_src.to_account_info(),
        to: ctx.accounts.quote_asset_pool.to_account_info(),
        authority: ctx.accounts.user_authority.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx =
        CpiContext::new_with_signer(cpi_token_program.to_account_info(), cpi_accounts, signer);
    // checked_mul => multiplies 2 numbers, checking for underflow or overflow. If underflow or overflow happens, None is returned.
    let underlying_transfer_amount = option_market
        .underlying_amount_per_contract
        .checked_mul(size)
        .unwrap();
    transfer(cpi_ctx, underlying_transfer_amount)?;

    // Tansfer an exercise fee
    let exercise_fee_account = validate_exercise_fee_acct(option_market, ctx.remaining_accounts)?;
    let exercise_fee_amount_per_contract =
        fees::fee_amount(option_market.quote_amount_per_contract);
    if exercise_fee_amount_per_contract > 0 {
        match exercise_fee_account {
            Some(account) => {
                let cpi_accounts = Transfer {
                    from: ctx.accounts.quote_asset_src.to_account_info(),
                    to: account.clone(),
                    authority: ctx.accounts.user_authority.to_account_info().clone(),
                };
                let cpi_token_program = ctx.accounts.token_program.clone();
                let cpi_ctx = CpiContext::new(cpi_token_program.to_account_info(), cpi_accounts);
                let total_fee = exercise_fee_amount_per_contract
                    .checked_mul(size)
                    .ok_or(errors::ErrorCode::NumberOverflow)?;
                transfer(cpi_ctx, total_fee)?;
            }
            None => {}
        }
    } else {
        // Handle NFT case with SOL fee
        let total_fee = fees::NFT_MINT_LAMPORTS
            .checked_mul(size)
            .ok_or(errors::ErrorCode::NumberOverflow)?;
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.user_authority.key,
                &fees::fee_owner_key::ID,
                total_fee,
            ),
            &[
                ctx.accounts.user_authority.to_account_info().clone(),
                ctx.accounts.fee_owner.clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;
    }

    Ok(())
}

pub fn validate_exercise_fee_acct<'c, 'info>(
    option_market: &OptionMarket,
    remaining_accounts: &'c [AccountInfo<'info>],
) -> Result<Option<&'c AccountInfo<'info>>> {
    let account_info_iter = &mut remaining_accounts.iter();
    let acct;

    if fees::fee_amount(option_market.quote_amount_per_contract) > 0 {
        let exercise_fee_recipient = next_account_info(account_info_iter)?;
        if exercise_fee_recipient.owner != &spl_token::ID {
            return Err(errors::ErrorCode::ExpectedSPLTokenProgramId.into());
        }

        let exercise_fee_account =
            SPLTokenAccount::unpack_from_slice(&exercise_fee_recipient.try_borrow_data()?)?;
        if exercise_fee_account.owner != fees::fee_owner_key::ID {
            return Err(errors::ErrorCode::ExerciseFeeMustBeOwnedByFeeOwner.into());
        }
        // check that the mint fee recipient account's mint is also the underlying mint
        if exercise_fee_account.mint != option_market.quote_asset_mint {
            return Err(errors::ErrorCode::ExerciseFeeTokenMustMatchQuoteAsset.into());
        }
        //check the exercise fee account matches the one on the OptionMarket
        if *exercise_fee_recipient.key != option_market.exercise_fee_account {
            return Err(errors::ErrorCode::ExerciseFeeKeyDoesNotMatchOptionMarket.into());
        }
        acct = Some(exercise_fee_recipient);
    } else {
        acct = None;
    }

    Ok(acct)
}
