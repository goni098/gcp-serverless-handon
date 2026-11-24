use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
use anchor_spl::{mint, token};

use crate::error::ErrorCode;
use crate::events::Deposited;
use crate::states::vault::Vault;

#[derive(Accounts)]
pub struct Deposit {
    #[account(mut)]
    pub depositor: Signer,

    #[account(
        address = vault.mint,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<Mint>,

    #[account(
        seeds = [Vault::SEED, vault.authority.as_ref(), mint.address().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<Vault>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = depositor,
        token::token_program = token_program
    )]
    pub source_token: InterfaceAccount<TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_token: InterfaceAccount<TokenAccount>,

    pub token_program: Interface<TokenInterface>,
}

pub fn process(ctx: &mut Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::ZeroDepositAmount);

    let accounts = &mut ctx.accounts;

    let transfer = CpiContext::<_, 4>::new(
        accounts.token_program.address(),
        TransferChecked {
            from: accounts.source_token.cpi_handle_mut(),
            mint: accounts.mint.cpi_handle(),
            to: accounts.vault_token.cpi_handle_mut(),
            authority: accounts.depositor.cpi_handle(),
        },
    );

    token_interface::transfer_checked(transfer, amount, accounts.mint.decimals())?;

    emit!(Deposited {
        mint: *accounts.mint.address(),
        depositor: *accounts.depositor.address(),
        amount
    });

    Ok(())
}
