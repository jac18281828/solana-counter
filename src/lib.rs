use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Entry point for the program
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Solana Counter Program");

    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    // Check that the account is writable
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