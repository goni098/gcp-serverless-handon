use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked};
use anchor_spl::{mint, token};

use crate::events::Withdrawn;
use crate::states::vault::Vault;

#[derive(Accounts)]
pub struct Withdraw {
    #[account(
        mut,
        address = vault.authority
    )]
    pub authority: Signer,

    #[account(
        address = vault.mint,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<Mint>,

    #[account(
        seeds = [Vault::SEED, authority.address().as_ref(), mint.address().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<Vault>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_token: InterfaceAccount<TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = authority,
        token::token_program = token_program
    )]
    pub destination_token: InterfaceAccount<TokenAccount>,

    pub token_program: Interface<TokenInterface>,
}

pub fn process(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
    let accounts = &mut ctx.accounts;

    let bump = [accounts.vault.bump];
    let seeds = [
        Vault::SEED.as_slice(),
        accounts.authority.address().as_ref(),
        accounts.mint.address().as_ref(),
        &bump,
    ];
    let signer_seeds = &[seeds.as_slice()];

    let transfer = CpiContext::<_, 4>::new_with_signer(
        accounts.token_program.address(),
        TransferChecked {
            from: accounts.vault_token.cpi_handle_mut(),
            mint: accounts.mint.cpi_handle(),
            to: accounts.destination_token.cpi_handle_mut(),
            authority: accounts.vault.cpi_handle(),
        },
        signer_seeds,
    );

    anchor_spl::token_interface::transfer_checked(transfer, amount, accounts.mint.decimals())?;

    emit!(Withdrawn {
        mint: *accounts.mint.address(),
        authority: *accounts.authority.address(),
        amount
    });

    Ok(())
}
