use anyhow::anyhow;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Pubkey([u8; 32]);

impl Pubkey {
    pub fn serialize(&self) -> &[u8] {
        &self.0
    }

    pub fn from_slice(data: &[u8]) -> Self {
        let mut tmp = [0u8; 32];
        tmp[..data.len()].copy_from_slice(data);
        Self(tmp)
    }
}

impl std::fmt::Display for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::str::FromStr for Pubkey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Pubkey::from_slice(&hex::decode(s).map_err(|_| anyhow!("Can't decode pubkey"))?))
    }
}