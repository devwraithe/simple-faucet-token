use solana_program::account_info::next_account_info;
use solana_program::program::invoke;
use solana_program::{account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey, system_instruction, system_program};
use solana_program::program_error::ProgramError;

use crate::instructions::FaucetInstruction;

mod instructions;

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = FaucetInstruction::unpack(instruction_data)?;

    match instruction {
        FaucetInstruction::RequestTokens => {
            if accounts.len() != 3 {
                msg!("Incorrect number of accounts");
                return Err(ProgramError::InvalidAccountData.into());
            }

            let accounts_iter = &mut accounts.iter();

            let faucet_account = next_account_info(accounts_iter)?;
            let user_account = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;

            if !faucet_account.is_signer {
                msg!("Faucet account must be the signer");
                return Err(ProgramError::MissingRequiredSignature.into());
            }

            if *system_program.key != system_program::id() {
                msg!("Incorrect system program");
                return Err(ProgramError::InvalidAccountData.into());
            }

            let amount = 100_000;

            if **faucet_account.try_borrow_lamports()? < amount {
                msg!("User account does not have enough lamports");
                return Err(ProgramError::InsufficientFunds.into());
            }

            let ix = system_instruction::transfer(faucet_account.key, user_account.key, amount);

            invoke(
                &ix,
                &[
                    faucet_account.clone(),
                    user_account.clone(),
                    system_program.clone(),
                ],
            )?;

            msg!("Transferred {} lamports to {}", amount, user_account.key);
        }
    }

    Ok(())
}