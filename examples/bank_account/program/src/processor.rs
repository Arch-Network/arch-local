use crate::models::{BankAccount, CreateAccountParams, DepositParams, WithdrawParams};
use arch_program::{account::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
use borsh::{BorshDeserialize, BorshSerialize};
fn io_error_to_program_error(e: std::io::Error) -> ProgramError {
    ProgramError::Custom(e.raw_os_error().unwrap_or(0) as u32)
}

pub fn create_account(params: CreateAccountParams) -> Result<BankAccount, ProgramError> {
    msg!("Creating account: {}", params.id);
    Ok(BankAccount {
        id: params.id,
        balance: params.balance,
        name: params.name,
    })
}

pub fn deposit(
    account_info: &AccountInfo,
    params: DepositParams,
) -> Result<BankAccount, ProgramError> {
    let mut account: BankAccount =
        borsh::from_slice(&account_info.data.borrow()).map_err(io_error_to_program_error)?;
    account.balance = account
        .balance
        .checked_add(params.value)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok(account)
}

pub fn withdraw(
    account_info: &AccountInfo,
    params: WithdrawParams,
) -> Result<BankAccount, ProgramError> {
    let mut account: BankAccount =
        borsh::from_slice(&account_info.data.borrow()).map_err(io_error_to_program_error)?;
    if account.balance < params.value {
        return Err(ProgramError::InsufficientFunds);
    }
    account.balance = account
        .balance
        .checked_sub(params.value)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok(account)
}
