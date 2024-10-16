use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Vault {
    pub owner: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}

impl Vault {
    pub fn new(owner: Pubkey, token_a: Pubkey, token_b: Pubkey) -> Self {
        Vault {
            owner,
            token_a,
            token_b,
            token_a_amount: 0,
            token_b_amount: 0,
        }
    }
}

