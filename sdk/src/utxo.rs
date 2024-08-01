use std::cell::RefCell;
use serde::{Deserialize, Serialize};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::Pubkey;

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct UtxoMeta {
    pub txid: String,
    pub vout: u32,
}

impl UtxoMeta {
    pub fn id(&self) -> String {
        format!("{}:{}", self.txid, self.vout)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UtxoInfo {
    pub txid: String,
    pub vout: u32,
    pub authority: RefCell<Pubkey>,
    pub data: RefCell<Vec<u8>>,
}

impl UtxoInfo {
    pub fn id(&self) -> String {
        format!("{}:{}", self.txid, self.vout)
    }
}