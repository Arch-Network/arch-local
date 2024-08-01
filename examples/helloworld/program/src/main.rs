#![no_main]
use anyhow::Result;
use bitcoin::consensus;
use bitcoin::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};
use sdk::{entrypoint, Pubkey, UtxoInfo};

#[cfg(target_os = "zkvm")]
entrypoint!(handler);

#[cfg(target_os = "zkvm")]
fn handler(program_id: &Pubkey, utxos: &[UtxoInfo], instruction_data: &[u8]) -> Result<Vec<u8>> {
    let params: HelloWorldParams = borsh::from_slice(instruction_data)?;

    for utxo in utxos {
        *utxo.data.borrow_mut() = format!("Hello {}!", params.name)
            .as_str()
            .as_bytes()
            .to_vec();

        *utxo.authority.borrow_mut() = program_id.clone();
    }

    //*utxos[0].data.borrow_mut() = format!("Hello {}!", params.name).as_str().as_bytes().to_vec();

    let mut tx: Transaction = consensus::deserialize(&params.tx_hex).unwrap();
    Ok(consensus::serialize(&tx))
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct HelloWorldParams {
    pub name: String,
    pub tx_hex: Vec<u8>,
}