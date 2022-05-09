use crate::state::option_market::OptionMarket;
use anchor_lang::prelude::*;
use anchor_spl::token::*;
use solana_program::{
    program::invoke, program_error::ProgramError, program_pack::Pack, system_instruction,
    system_program,
};
use spl_token::state::Account as SPLTokenAccount;

use crate::errors;
use crate::fees;

#[derive(Accounts)]
pub struct MintOption<'info> {
    /// The user authority must be the authority that has ownership of the `underlying_asset_src`
    #[account(mut)]
    pub user_authority: Signer<'info>,
    /// CHECK: Handled
    pub underlying_asset_mint: AccountInfo<'info>,
    #[account(mut)]
    pub underlying_asset_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub underlying_asset_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub minted_option_dest: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub writer_token_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub minted_writer_token_dest: Box<Account<'info, TokenAccount>>,
    pub option_market: Box<Account<'info, OptionMarket>>,
    /// CHECK: Handled
    #[account(mut)]
    pub fee_owner: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK Unnecessary account, but left for backwards compatibility
    pub associated_token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintOption<'info> {
    pub fn accounts(ctx: &Context<MintOption<'info>>) -> Result<()> {
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

        // Validate the writer token mint is the same as on the OptionMarket
        if *ctx.accounts.writer_token_mint.to_account_info().key
            != ctx.accounts.option_market.writer_token_mint
        {
            return Err(errors::ErrorCode::FeeOwnerDoesNotMatchProgram.into());
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

    pub fn unexpired_market(ctx: &Context<MintOption<'info>>) -> Result<()> {
        // Validate the market is not expired
        if ctx.accounts.option_market.expiration_unix_timestamp < ctx.accounts.clock.unix_timestamp
        {
            return Err(errors::ErrorCode::OptionMarketExpiredCantMint.into());
        }

        Ok(())
    }
}

pub fn helper<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, MintOption<'info>>,
    size: u64,
) -> Result<()> {
    let option_market = &ctx.accounts.option_market;
    let mint_fee_account = validate_mint_fee_acct(option_market, ctx.remaining_accounts)?;

    // Take a mint fee
    let mint_fee_amount_per_contract =
        fees::fee_amount(option_market.underlying_amount_per_contract);
    if mint_fee_amount_per_contract > 0 {
        match mint_fee_account {
            Some(account) => {
                let cpi_accounts = Transfer {
                    from: ctx.accounts.underlying_asset_src.to_account_info(),
                    to: account.clone(),
                    authority: ctx.accounts.user_authority.to_account_info().clone(),
                };
                let cpi_token_program = ctx.accounts.token_program.clone();
                let cpi_ctx = CpiContext::new(cpi_token_program.to_account_info(), cpi_accounts);
                let total_fee = mint_fee_amount_per_contract
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

    // Transfer the underlying assets to the underlying assets pool
    let cpi_accounts = Transfer {
        from: ctx.accounts.underlying_asset_src.to_account_info(),
        to: ctx.accounts.underlying_asset_pool.to_account_info(),
        authority: ctx.accounts.user_authority.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_token_program.to_account_info(), cpi_accounts);
    let underlying_transfer_amount = option_market
        .underlying_amount_per_contract
        .checked_mul(size)
        .unwrap();
    transfer(cpi_ctx, underlying_transfer_amount)?;

    let seeds = &[
        option_market.underlying_asset_mint.as_ref(),
        option_market.quote_asset_mint.as_ref(),
        &option_market.underlying_amount_per_contract.to_le_bytes(),
        &option_market.quote_amount_per_contract.to_le_bytes(),
        &option_market.expiration_unix_timestamp.to_le_bytes(),
        &[option_market.bump_seed],
    ];
    let signer = &[&seeds[..]];

    // Mint a new OptionToken(s)
    let cpi_accounts = MintTo {
        mint: ctx.accounts.option_mint.to_account_info(),
        to: ctx.accounts.minted_option_dest.to_account_info(),
        authority: ctx.accounts.option_market.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx =
        CpiContext::new_with_signer(cpi_token_program.to_account_info(), cpi_accounts, signer);
    mint_to(cpi_ctx, size)?;

    // Mint a new WriterToken(s)
    let cpi_accounts = MintTo {
        mint: ctx.accounts.writer_token_mint.to_account_info(),
        to: ctx.accounts.minted_writer_token_dest.to_account_info(),
        authority: ctx.accounts.option_market.to_account_info(),
    };
    let cpi_token_program = ctx.accounts.token_program.clone();
    let cpi_ctx =
        CpiContext::new_with_signer(cpi_token_program.to_account_info(), cpi_accounts, signer);
    mint_to(cpi_ctx, size)?;

    Ok(())
}

fn validate_mint_fee_acct<'c, 'info>(
    option_market: &OptionMarket,
    remaining_accounts: &'c [AccountInfo<'info>],
) -> Result<Option<&'c AccountInfo<'info>>> {
    let account_info_iter = &mut remaining_accounts.iter();
    let acct;

    if fees::fee_amount(option_market.underlying_amount_per_contract) > 0 {
        let mint_fee_recipient = next_account_info(account_info_iter)?;
        if mint_fee_recipient.owner != &spl_token::ID {
            return Err(errors::ErrorCode::ExpectedSPLTokenProgramId.into());
        }
        let mint_fee_account =
            SPLTokenAccount::unpack_from_slice(&mint_fee_recipient.try_borrow_data()?)?;
        if mint_fee_account.owner != fees::fee_owner_key::ID {
            return Err(errors::ErrorCode::MintFeeMustBeOwnedByFeeOwner.into());
        }

        // check that the mint fee recipient account's mint is also the underlying mint
        if mint_fee_account.mint != option_market.underlying_asset_mint {
            return Err(errors::ErrorCode::MintFeeTokenMustMatchUnderlyingAsset.into());
        }
        if *mint_fee_recipient.key != option_market.mint_fee_account {
            return Err(errors::ErrorCode::MintFeeKeyDoesNotMatchOptionMarket.into());
        }
        acct = Some(mint_fee_recipient)
    } else {
        acct = None;
    }

    Ok(acct)
}
