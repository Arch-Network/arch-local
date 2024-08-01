use sha256::digest;
use serde::{Deserialize, Serialize};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::Pubkey;
use crate::UtxoMeta;

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Instruction {
    pub program_id: Pubkey,
    pub utxos: Vec<UtxoMeta>,
    pub data: Vec<u8>,
}

impl Instruction {
    pub fn hash(&self) -> anyhow::Result<String> {
        Ok(digest(digest(match borsh::to_vec(self) {
            Ok(d) => d,
            Err(err) => return Err(anyhow::anyhow!(err)),
        })))
    }
}