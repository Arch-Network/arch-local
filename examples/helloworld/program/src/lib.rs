use arch_program::{
    account::{AccountInfo},
    entrypoint,
    instruction::Instruction,
    msg,
    program::{
        invoke, set_return_data, get_bitcoin_tx, 
        validate_utxo_ownership, get_network_xonly_pubkey,
        set_transaction_to_sign, next_account_info,
        get_account_script_pubkey
    },
    helper::get_state_trasition_tx,
    transaction_to_sign::TransactionToSign,
    program_error::ProgramError,
    input_to_sign::InputToSign,
    pubkey::Pubkey,
    utxo::UtxoMeta,
    system_instruction::SystemInstruction,
};
use borsh::{BorshSerialize, BorshDeserialize};

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    if accounts.len() != 1 {
        return Err(ProgramError::Custom(501));
    }

    let account_iter = &mut accounts.iter();
    let account = next_account_info(account_iter)?;

    let params: HelloWorldParams = borsh::from_slice(instruction_data).unwrap();

    let new_data = format!("Hello {}", params.name);

    let data_len = account.data.try_borrow().unwrap().len();
    if new_data.as_bytes().len() > data_len {
        account.realloc(new_data.len(), true)?;
    }
    
    let script_pubkey = get_account_script_pubkey(account.key);
    msg!("script_pubkey {:?}", script_pubkey);

    account.data.try_borrow_mut().unwrap().copy_from_slice(new_data.as_bytes());

    msg!("hello");

    Ok(())
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct HelloWorldParams {
    pub name: String,
    pub tx_hex: Vec<u8>,
}