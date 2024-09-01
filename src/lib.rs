use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use instructions::FaucetInstruction;
use state::FaucetState;

pub mod instructions;
pub mod state;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = FaucetInstruction::unpack(instruction_data)?;

    match instruction {
        FaucetInstruction::Initialize {
            distribution_amount,
        } => {
            process_initialize(program_id, accounts, distribution_amount)
                .expect("Error processing Initialize instruction");
        }
        FaucetInstruction::RequestTokens => {
            process_request_tokens(program_id, accounts)
                .expect("Error processing RequestTokens instruction");
        }
        FaucetInstruction::ReplenishTokens { replenish_amount } => {
            process_replenish_tokens(program_id, accounts, replenish_amount)
                .expect("Error processing ReplenishTokens instruction");
        }
    }

    Ok(())
}

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    distribution_amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let faucet_account = next_account_info(accounts_iter)?;
    let admin_account = next_account_info(accounts_iter)?;
    let rent = &Rent::from_account_info(next_account_info(accounts_iter)?)?;

    // Check if the faucet account is the correct account
    if faucet_account.owner != program_id {
        msg!("Faucet account must be owned by the program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check if the admin account is a signer
    if !admin_account.is_signer {
        msg!("Admin account must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !rent.is_exempt(faucet_account.lamports(), faucet_account.data_len()) {
        msg!("Faucet account lamports is below rent-exempt threshold");
        return Err(ProgramError::AccountNotRentExempt);
    }

    let faucet_state = FaucetState {
        admin: *admin_account.key, // * dereferences &Pubkey to Pubkey
        distribution_amount,
    };

    faucet_state.serialize(&mut &mut faucet_account.data.borrow_mut()[..])?;

    msg!(
        "Faucet initialized. Admin: {}, Distribution Amount: {}",
        faucet_state.admin,
        faucet_state.distribution_amount
    );

    Ok(())
}

pub fn process_request_tokens(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    if accounts.len() != 3 {
        msg!("Incorrect number of accounts");
        return Err(ProgramError::InvalidAccountData);
    }

    let accounts_iter = &mut accounts.iter();

    let faucet_account = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;

    // Check if the faucet account is the correct account
    if faucet_account.owner != program_id {
        msg!("Faucet account must be owned by the program");
        return Err(ProgramError::IncorrectProgramId);
    }

    let faucet_state = FaucetState::try_from_slice(&faucet_account.data.borrow())
        .expect("Failed to deserialize FaucetState");

    let transfer_amount = faucet_state.distribution_amount;

    **faucet_account.try_borrow_mut_lamports()? -= transfer_amount;
    **user_account.try_borrow_mut_lamports()? += transfer_amount;

    msg!(
        "Transferred {} lamports to {}",
        transfer_amount,
        user_account.key
    );

    Ok(())
}

fn process_replenish_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    replenish_amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let faucet_account = next_account_info(accounts_iter)?;
    let admin_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if faucet_account.owner != program_id {
        msg!("Faucet account must be owned by the program");
        return Err(ProgramError::IncorrectProgramId);
    }

    if !admin_account.is_signer {
        msg!("Admin account must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let faucet_state = FaucetState::try_from_slice(&mut &faucet_account.data.borrow())?;

    if faucet_state.admin != *admin_account.key {
        msg!("Admin account must be the faucet admin");
        return Err(ProgramError::InvalidAccountData);
    }

    // Create the transfer instruction
    let transfer_instruction =
        system_instruction::transfer(admin_account.key, faucet_account.key, replenish_amount);

    // Invoke the transfer instruction
    solana_program::program::invoke(
        &transfer_instruction,
        &[
            admin_account.clone(),
            faucet_account.clone(),
            system_program.clone(),
        ],
    )?;

    msg!(
        "Allocated {} lamports to {}",
        replenish_amount,
        faucet_account.key,
    );

    Ok(())
}
