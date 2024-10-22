use arch_program::{
    account::AccountInfo,
    account::AccountMeta,
    entrypoint,
    helper::add_state_transition,
    input_to_sign::InputToSign,
    instruction::Instruction,
    msg,
    program::{
        get_account_script_pubkey, get_bitcoin_block_height, invoke, next_account_info,
        set_transaction_to_sign,
    },
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::SystemInstruction,
    transaction_to_sign::TransactionToSign,
    utxo::UtxoMeta,
    bitcoin::{self, absolute::LockTime, transaction::Version, Transaction}
};
use borsh::{BorshDeserialize, BorshSerialize};

entrypoint!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let account_iter = &mut accounts.iter();
    let account = next_account_info(account_iter)?;

    let data_len = account
        .data
        .try_borrow()
        .map_err(|_e| ProgramError::AccountBorrowFailed)?
        .len();

    let counter_input: CounterInput =
        borsh::from_slice(instruction_data).map_err(|_e| ProgramError::InvalidArgument)?;

    let instruction = counter_input.instruction.clone();

    match instruction {
        CounterInstruction::InitializeCounter(initial_value, step) => {
            if data_len > 0 {
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            let new_counter_data = CounterData::new(initial_value, step);

            msg!(&format!("INITIALIZING COUNTER ! {:?}", new_counter_data));

            let serialized_counter_data = borsh::to_vec(&new_counter_data)
                .map_err(|e| ProgramError::BorshIoError(e.to_string()))?;

            if serialized_counter_data.len() > data_len {
                account.realloc(serialized_counter_data.len(), true)?;
            }

            account
                .data
                .try_borrow_mut()
                .map_err(|_e| ProgramError::AccountBorrowFailed)?
                .copy_from_slice(&serialized_counter_data);

            msg!(&format!("Mutatit memory ! {:?}",serialized_counter_data.len()));

        }
        CounterInstruction::IncreaseCounter => {
            if data_len == 0 {
                return Err(ProgramError::UninitializedAccount);
            }

            let serialized_current_counter_data = account
                .data
                .try_borrow()
                .map_err(|_e| ProgramError::AccountBorrowFailed)?;

            let counter_data: CounterData = borsh::from_slice(&serialized_current_counter_data)
                .map_err(|_e| ProgramError::InvalidAccountData)?;


            let new_counter_data = CounterData::new(counter_data.current_value + counter_data.current_step, counter_data.current_step);

            let new_data = borsh::to_vec(&new_counter_data).unwrap();

            msg!(&format!("Increasing counter ! {:?} new data {:?}", counter_data,new_data));

            if new_data.len() > data_len {
                account.realloc(new_data.len(), true)?;
            }
            
            msg!(&format!("Reallocated memory ! {:?}",new_data.len()));
            drop(serialized_current_counter_data);
            account
                .data
                .try_borrow_mut()
                .unwrap()
                .copy_from_slice(&new_data);

            msg!("Successfully increased counter !");
        },
    }

    if counter_input.anchoring.is_some(){
        let (utxo, serialized_tx, anchoring_should_fail) = counter_input.anchoring.unwrap();

        let fees_tx: Transaction = bitcoin::consensus::deserialize(&serialized_tx).unwrap();

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![],
        };

        add_state_transition(&mut tx, account);
        
        let index = 0;

        if !anchoring_should_fail {
            tx.input.push(fees_tx.input[0].clone());
        }
        let tx_to_sign = TransactionToSign {
            tx_bytes: &bitcoin::consensus::serialize(&tx),
            inputs_to_sign: &[InputToSign {
                index,
                signer: account.key.clone(),
            }],
        };
    
        msg!("Transaction to sign {:?}", tx_to_sign);
    
        set_transaction_to_sign(accounts, tx_to_sign)?
    }

    if counter_input.should_panic{
        panic!("PANICKED BY REQUEST");
    }

    if counter_input.should_return_err{
        return Err(ProgramError::Custom(1))
    }
    
    Ok(())
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum CounterInstruction {
    InitializeCounter(u16, u16),
    IncreaseCounter,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CounterInput {
    pub instruction: CounterInstruction,
    pub anchoring: Option<(UtxoMeta, Vec<u8>, bool)>,
    pub should_return_err: bool,
    pub should_panic: bool,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CounterData {
    current_value: u16,
    current_step: u16,
}

impl CounterData {
    pub fn new(current_value: u16, current_step: u16) -> Self {
        CounterData {
            current_value,
            current_step,
        }
    }
}
