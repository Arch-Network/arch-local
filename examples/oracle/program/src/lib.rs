use arch_program::{
    account::AccountInfo,
    entrypoint,
    helper::get_state_transition_tx,
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

entrypoint!(update_data);
pub fn update_data(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let account_iter = &mut accounts.iter();
    let oracle_account = next_account_info(account_iter)?;

    assert!(oracle_account.is_signer);
    //assert!(oracle_account.key.clone() == Pubkey::from_slice(&[0; 32]));
    assert_eq!(instruction_data.len(), 8);

    let data_len = oracle_account.data.try_borrow().unwrap().len();
    if instruction_data.len() > data_len {
        oracle_account.realloc(instruction_data.len(), true)?;
    }

    oracle_account
        .data
        .try_borrow_mut()
        .unwrap()
        .copy_from_slice(instruction_data);

    msg!("updated");

    Ok(())
}
