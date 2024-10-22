use arch_program::account::AccountMeta;
use arch_program::instruction::Instruction;
use arch_program::pubkey::Pubkey;
use arch_program::system_instruction::SystemInstruction;
use arch_program::utxo::UtxoMeta;
use bitcoin::key::Keypair;

use bip322::sign_message_bip322;
use bitcoin::XOnlyPublicKey;
use borsh::{BorshDeserialize, BorshSerialize};
use common::constants::{GET_PROCESSED_TRANSACTION, NODE1_ADDRESS};
use common::helper::{
    get_processed_transaction, post_data, process_get_transaction_result, process_result,
    read_account_info, send_utxo, sign_and_send_instruction,
};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;

use anyhow::{anyhow, Result};
use sdk::processed_transaction::{ProcessedTransaction, Status};
use tracing::{debug, error};

pub(crate) fn start_new_counter(
    program_pubkey: &Pubkey,
    step: u16,
    initial_value: u16,
) -> Result<(Pubkey, Keypair)> {
    println!();
    println!("\x1b[1m\x1b[32m===== COUNTER INITIALIZATION ===================================================================================================================================================================\x1b[0m");

    let (account_key_pair, account_pubkey, address) = generate_new_keypair();

    let (txid, vout) = send_utxo(account_pubkey);

    println!(
        "\x1b[32m Step 1/3 Successful :\x1b[0m Account created with address  {}",
        address
    );

    let (txid, _) = sign_and_send_instruction(
        SystemInstruction::new_create_account_instruction(
            hex::decode(txid).unwrap().try_into().unwrap(),
            vout,
            account_pubkey,
        ),
        vec![account_key_pair],
    )
    .expect("signing and sending a transaction should not fail");

    let _processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
        .expect("get processed transaction should not fail");

    assign_ownership_to_program(program_pubkey, account_pubkey, account_key_pair);

    let serialized_counter_input = borsh::to_vec(&CounterInput {
        instruction: CounterInstruction::InitializeCounter(1, 1),
        anchoring: None,
        should_return_err: false,
        should_panic: false,
    })
    .unwrap();

    let (txid, _) = sign_and_send_instruction(
        arch_program::instruction::Instruction {
            program_id: program_pubkey.clone(),
            accounts: vec![AccountMeta {
                pubkey: account_pubkey,
                is_signer: true,
                is_writable: true,
            }],
            data: serialized_counter_input,
        },
        vec![account_key_pair],
    )
    .expect("signing and sending a transaction should not fail");

    let _processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
        .expect("get processed transaction should not fail");

    let account_info = read_account_info(NODE1_ADDRESS, account_pubkey).unwrap();

    let mut account_info_data = account_info.data.as_slice();

    let account_counter = CounterData::deserialize(&mut account_info_data).unwrap();

    if account_counter != CounterData::new(initial_value, step) {
        error!("Account content different from provided initial step and initial value !");

        debug!("Account info found within account {:?}", account_info);

        return Err(anyhow!("Account content after initialization is wrong !"));
    }

    println!("\x1b[32m Step 3/3 Successful :\x1b[0m Counter succesfully initialized \x1b[1m\x1B[34mCounter Data : Step {} ======= Value {}\x1b[0m",account_counter.current_step, account_counter.current_value);

    println!("\x1b[1m\x1b[32m===== COUNTER INITIALIZATION : OK ! ============================================================================================================================================================\x1b[0m");

    Ok((account_pubkey, account_key_pair))
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum CounterInstruction {
    InitializeCounter(u16, u16),
    IncreaseCounter,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct CounterData {
    pub current_value: u16,
    pub current_step: u16,
}

impl CounterData {
    pub fn new(current_value: u16, current_step: u16) -> Self {
        CounterData {
            current_value,
            current_step,
        }
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CounterInput {
    pub instruction: CounterInstruction,
    pub anchoring: Option<(UtxoMeta, Vec<u8>, bool)>,
    pub should_return_err: bool,
    pub should_panic: bool,
}

pub(crate) fn get_counter_increase_instruction(
    program_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    should_return_err: bool,
    should_panic: bool,
    anchoring: Option<(UtxoMeta, Vec<u8>, bool)>,
) -> Instruction {
    let serialized_counter_input = borsh::to_vec(&CounterInput {
        instruction: CounterInstruction::IncreaseCounter,
        anchoring,
        should_return_err,
        should_panic,
    })
    .unwrap();

    Instruction {
        program_id: program_pubkey.clone(),
        accounts: vec![AccountMeta {
            pubkey: account_pubkey.clone(),
            is_signer: true,
            is_writable: true,
        }],
        data: serialized_counter_input,
    }
}
use sdk::arch_program::message::Message;
use sdk::runtime_transaction::RuntimeTransaction;
use sdk::signature::Signature;

use crate::counter_helpers::{assign_ownership_to_program, generate_new_keypair};

pub(crate) fn build_transaction(
    signer_key_pairs: Vec<Keypair>,
    instructions: Vec<Instruction>,
) -> RuntimeTransaction {
    let pubkeys = signer_key_pairs
        .iter()
        .map(|signer| Pubkey::from_slice(&XOnlyPublicKey::from_keypair(signer).0.serialize()))
        .collect::<Vec<Pubkey>>();

    let message = Message {
        signers: pubkeys,
        instructions,
    };

    let digest_slice = hex::decode(message.hash()).expect("hashed message should be decodable");

    let signatures = signer_key_pairs
        .iter()
        .map(|signer| {
            let signature = sign_message_bip322(signer, &digest_slice).to_vec();
            Signature(signature)
        })
        .collect::<Vec<Signature>>();

    let params = RuntimeTransaction {
        version: 0,
        signatures,
        message,
    };

    params
}

pub(crate) fn build_and_send_block(transactions: Vec<RuntimeTransaction>) -> Vec<String> {
    let result: bitcoincore_rpc::jsonrpc::serde_json::Value =
        process_result(post_data(NODE1_ADDRESS, "send_transactions", transactions))
            .expect("send_transaction should not fail");

    let transaction_ids: Vec<String> =
        bitcoincore_rpc::jsonrpc::serde_json::from_value(result).expect("Couldn't decode response");

    return transaction_ids;
}

pub(crate) fn fetch_processed_transactions(
    transaction_ids: Vec<String>,
) -> Result<Vec<ProcessedTransaction>> {
    println!("Fetching {} processed transactions", transaction_ids.len());

    let pb = ProgressBar::new(transaction_ids.len() as u64);

    pb.set_style(ProgressStyle::default_bar()
            .progress_chars("x>-")
            .template("{spinner:.green}[{elapsed_precise:.blue}] {msg:.blue} [{bar:100.green/blue}] {pos}/{len} ({eta})").unwrap());

    pb.set_message("Successfully Processed Deployment Transactions :");

    let mut processed_transactions: Vec<ProcessedTransaction> = vec![];

    for transaction_id in transaction_ids.iter() {
        let mut wait_time = 1;

        let mut processed_tx = process_get_transaction_result(post_data(
            NODE1_ADDRESS,
            GET_PROCESSED_TRANSACTION,
            transaction_id.clone(),
        ))
        .unwrap();

        while processed_tx == Value::Null {
            std::thread::sleep(std::time::Duration::from_secs(wait_time));
            processed_tx = process_get_transaction_result(post_data(
                NODE1_ADDRESS,
                GET_PROCESSED_TRANSACTION,
                transaction_id.clone(),
            ))
            .unwrap();
            wait_time += 1;
            if wait_time >= 60 {
                println!("get_processed_transaction has run for more than 60 seconds");
                return Err(anyhow!("Failed to retrieve processed transaction"));
            }
        }

        while Status::from_value(&processed_tx["status"]) == Some(Status::Processing) {
            println!("Processed transaction is not yet finalized. Retrying...");
            std::thread::sleep(std::time::Duration::from_secs(wait_time));
            processed_tx = process_get_transaction_result(post_data(
                NODE1_ADDRESS,
                GET_PROCESSED_TRANSACTION,
                transaction_id.clone(),
            ))
            .unwrap();
            wait_time += 1;
            if wait_time >= 60 {
                println!("get_processed_transaction has run for more than 60 seconds");
                return Err(anyhow!("Failed to retrieve processed transaction"));
            }
        }
        processed_transactions.push(serde_json::from_value(processed_tx).unwrap());
        pb.inc(1);
        pb.set_message("Processed Deployment Transactions :");
    }
    pb.finish();

    return Ok(processed_transactions);
}
