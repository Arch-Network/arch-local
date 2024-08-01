#![no_main]
use anyhow::Result;
use bitcoin::consensus;
use bitcoin::Transaction;
use sdk::{entrypoint, Pubkey, UtxoInfo};

pub mod models;
pub mod processor;

use models::AccountInstruction;

#[cfg(target_os = "zkvm")]
entrypoint!(handler);

#[cfg(target_os = "zkvm")]
fn handler(_program_id: &Pubkey, utxos: &[UtxoInfo], instruction_data: &[u8]) -> Result<Vec<u8>> {
    let account_instruction: AccountInstruction = borsh::from_slice(instruction_data)?;
    let tx_hex = account_instruction.tx_hex();
    let account = match account_instruction {
        AccountInstruction::CreateAccount(params) => processor::create_account(params),
        AccountInstruction::Deposit(params) => processor::deposit(params),
        AccountInstruction::Withdraw(params) => processor::withdraw(params),
    };

    for utxo in utxos {
        *utxo.data.borrow_mut() = borsh::to_vec(&account).expect("Account should be serializable");
    }

    let tx: Transaction = consensus::deserialize(&tx_hex).unwrap();
    Ok(consensus::serialize(&tx))
}
