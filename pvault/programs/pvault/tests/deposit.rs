mod common;

use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use common::TransferFixture;
use mollusk_svm_programs_token::token;
use solana_pubkey::Pubkey;

#[test]
fn deposit_transfers_tokens_from_a_non_authority_wallet() {
    let fixture = TransferFixture::new(Pubkey::new_unique(), Pubkey::new_unique(), 100, 10);
    let instruction = Instruction::new_with_bytes(
        pvault::id(),
        &pvault::instruction::Deposit { amount: 40 }.data(),
        pvault::accounts::Deposit {
            depositor: fixture.signer,
            mint: fixture.mint,
            vault: fixture.vault,
            source_token: fixture.wallet_token,
            vault_token: fixture.vault_token,
            token_program: token::ID,
        }
        .to_account_metas(None),
    );

    fixture.check_transfer(instruction, 60, 50);
}
