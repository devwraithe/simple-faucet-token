use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::lamports_to_sol,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    signer::keypair::{read_keypair_file, write_keypair_file},
    sysvar,
    transaction::Transaction,
};
use std::str::FromStr;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
enum FaucetInstruction {
    Initialize { distribution_amount: u64 },
    RequestTokens,
    ReplenishTokens { amount: u64 },
}

fn main() {
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let client = RpcClient::new(rpc_url);

    let raw_program_id = "5gpW17UnnPPzgdhdoHBBJM75fmZaCvX14DjvtxqXsqCY";
    let program_id = Pubkey::from_str(raw_program_id).expect("Failed to parse program ID");

    let faucet_keypair =
        read_keypair_file("faucet_keypair.json").expect("Failed to read faucet keypair");

    // Check faucet balance before processing
    check_faucet_balance(&client, &faucet_keypair.pubkey(), 1).expect("Faucet balance is too low");

    // Initialize faucet (comment out once initialized)
    initialize_faucet(
        &client,
        &program_id,
        &faucet_keypair,
        &faucet_keypair,
        100_000_000,
    );

    // Request tokens from the faucet
    request_tokens(&client, &program_id, &faucet_keypair);

    // Replenish token
    replenish_tokens(
        &client,
        &program_id,
        &faucet_keypair,
        &faucet_keypair,
        100_000_000,
    );
}

fn initialize_faucet(
    client: &RpcClient,
    program_id: &Pubkey,
    faucet_keypair: &Keypair,
    admin_keypair: &Keypair,
    distribution_amount: u64,
) {
    let instruction = Instruction::new_with_borsh(
        *program_id,
        &FaucetInstruction::Initialize {
            distribution_amount,
        },
        vec![
            AccountMeta::new(faucet_keypair.pubkey(), false),
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::id(), true),
        ],
    );

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&admin_keypair.pubkey()),
        &[admin_keypair],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction).unwrap();
    println!("Faucet initialized. Transaction signature: {}", signature);
}

fn request_tokens(client: &RpcClient, program_id: &Pubkey, faucet_keypair: &Keypair) {
    let user_keypair = generate_and_save_keypair();
    println!("User keypair pubkey: {}", user_keypair.pubkey());

    let instruction = Instruction::new_with_borsh(
        *program_id,
        &FaucetInstruction::RequestTokens,
        vec![
            AccountMeta::new(faucet_keypair.pubkey(), false),
            AccountMeta::new(user_keypair.pubkey(), false),
        ],
    );

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&user_keypair.pubkey()),
        &[&user_keypair],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction).unwrap();
    println!("Transaction signature: {}", signature);

    let faucet_balance = client.get_balance(&faucet_keypair.pubkey()).unwrap();
    println!(
        "Faucet current balance: {} lamports ({} SOL)",
        faucet_balance,
        lamports_to_sol(faucet_balance)
    );

    let user_balance = client.get_balance(&user_keypair.pubkey()).unwrap();
    println!(
        "User current balance: {} lamports ({} SOL)",
        user_balance,
        lamports_to_sol(user_balance)
    );

    let transaction_url = format!(
        "https://explorer.solana.com/tx/{}?cluster=devnet",
        signature
    );
    println!("Transaction URL: {}", transaction_url);
}

fn replenish_tokens(
    client: &RpcClient,
    program_id: &Pubkey,
    faucet_keypair: &Keypair,
    admin_keypair: &Keypair,
    amount: u64,
) {
    let instruction = Instruction::new_with_borsh(
        *program_id,
        &FaucetInstruction::ReplenishTokens { amount },
        vec![
            AccountMeta::new(faucet_keypair.pubkey(), false),
            AccountMeta::new(admin_keypair.pubkey(), true),
        ],
    );

    let blockhash = client.get_latest_blockhash().unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&admin_keypair.pubkey()),
        &[admin_keypair],
        blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction).unwrap();
    println!("Transaction replenished. Signature: {}", signature);

    let balance = client.get_balance(&faucet_keypair.pubkey()).unwrap();
    println!(
        "Faucet balance: {} lamports ({} SOL)",
        balance,
        lamports_to_sol(balance)
    );
}

fn generate_and_save_keypair() -> Keypair {
    let file_path = "user_keypair.json";

    if !std::path::Path::new(file_path).exists() {
        println!("Keypair file does not exist, creating one!");

        let user_keypair = Keypair::new();
        write_keypair_file(&user_keypair, file_path).expect("Failed to write keypair to file");

        println!("Keypair saved to user_keypair.json!");
        user_keypair
    } else {
        println!("Keypair file already exists");
        read_keypair_file(file_path).expect("Failed to read keypair file")
    }
}

// Check the SOL balance in the faucet account
fn check_faucet_balance(
    client: &RpcClient,
    faucet_pubkey: &Pubkey,
    min_balance: u32,
) -> Result<(), String> {
    let faucet_balance = client.get_balance(&faucet_pubkey).unwrap_or(0);
    let balance_in_sol = lamports_to_sol(faucet_balance);

    if balance_in_sol < min_balance as f64 {
        return Err(format!(
            "Faucet balance is too low. Current balance: {} SOL, Minimum required: {} SOL",
            balance_in_sol, min_balance,
        ));
    }

    println!(
        "Faucet current balance: {} lamports ({} SOL)",
        faucet_balance,
        lamports_to_sol(faucet_balance)
    );

    Ok(())
}