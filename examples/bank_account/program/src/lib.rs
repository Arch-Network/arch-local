use arch_program::{
    account::AccountInfo, entrypoint, msg, program::next_account_info, program_error::ProgramError,
    pubkey::Pubkey,
};
use bitcoin::consensus;
use bitcoin::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};
pub mod models;
pub mod processor;
use models::{AccountInstruction, BankAccount, CreateAccountParams, DepositParams, WithdrawParams};

entrypoint!(process_instruction);

// Helper function to convert std::io::Error to ProgramError
fn io_error_to_program_error(e: std::io::Error) -> ProgramError {
    ProgramError::Custom(e.raw_os_error().unwrap_or(0) as u32)
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    msg!("Bank Account program entered");

    // Verify we have the correct number of accounts
    if accounts.len() != 1 {
        return Err(ProgramError::InvalidAccountData);
    }

    let account_iter = &mut accounts.iter();
    let account = next_account_info(account_iter)?;
    msg!("Account pubkey: {:?}", account.key);

    // Deserialize the instruction
    let instruction: AccountInstruction = borsh::from_slice(instruction_data).map_err(|e| {
        msg!("Failed to deserialize instruction: {}", e);
        ProgramError::InvalidInstructionData
    })?;

    // Process the instruction
    match instruction {
        AccountInstruction::CreateAccount(params) => {
            // if account.owner != &Pubkey::system_program() {
            //     return Err(ProgramError::InvalidAccountData);
            // }

            // Debug params
            msg!("Creating account with params: {:?}", params);

            // Create a new BankAccount
            let bank_account = BankAccount {
                id: params.id,
                name: params.name,
                balance: params.balance,
            };

            let serialized_data = borsh::to_vec(&bank_account).map_err(|e| {
                msg!("Failed to serialize account data: {}", e);
                ProgramError::AccountDataTooSmall
            })?;

            // Check if the account has enough space
            if account.data_len() < serialized_data.len() {
                msg!("Account data too small");
                return Err(ProgramError::AccountDataTooSmall);
            }

            account.data.borrow_mut()[..serialized_data.len()].copy_from_slice(&serialized_data);
        }
        AccountInstruction::Deposit(params) => {
            // Deserialize the existing BankAccount from AccountInfo's data
            let mut bank_account: BankAccount =
                borsh::from_slice(&account.data.borrow()).map_err(|e| {
                    msg!("Failed to deserialize account data: {}", e);
                    ProgramError::InvalidAccountData
                })?;

            // Process deposit
            bank_account.balance = bank_account
                .balance
                .checked_add(params.value)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            // Serialize and store the updated BankAccount
            borsh::to_writer(&mut *account.data.borrow_mut(), &bank_account).map_err(|e| {
                msg!("Failed to serialize account data: {}", e);
                ProgramError::AccountDataTooSmall
            })?;
        }
        AccountInstruction::Withdraw(params) => {
            // Deserialize the existing BankAccount from AccountInfo's data
            let mut bank_account: BankAccount =
                borsh::from_slice(&account.data.borrow()).map_err(|e| {
                    msg!("Failed to deserialize account data: {}", e);
                    ProgramError::InvalidAccountData
                })?;

            // Process withdrawal
            bank_account.balance = bank_account
                .balance
                .checked_sub(params.value)
                .ok_or(ProgramError::InsufficientFunds)?;

            // Serialize and store the updated BankAccount
            borsh::to_writer(&mut *account.data.borrow_mut(), &bank_account).map_err(|e| {
                msg!("Failed to serialize account data: {}", e);
                ProgramError::AccountDataTooSmall
            })?;
        }
    }

    msg!("Instruction processed successfully");
    Ok(())
}
