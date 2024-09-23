use arch_program::{
    account::AccountInfo,
    entrypoint,
    helper::get_state_trasition_tx,
    input_to_sign::InputToSign,
    instruction::Instruction,
    msg,
    program::{
        get_account_script_pubkey, get_bitcoin_tx, get_network_xonly_pubkey, invoke,
        next_account_info, set_return_data, set_transaction_to_sign, validate_utxo_ownership,
    },
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::SystemInstruction,
    transaction_to_sign::TransactionToSign,
    utxo::UtxoMeta,
};
use bitcoin::consensus;
use bitcoin::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    msg!("Counter program entered");
    msg!("Program ID: {:?}", program_id);
    msg!("Number of accounts: {}", accounts.len());

    if accounts.len() != 1 {
        msg!("Error: Invalid number of accounts");
        return Err(ProgramError::Custom(501));
    }

    let account_iter = &mut accounts.iter();
    let account = next_account_info(account_iter)?;
    msg!("Account pubkey: {:?}", account.key);

    let params: CounterParams = borsh::from_slice(instruction_data).map_err(|_| {
        msg!("Error: Failed to deserialize CounterParams");
        ProgramError::Custom(502)
    })?;

    msg!("Successfully deserialized CounterParams");
    msg!("Name from params: {}", params.name);

    // Ensure the account has at least 4 bytes
    let minimum_size = 4;
    let current_size = account.data.borrow().len();
    msg!("Current account size: {}", current_size);

    if current_size < minimum_size {
        msg!("Resizing account to minimum size");
        account.realloc(minimum_size, false)?;
    }

    // Read the current counter value or initialize it
    let mut data = account.data.borrow_mut();
    let current_value = if data.len() >= 4 {
        u32::from_le_bytes(data[..4].try_into().unwrap())
    } else {
        0
    };

    msg!("Current counter value: {}", current_value);

    // Increment the counter
    let new_value = current_value + 1;
    msg!("New counter value: {}", new_value);

    // Create the new data string
    let new_data = format!("{}'s counter updated to {}!", params.name, new_value);
    msg!("New data string: {}", new_data);

    // Resize the account if necessary
    let required_size = 4 + new_data.as_bytes().len();
    if data.len() < required_size {
        msg!("Resizing account to {} bytes", required_size);
        drop(data); // Drop the mutable borrow before calling realloc
        account.realloc(required_size, false)?;
        data = account.data.borrow_mut(); // Reborrow after realloc
    }

    // Update the account data
    msg!("Updating account data");
    data[..4].copy_from_slice(&new_value.to_le_bytes());
    data[4..].copy_from_slice(new_data.as_bytes());

    let script_pubkey = get_account_script_pubkey(account.key);
    msg!("script_pubkey {:?}", script_pubkey);
    msg!(
        "Counter updated for {}. New value: {}",
        params.name,
        new_value
    );

    // Process the transaction
    msg!("Processing transaction");
    let tx: Transaction = consensus::deserialize(&params.tx_hex).map_err(|_| {
        msg!("Error: Failed to deserialize transaction");
        ProgramError::Custom(504)
    })?;
    msg!("Transaction successfully deserialized");

    msg!("Counter program completed successfully");
    Ok(())
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CounterParams {
    pub name: String,
    pub tx_hex: Vec<u8>,
}
