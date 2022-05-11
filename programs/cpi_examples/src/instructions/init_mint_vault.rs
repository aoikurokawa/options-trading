use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitMintVault<'info> {
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    pub underlying_asset: Box<Account<'info, Mint>>,
}