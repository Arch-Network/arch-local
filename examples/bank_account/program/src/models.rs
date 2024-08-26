use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct BankAccount {
    pub id: String,
    pub name: String,
    pub balance: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum AccountInstruction {
    CreateAccount(CreateAccountParams),
    Deposit(DepositParams),
    Withdraw(WithdrawParams),
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CreateAccountParams {
    pub id: String,
    pub name: String,
    pub balance: u32,
    // pub tx_hex: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DepositParams {
    pub value: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct WithdrawParams {
    pub value: u32,
}
