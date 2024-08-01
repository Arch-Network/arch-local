#![no_main]
use anyhow::Result;
use bitcoin::consensus;
use bitcoin::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};
use sdk::{entrypoint, Pubkey, UtxoInfo};

#[cfg(target_os = "zkvm")]
entrypoint!(handler);

#[cfg(target_os = "zkvm")]
fn handler(_program_id: &Pubkey, utxos: &[UtxoInfo], instruction_data: &[u8]) -> Result<Vec<u8>> {
    let params: CounterParams = borsh::from_slice(instruction_data)?;

    for utxo in utxos {
        *utxo.data.borrow_mut() = format!("{}'s counter updated to {}!", params.name, params.value + 1).as_str().as_bytes().to_vec();
    }

    let mut tx: Transaction = consensus::deserialize(&params.tx_hex).unwrap();
    Ok(consensus::serialize(&tx))
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct CounterParams {
    pub name: String,
    pub value: u32,
    pub tx_hex: Vec<u8>,
}