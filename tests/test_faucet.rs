#[cfg(not(target_arch = "bpf"))]
mod test {
    use solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    };
    use solana_sdk::{
        account::Account,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use solana_program_test::*;
    use simple_token_faucet::process_instruction;

    #[tokio::test]
    async fn test_faucet_distribution() {
        let program_id = Pubkey::new_unique();
        let faucet_keypair = Keypair::new();
        let user_keypair = Keypair::new();

        let mut program_test = ProgramTest::new(
            "simple_token_faucet",
            program_id,
            processor!(process_instruction),
        );

        program_test.add_account(
            faucet_keypair.pubkey(),
            Account {
                lamports: 100_000_000_000,
                owner: program_id,
                ..Account::default()
            }
        );

        program_test.add_account(
            user_keypair.pubkey(),
            Account {
                lamports: 0,
                owner: system_program::id(),
                ..Account::default()
            }
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Create a transaction to send 100 SOL to the user account
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bincode(
                program_id,
                &[0],
                vec![
                    AccountMeta::new(faucet_keypair.pubkey(), true),
                    AccountMeta::new(user_keypair.pubkey(), false),
                    AccountMeta::new(system_program::id(), false),
                ],
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &faucet_keypair], recent_blockhash);

        // Send and confirm transaction
        banks_client.process_transaction(transaction).await.unwrap();

        // Check balances
        let user_account = banks_client.get_account(user_keypair.pubkey()).await.unwrap().unwrap();
        assert_eq!(user_account.lamports, 100_000);
    }
}