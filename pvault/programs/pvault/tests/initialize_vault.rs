use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anyhow::{Context, Result};
use mollusk_svm::Mollusk;
use mollusk_svm::program::keyed_account_for_system_program;
use mollusk_svm::result::Check;
use mollusk_svm_programs_token::{associated_token, token};
use pvault::states::vault::Vault;
use solana_account::Account;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_sdk_ids::system_program;
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};

#[test]
fn initialize_creates_vault_and_its_token_account() -> Result<()> {
    let program_id = pvault::id();
    let mut mollusk = Mollusk::new(&program_id, "pvault");
    token::add_program(&mut mollusk);
    associated_token::add_program(&mut mollusk);

    let payer = Pubkey::new_unique();
    let mint = Pubkey::new_unique();
    let (vault, bump) =
        Pubkey::find_program_address(&[Vault::SEED, payer.as_ref(), mint.as_ref()], &program_id);
    let (vault_token, _) = Pubkey::find_program_address(
        &[vault.as_ref(), token::ID.as_ref(), mint.as_ref()],
        &associated_token::ID,
    );
    let instruction = Instruction::new_with_bytes(
        program_id,
        &pvault::instruction::Initialize {}.data(),
        pvault::accounts::InitializeVault {
            payer,
            mint,
            vault,
            vault_token,
            token_program: token::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let accounts = [
        (payer, Account::empty(1_000_000_000, &system_program::ID)),
        (
            mint,
            token::create_account_for_mint(Mint {
                is_initialized: true,
                decimals: 6,
                ..Mint::default()
            }),
        ),
        (vault, Account::default()),
        (vault_token, Account::default()),
        token::keyed_account(),
        associated_token::keyed_account(),
        keyed_account_for_system_program(),
    ];

    let result =
        mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

    let vault_account = result
        .get_account(&vault)
        .with_context(|| format!("Missing vault account {vault}"))?;
    assert_eq!(vault_account.owner, program_id);
    let vault_state = Vault::try_deserialize(&mut vault_account.data.as_slice())
        .with_context(|| format!("Could not deserialize vault account {vault}"))?;
    assert_eq!(vault_state.authority, payer);
    assert_eq!(vault_state.mint, mint);
    assert_eq!(vault_state.bump, bump);

    let token_account = result
        .get_account(&vault_token)
        .with_context(|| format!("Missing vault token account {vault_token}"))?;
    assert_eq!(token_account.owner, token::ID);
    let token_state = TokenAccount::unpack(&token_account.data)
        .with_context(|| format!("Could not unpack vault token account {vault_token}"))?;
    assert_eq!(token_state.state, AccountState::Initialized);
    assert_eq!(token_state.owner, vault);
    assert_eq!(token_state.mint, mint);
    assert_eq!(token_state.amount, 0);
    Ok(())
}
