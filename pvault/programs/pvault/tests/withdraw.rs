mod common;

use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use common::TransferFixture;
use mollusk_svm_programs_token::token;
use solana_pubkey::Pubkey;

#[test]
fn withdraw_transfers_tokens_to_the_authority_wallet() {
    let authority = Pubkey::new_unique();
    let fixture = TransferFixture::new(authority, authority, 10, 100);
    let instruction = Instruction::new_with_bytes(
        pvault::id(),
        &pvault::instruction::Withdraw { amount: 40 }.data(),
        pvault::accounts::Withdraw {
            authority: fixture.signer,
            mint: fixture.mint,
            vault: fixture.vault,
            vault_token: fixture.vault_token,
            destination_token: fixture.wallet_token,
            token_program: token::ID,
        }
        .to_account_metas(None),
    );

    fixture.check_transfer(instruction, 50, 60);
}
