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

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    msg!("Bank Account program entered");

    if accounts.len() != 1 {
        msg!("Error: Invalid number of accounts");
        return Err(ProgramError::InvalidAccountData);
    }

    let account_iter = &mut accounts.iter();
    let account = next_account_info(account_iter)?;
    msg!("Account pubkey: {:?}", account.key);

    let instruction: AccountInstruction = borsh::from_slice(instruction_data).map_err(|e| {
        msg!("Failed to deserialize instruction: {}", e);
        ProgramError::InvalidInstructionData
    })?;

    match instruction {
        AccountInstruction::CreateAccount(params) => create_account(account, params),
        AccountInstruction::Deposit(params) => deposit(account, params),
        AccountInstruction::Withdraw(params) => withdraw(account, params),
    }
}

fn io_error_to_program_error(e: std::io::Error) -> ProgramError {
    msg!("Deserialization error: {}", e);
    ProgramError::InvalidAccountData
}

fn create_account(account: &AccountInfo, params: CreateAccountParams) -> Result<(), ProgramError> {
    msg!("Creating account with params: {:?}", params);

    let bank_account = BankAccount {
        id: params.id,
        name: params.name,
        balance: params.balance,
    };

    let serialized_data = borsh::to_vec(&bank_account).map_err(|e| {
        msg!("Failed to serialize account data: {}", e);
        ProgramError::InvalidAccountData
    })?;

    msg!("Serialized data length: {}", serialized_data.len());
    msg!("Current account data length: {}", account.data_len());

    ensure_account_size(account, serialized_data.len())?;

    account.data.borrow_mut()[..serialized_data.len()].copy_from_slice(&serialized_data);
    msg!("Account created successfully");
    Ok(())
}

fn deposit(account: &AccountInfo, params: DepositParams) -> Result<(), ProgramError> {
    let mut bank_account: BankAccount = deserialize_bank_account(account)?;

    bank_account.balance = bank_account
        .balance
        .checked_add(params.value)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    serialize_and_store_bank_account(account, &bank_account)?;
    msg!("Deposit processed successfully");
    Ok(())
}

fn withdraw(account: &AccountInfo, params: WithdrawParams) -> Result<(), ProgramError> {
    let mut bank_account: BankAccount = deserialize_bank_account(account)?;

    bank_account.balance = bank_account
        .balance
        .checked_sub(params.value)
        .ok_or(ProgramError::InsufficientFunds)?;

    serialize_and_store_bank_account(account, &bank_account)?;
    msg!("Withdrawal processed successfully");
    Ok(())
}

fn deserialize_bank_account(account: &AccountInfo) -> Result<BankAccount, ProgramError> {
    let data = account.data.borrow();
    msg!("Raw account data ({}): {:?}", data.len(), data);

    if data.is_empty() {
        msg!("Account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut slice: &[u8] = &data;

    let id = String::deserialize(&mut slice).map_err(io_error_to_program_error)?;
    msg!("Deserialized id: {}", id);

    let name = String::deserialize(&mut slice).map_err(io_error_to_program_error)?;
    msg!("Deserialized name: {}", name);

    let balance = u32::deserialize(&mut slice).map_err(io_error_to_program_error)?;
    msg!("Deserialized balance: {}", balance);

    // Log any remaining bytes, but don't treat it as an error
    if !slice.is_empty() {
        msg!("{} bytes remaining after deserialization", slice.len());
    }

    Ok(BankAccount { id, name, balance })
}

fn serialize_and_store_bank_account(
    account: &AccountInfo,
    bank_account: &BankAccount,
) -> Result<(), ProgramError> {
    let serialized_data = borsh::to_vec(bank_account).map_err(|e| {
        msg!("Failed to serialize account data: {}", e);
        ProgramError::InvalidAccountData
    })?;

    ensure_account_size(account, serialized_data.len())?;

    account.data.borrow_mut()[..serialized_data.len()].copy_from_slice(&serialized_data);
    Ok(())
}

fn ensure_account_size(account: &AccountInfo, required_size: usize) -> Result<(), ProgramError> {
    let current_size = account.data_len();
    msg!(
        "Current account size: {}, Required size: {}",
        current_size,
        required_size
    );

    if current_size < required_size {
        msg!(
            "Resizing account from {} to {} bytes",
            current_size,
            required_size
        );
        account.realloc(required_size, false)?;
        msg!("Account resized successfully");
    }

    Ok(())
}
