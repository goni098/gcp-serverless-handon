use anchor_lang::Discriminator;
use anchor_lang::solana_program::instruction::Instruction;
use mollusk_svm::Mollusk;
use mollusk_svm::result::Check;
use mollusk_svm_programs_token::{associated_token, token};
use pvault::states::vault::Vault;
use solana_account::Account;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_sdk_ids::system_program;
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};

pub struct TransferFixture {
    mollusk: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
    pub signer: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub vault_token: Pubkey,
    pub wallet_token: Pubkey,
}

impl TransferFixture {
    pub fn new(
        signer: Pubkey,
        authority: Pubkey,
        wallet_balance: u64,
        vault_balance: u64,
    ) -> Self {
        let program_id = pvault::id();
        let mut mollusk = Mollusk::new(&program_id, "pvault");
        token::add_program(&mut mollusk);

        let mint = Pubkey::new_unique();
        let (vault, bump) = Pubkey::find_program_address(
            &[Vault::SEED, authority.as_ref(), mint.as_ref()],
            &program_id,
        );
        let (vault_token, _) = Pubkey::find_program_address(
            &[vault.as_ref(), token::ID.as_ref(), mint.as_ref()],
            &associated_token::ID,
        );
        let wallet_token = Pubkey::new_unique();
        let vault_state = Vault {
            authority,
            mint,
            bump,
        };
        let mut vault_data = Vault::DISCRIMINATOR.to_vec();
        vault_data.extend_from_slice(bytemuck::bytes_of(&vault_state));

        let token_account = |owner, amount| {
            token::create_account_for_token_account(TokenAccount {
                mint,
                owner,
                amount,
                state: AccountState::Initialized,
                ..TokenAccount::default()
            })
        };
        let accounts = vec![
            (signer, Account::empty(1_000_000_000, &system_program::ID)),
            (
                mint,
                token::create_account_for_mint(Mint {
                    is_initialized: true,
                    decimals: 6,
                    supply: wallet_balance + vault_balance,
                    ..Mint::default()
                }),
            ),
            (
                vault,
                Account {
                    lamports: 1_000_000_000,
                    data: vault_data,
                    owner: program_id,
                    ..Account::default()
                },
            ),
            (vault_token, token_account(vault, vault_balance)),
            (wallet_token, token_account(signer, wallet_balance)),
            token::keyed_account(),
        ];

        Self {
            mollusk,
            accounts,
            signer,
            mint,
            vault,
            vault_token,
            wallet_token,
        }
    }

    pub fn check_transfer(
        &self,
        instruction: Instruction,
        wallet_balance: u64,
        vault_balance: u64,
    ) {
        let result = self.mollusk.process_and_validate_instruction(
            &instruction,
            &self.accounts,
            &[Check::success()],
        );

        for (address, expected_balance) in [
            (self.wallet_token, wallet_balance),
            (self.vault_token, vault_balance),
        ] {
            let account = result.get_account(&address).expect("token account");
            let token = TokenAccount::unpack(&account.data).expect("initialized token account");
            assert_eq!(token.amount, expected_balance, "balance of {address}");
        }
    }
}
