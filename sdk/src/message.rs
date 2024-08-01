use sha256::digest;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::{Pubkey, Instruction};

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Message {
    pub signers: Vec<Pubkey>,
    pub instructions: Vec<Instruction>,
}

impl Message {
    pub fn hash(&self) -> Result<String> {
        Ok(digest(digest(match borsh::to_vec(self) {
            Ok(d) => d,
            Err(err) => return Err(anyhow!(err)),
        })))
    }
}