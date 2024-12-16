use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Entry point
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Solana Program Started");

    // Get the instruction type from the first byte of instruction_data
    let instruction_type = instruction_data
        .get(0)
        .ok_or(ProgramError::InvalidInstructionData)?;

    match instruction_type {
        0 => {
            if instruction_data.len() != 9 {
                msg!("Invalid instruction data length");
                return Err(ProgramError::InvalidInstructionData);
            }
            let instruction_data = &instruction_data[1..];
            initialize_account(program_id, accounts, instruction_data)
        }
        1 => increment_counter(accounts),
        _ => {
            msg!("Invalid instruction type");
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

// Instruction 0: Initialize an account
pub fn initialize_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Get the account to initialize
    let account = next_account_info(accounts_iter)?;

    // Verify ownership
    if account.owner != program_id {
        msg!("Account does not have the correct program ID");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Ensure the account is writable
    if !account.is_writable {
        msg!("Account is not writable");
        return Err(ProgramError::InvalidArgument);
    }

    // Check if the account has already been initialized
    let mut data = account.try_borrow_mut_data()?;
    if data.iter().any(|&x| x != 0) {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Initialize the account with the provided value
    let initial_value = u64::from_le_bytes(
        instruction_data
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    data[..8].copy_from_slice(&initial_value.to_le_bytes());

    msg!(
        "Account initialized successfully with value: {}",
        initial_value
    );
    Ok(())
}

// Instruction 1: Increment the counter
pub fn increment_counter(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Get the account to increment
    let account = next_account_info(accounts_iter)?;

    // Ensure the account is writable
    if !account.is_writable {
        msg!("Account is not writable");
        return Err(ProgramError::InvalidArgument);
    }

    // Increment the counter stored in the account's data
    let mut data = account.try_borrow_mut_data()?;
    if data.len() < 8 {
        msg!("Account data is too small");
        return Err(ProgramError::InvalidAccountData);
    }

    let counter = u64::from_le_bytes(data[..8].try_into().unwrap());
    let new_counter = counter.wrapping_add(1);

    data[..8].copy_from_slice(&new_counter.to_le_bytes());
    msg!("Counter incremented to: {}", new_counter);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use solana_program::clock::Epoch; // Import Pack trait for Rent serialization

    #[test]
    fn test_initialize_account() {
        let program_id = Pubkey::new_unique();
        let key = Pubkey::new_unique();
        let payer_key = Pubkey::new_unique();

        let mut lamports = 1_000_000; // Arbitrary amount for tests
        let mut payer_lamports = lamports;

        let mut data = vec![0; 8];
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::default(),
        );

        let mut payer_data = vec![0; 0];
        let payer_account = AccountInfo::new(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &program_id,
            false,
            Epoch::default(),
        );

        let accounts = vec![account.clone(), payer_account.clone()];
        let instruction_data = [1, 0, 0, 0, 0, 0, 0, 0, 0]; // Initialize with value 1

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert_eq!(result.is_ok(), true, "{:?}", result);

        // Verify account data
        let account_data = account.try_borrow_data().unwrap();
        assert_eq!(
            account_data[..8],
            1u64.to_le_bytes(),
            "data not matching {:?}",
            account_data
        );
    }

    #[test]
    fn test_increment_counter() {
        let program_id = Pubkey::new_unique();
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000;
        let mut data = 1u64.to_le_bytes().to_vec();

        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::default(),
        );

        let accounts = vec![account.clone()];
        let instruction_data = [1]; // Increment instruction

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert!(result.is_ok(), "Failed to increment counter: {:?}", result);
        assert_eq!(
            account.try_borrow_data().unwrap()[..8],
            2u64.to_le_bytes(),
            "Counter was not incremented correctly"
        );
    }
}
