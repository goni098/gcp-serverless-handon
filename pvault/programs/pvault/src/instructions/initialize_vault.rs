use anchor_lang::prelude::*;
use anchor_lang::programs::AssociatedToken;
use anchor_spl::mint;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::events::InitializedVault;
use crate::states::vault::Vault;

#[derive(Accounts)]
pub struct InitializeVault {
    #[account(mut)]
    pub payer: Signer,

    #[account(mint::token_program = token_program)]
    pub mint: InterfaceAccount<Mint>,

    #[account(
        init,
        // space = <Account<Vault> as Space>::INIT_SPACE,
        payer = payer,
        seeds = [Vault::SEED, payer.address(), mint.address()],
        bump
    )]
    pub vault: Account<Vault>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_token: InterfaceAccount<TokenAccount>,

    pub token_program: Interface<TokenInterface>,

    pub associated_token_program: Program<AssociatedToken>,

    pub system_program: Program<System>,
}

pub fn process(ctx: &mut Context<InitializeVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let payer = &ctx.accounts.payer;
    let mint = &ctx.accounts.mint;

    vault.authority = *payer.address();
    vault.mint = *mint.address();
    vault.bump = ctx.bumps.vault;

    emit!(InitializedVault {
        authority: *payer.address(),
        mint: *mint.address()
    });

    Ok(())
}
