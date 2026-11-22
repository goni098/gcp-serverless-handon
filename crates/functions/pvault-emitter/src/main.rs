use std::process::ExitCode;

use error::R;
use pvault_client::pvault;
use shared::env::Env;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::VersionedMessage;
use solana_sdk::message::v1::{self, TransactionConfig};
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::transaction::VersionedTransaction;
use solana_system_interface::instruction::create_account;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_associated_token_account_interface::instruction::create_associated_token_account;
use spl_token_2022_interface::instruction::{initialize_mint2, mint_to_checked};
use spl_token_2022_interface::state::Mint;

#[tokio::main]
async fn main() -> ExitCode {
    shared::tracing::install();
    shared::env::load();

    match run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::info!("error {:#?}", error);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> R<()> {
    const MAX_COMPUTE_UNITS: u32 = 1_400_000;
    const MAX_LOADED_DATA_BYTES: u32 = 64 * 1024 * 1024;

    let rpc_url = Env::SolRpcUrl.read()?;
    let sol_pk = Env::SolPk.read()?;

    let client = RpcClient::new(rpc_url);
    let owner_keypair = Keypair::from_base58_string(&sol_pk);
    let mint_keypair = Keypair::new();

    let owner = owner_keypair.pubkey();
    let mint_address = mint_keypair.pubkey();

    let mut instructions = create_blc(&client, &owner_keypair, &mint_address)
        .await?
        .to_vec();

    instructions.push(initialize_vault(owner, mint_address));
    instructions.push(deposit(owner, mint_address, 100_000));
    instructions.push(withdraw(owner, mint_address, 70_000));

    let hash = client.get_latest_blockhash().await?;

    let config = TransactionConfig::default()
        .with_compute_unit_limit(MAX_COMPUTE_UNITS)
        .with_loaded_accounts_data_size_limit(MAX_LOADED_DATA_BYTES);

    let message = v1::Message::try_compile_with_config(&owner, &instructions, hash, config)?;

    let transaction = VersionedTransaction::try_new(
        VersionedMessage::V1(message),
        &[&owner_keypair, &mint_keypair],
    )?;

    let sig = client.send_and_confirm_transaction(&transaction).await?;

    tracing::info!("sig {}", sig);

    Ok(())
}

async fn create_blc(client: &RpcClient, keypair: &Keypair, mint: &Pubkey) -> R<[Instruction; 4]> {
    const AMOUNT: u64 = 1_000_000;
    const DECIMALS: u8 = 6;

    let owner = keypair.pubkey();
    let token_program = spl_token_2022_interface::id();
    let token_account = get_associated_token_address_with_program_id(&owner, mint, &token_program);
    let rent = client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .await?;

    Ok([
        create_account(&owner, mint, rent, Mint::LEN as u64, &token_program),
        initialize_mint2(&token_program, mint, &owner, None, DECIMALS)?,
        create_associated_token_account(&owner, &owner, mint, &token_program),
        mint_to_checked(
            &token_program,
            mint,
            &token_account,
            &owner,
            &[],
            AMOUNT,
            DECIMALS,
        )?,
    ])
}

fn initialize_vault(payer: Pubkey, mint: Pubkey) -> Instruction {
    let token_program = spl_token_2022_interface::id();
    let vault = vault_address(&payer, &mint);
    let vault_token = get_associated_token_address_with_program_id(&vault, &mint, &token_program);

    pvault::instruction::Initialize {}.to_instruction(pvault::accounts::Initialize {
        payer,
        mint,
        vault,
        vault_token,
        token_program,
        associated_token_program: spl_associated_token_account_interface::program::ID,
        system_program: solana_system_interface::program::ID,
    })
}

fn deposit(depositor: Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    let token_program = spl_token_2022_interface::id();
    let vault = vault_address(&depositor, &mint);
    let source_token =
        get_associated_token_address_with_program_id(&depositor, &mint, &token_program);
    let vault_token = get_associated_token_address_with_program_id(&vault, &mint, &token_program);

    pvault::instruction::Deposit { amount }.to_instruction(pvault::accounts::Deposit {
        depositor,
        mint,
        vault,
        source_token,
        vault_token,
        token_program,
    })
}

fn withdraw(authority: Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    let token_program = spl_token_2022_interface::id();
    let vault = vault_address(&authority, &mint);

    let vault_token = get_associated_token_address_with_program_id(&vault, &mint, &token_program);
    let destination_token =
        get_associated_token_address_with_program_id(&authority, &mint, &token_program);

    pvault::instruction::Withdraw { amount }.to_instruction(pvault::accounts::Withdraw {
        authority,
        mint,
        vault,
        vault_token,
        token_program,
        destination_token,
    })
}

fn vault_address(authority: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            &pvault::constants::VAULT_SEED,
            authority.as_ref(),
            mint.as_ref(),
        ],
        &pvault::ID,
    )
    .0
}
