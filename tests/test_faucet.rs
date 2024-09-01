use borsh::BorshDeserialize;
use simple_token_faucet::instructions::FaucetInstruction;
use simple_token_faucet::process_instruction;
use simple_token_faucet::state::FaucetState;
use solana_program::rent::Rent;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_initialize() {
    // Create program and test environment
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "simple_token_faucet_initialize",
        program_id,
        processor!(process_instruction),
    );

    // Generate keypairs for accounts
    let faucet_keypair = Keypair::new();
    let admin_keypair = Keypair::new();

    // Calculate rent-exempt balance
    let rent = Rent::default();
    let faucet_account_rent = rent.minimum_balance(size_of::<FaucetState>()); // rent exempt

    // Add faucet account to test environment
    program_test.add_account(
        faucet_keypair.pubkey(),
        Account {
            lamports: faucet_account_rent,
            data: vec![0; size_of::<FaucetState>()],
            owner: program_id,
            ..Account::default() // use default values for other fields
        },
    );

    // Add admin account to test environment
    program_test.add_account(
        admin_keypair.pubkey(),
        Account {
            lamports: 100_000_000, // 0.001 SOL, enough for rent & gas
            ..Account::default()
        },
    );

    // Start the test env
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create init instruction
    let distribution_amount = 1000;
    let init_instruction = Instruction::new_with_borsh(
        program_id,
        &FaucetInstruction::Initialize {
            distribution_amount,
        },
        vec![ // defines accounts involved with this instruction
            AccountMeta::new(faucet_keypair.pubkey(), false),
            AccountMeta::new(admin_keypair.pubkey(), true), // a signer
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
    );

    // Create and sign transaction
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &admin_keypair], recent_blockhash);

    // Submit transaction
    banks_client.process_transaction(transaction).await.unwrap();

    // Fetch the faucet account
    let faucet_account = banks_client
        .get_account(faucet_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();

    // Deserialize the faucet account
    let faucet_state = FaucetState::try_from_slice(&faucet_account.data).unwrap();

    // Verify the faucet state
    assert_eq!(faucet_state.admin, admin_keypair.pubkey());
    assert_eq!(faucet_state.distribution_amount, distribution_amount);

    // Verify the account is still rent-exempt
    assert!(rent.is_exempt(faucet_account.lamports, faucet_account.data.len()));
}

#[tokio::test]
async fn test_request_token() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "simple_token_faucet_request_token",
        program_id,
        processor!(process_instruction),
    );

    let faucet_keypair = Keypair::new();
    let user_keypair = Keypair::new();
    let admin_keypair = Keypair::new();

    // Initialize faucet state
    let distribution_amount = 1000;

    let rent = Rent::default();
    let account_size = size_of::<FaucetState>();
    let faucet_account_rent = rent.minimum_balance(account_size);

    program_test.add_account(
        faucet_keypair.pubkey(),
        Account {
            lamports: faucet_account_rent + 10_000_000, // Rent + initial balance
            data: vec![0; account_size],
            owner: program_id,
            ..Account::default()
        },
    );

    // Add user account
    program_test.add_account(
        user_keypair.pubkey(),
        Account {
            lamports: rent.minimum_balance(0),
            owner: system_program::id(),
            ..Account::default()
        },
    );

    // Add admin account
    program_test.add_account(
        admin_keypair.pubkey(),
        Account {
            lamports: 1_000_000_000,
            owner: system_program::id(),
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Initialize the faucet
    let init_instruction = Instruction::new_with_borsh(
        program_id,
        &FaucetInstruction::Initialize {
            distribution_amount,
        },
        vec![
            AccountMeta::new(faucet_keypair.pubkey(), false),
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &admin_keypair], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    // Request tokens
    let request_instruction = Instruction::new_with_borsh(
        program_id,
        &FaucetInstruction::RequestTokens,
        vec![
            AccountMeta::new(faucet_keypair.pubkey(), false),
            AccountMeta::new(user_keypair.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction =
        Transaction::new_with_payer(&[request_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &user_keypair], recent_blockhash);

    // Send and confirm transaction
    banks_client.process_transaction(transaction).await.unwrap();

    // Check balances
    let faucet_account = banks_client
        .get_account(faucet_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let user_account = banks_client
        .get_account(user_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        faucet_account.lamports,
        faucet_account_rent + 10_000_000 - distribution_amount
    );
    assert_eq!(
        user_account.lamports,
        rent.minimum_balance(0) + distribution_amount
    );
}

#[tokio::test]
async fn test_replenish_token() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "simple_token_faucet_replenish_token",
        program_id,
        processor!(process_instruction),
    );

    let faucet_keypair = Keypair::new();
    let admin_keypair = Keypair::new();

    // Initialize faucet state
    let distribution_amount = 1000;

    let rent = Rent::default();
    let account_size = size_of::<FaucetState>();
    let faucet_account_rent = rent.minimum_balance(account_size);

    program_test.add_account(
        faucet_keypair.pubkey(),
        Account {
            lamports: faucet_account_rent + 10_000_000, // Rent + initial balance
            data: vec![0; account_size],
            owner: program_id,
            ..Account::default()
        },
    );

    program_test.add_account(
        admin_keypair.pubkey(),
        Account {
            lamports: 1_000_000_000,
            owner: system_program::id(),
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Initialize the faucet
    let init_instruction = Instruction::new_with_borsh(
        program_id,
        &FaucetInstruction::Initialize {
            distribution_amount,
        },
        vec![
            AccountMeta::new(faucet_keypair.pubkey(), false),
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &admin_keypair], recent_blockhash);

    // Process initialization transaction
    banks_client.process_transaction(transaction).await.unwrap();

    // Get a new recent blockhash
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    // Replenish tokens
    let replenish_amount = 5000;
    let replenish_instruction = Instruction::new_with_borsh(
        program_id,
        &FaucetInstruction::ReplenishTokens { replenish_amount },
        vec![
            AccountMeta::new(faucet_keypair.pubkey(), false),
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction =
        Transaction::new_with_payer(&[replenish_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &admin_keypair], recent_blockhash);

    // Send and confirm transaction
    banks_client.process_transaction(transaction).await.unwrap();

    // Check balances
    let faucet_account = banks_client
        .get_account(faucet_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();

    let admin_account = banks_client
        .get_account(admin_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        faucet_account.lamports,
        faucet_account_rent + 10_000_000 + replenish_amount
    );
    assert_eq!(admin_account.lamports, 1_000_000_000 - replenish_amount);
}
