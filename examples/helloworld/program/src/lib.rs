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
use borsh::{BorshDeserialize, BorshSerialize};

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

    account
        .data
        .try_borrow_mut()
        .unwrap()
        .copy_from_slice(new_data.as_bytes());

    msg!("Hello, {}!", params.name);

    Ok(())
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct HelloWorldParams {
    pub name: String,
    pub tx_hex: Vec<u8>,
}
