use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct Vault {
    pub authority: Address,
    pub mint: Address,
    pub bump: u8,
}

#[constant]
const VAULT_SEED: &[u8; 5] = b"vault";

impl Vault {
    pub const SEED: &[u8; 5] = VAULT_SEED;
}
