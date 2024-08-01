use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub balance: u32,
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub enum AccountInstruction {
    CreateAccount(CreateAccountParams),
    Deposit(DepositParams),
    Withdraw(WithdrawParams),
}

impl AccountInstruction {
    pub fn tx_hex(&self) -> Vec<u8> {
        match self {
            AccountInstruction::CreateAccount(inner) => inner.tx_hex.clone(),
            AccountInstruction::Deposit(inner) => inner.tx_hex.clone(),
            AccountInstruction::Withdraw(inner) => inner.tx_hex.clone(),
        }
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct CreateAccountParams {
    pub id: String,
    pub name: String,
    pub balance: u32,
    pub tx_hex: Vec<u8>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct DepositParams {
    pub account: Account,
    pub value: u32,
    pub tx_hex: Vec<u8>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct WithdrawParams {
    pub account: Account,
    pub value: u32,
    pub tx_hex: Vec<u8>,
}
