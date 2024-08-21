use crate::models::{Account, CreateAccountParams, DepositParams, WithdrawParams};

pub fn create_account(params: CreateAccountParams) -> Account {
    Account {
        id: params.id,
        balance: params.balance,
        name: params.name,
    }
}

pub fn deposit(mut params: DepositParams) -> Account {
    let balance = params.account.balance;
    let new_balance = balance.checked_add(params.value);
    params.account.balance = match new_balance {
        Some(new_balance) => new_balance,
        None => balance,
    };
    params.account
}

pub fn withdraw(mut params: WithdrawParams) -> Account {
    if params.account.balance < params.value {
        return params.account;
    }
    params.account.balance -= params.value;
    params.account
}
