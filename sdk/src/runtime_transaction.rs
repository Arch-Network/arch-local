use sha256::digest;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::{Signature, Message};

pub const RUNTIME_TX_SIZE_LIMIT: u16 = 1024;

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct RuntimeTransaction {
    pub version: u32,
    pub signatures: Vec<Signature>,
    pub message: Message,
}

impl RuntimeTransaction {
    pub fn txid(&self) -> Result<String> {
        Ok(digest(digest(borsh::to_vec(self)?)))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        Ok(borsh::to_vec(self)?)
    }

    pub fn from_vec(data: &[u8]) -> Result<Self> {
        Ok(borsh::from_slice(data)?)
    }

    pub fn check_tx_size_limit(&self) -> anyhow::Result<()> {
        let serialized_tx = self
            .to_vec()
            .map_err(|e| anyhow::anyhow!("runtime tx couldn't be serialized: {:?}", e))?;
        if serialized_tx.len() > RUNTIME_TX_SIZE_LIMIT.into() {
            Err(anyhow::anyhow!("runtime tx size exceeds RUNTIME_TX_SIZE_LIMIT"))
        } else {
            Ok(())
        }
    }
}