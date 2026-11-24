pub mod error;
pub mod events;
pub mod instructions;
pub mod states;

use anchor_lang::prelude::*;
pub use instructions::*;

declare_id!("GL1Z1SKER9eAWHhnKbTiMUHZF2NuDJzeVs34PV7TiXmZ");

#[program]
pub mod pvault {
    use super::*;

    pub fn initialize(ctx: &mut Context<initialize_vault::InitializeVault>) -> Result<()> {
        initialize_vault::process(ctx)
    }

    pub fn deposit(ctx: &mut Context<deposit::Deposit>, amount: u64) -> Result<()> {
        deposit::process(ctx, amount)
    }

    pub fn withdraw(ctx: &mut Context<withdraw::Withdraw>, amount: u64) -> Result<()> {
        withdraw::process(ctx, amount)
    }
}
