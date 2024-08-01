/// Running Tests
#[cfg(test)]
mod tests {
    use super::*;
    use bitcoincore_rpc::{Auth, Client};
    use borsh::{BorshDeserialize, BorshSerialize};
    use common::constants::*;
    use common::helper::*;
    use common::models::*;
    use sdk::{Pubkey, UtxoMeta};
    use serial_test::serial;
    use std::thread;
    use std::str::FromStr;

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

    fn assert_send_and_sign_instruction_process(input: AccountInstruction, expected: Account) {
        let expected = borsh::to_vec(&expected).expect("Account should be serializable");

        start_key_exchange();
        start_dkg();

        let rpc = Client::new(
            "https://bitcoin-node.dev.aws.archnetwork.xyz:18443/wallet/testwallet",
            Auth::UserPass(
                "bitcoin".to_string(),
                "428bae8f3c94f8c39c50757fc89c39bc7e6ebc70ebf8f618".to_string(),
            ),
        )
        .unwrap();

        let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();

        let state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone()))
            .expect("read utxo should not fail");

        let instruction_data =
            borsh::to_vec(&input).expect("CreateAccountParams should be serializable");

        let (txid, instruction_hash) = sign_and_send_instruction(
            deployed_program_id.clone(),
            vec![UtxoMeta {
                txid: state_txid.clone(),
                vout: 1,
            }],
            instruction_data,
        )
        .expect("signing and sending a transaction should not fail");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
            .expect("get processed transaction should not fail");
        println!("processed_tx {:?}", processed_tx);

        let state_txid = &processed_tx.bitcoin_txids[&instruction_hash];
        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
            .expect("read utxo should not fail");

        assert_eq!(
            utxo.data,
            expected,
        );
    }

    #[test]
    #[serial]
    fn test_create_account_creates_account() {
        let input = CreateAccountParams {
            id: "1".to_string(),
            name: "Amine".to_string(),
            balance: 32768,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let expected = Account {
            id: "1".to_string(),
            name: "Amine".to_string(),
            balance: 32768,
        };
        assert_send_and_sign_instruction_process(
            AccountInstruction::CreateAccount(input),
            expected,
        );
    }

    #[test]
    #[should_panic]
    #[serial]
    fn test_create_account_panics_on_wrong_account_details() {
        let input = CreateAccountParams {
            id: "1".to_string(),
            name: "Marouane".to_string(),
            balance: 32768,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let expected = Account {
            id: "1".to_string(),
            name: "Amine".to_string(),
            balance: 32768,
        };
        assert_send_and_sign_instruction_process(
            AccountInstruction::CreateAccount(input),
            expected,
        );
    }

    #[test]
    #[serial]
    fn test_deposit_puts_money_in_account() {
        let input = DepositParams {
            account: Account {
                id: "1".to_string(),
                name: "Amine".to_string(),
                balance: 10000,
            },
            value: 1000,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let expected = Account {
            id: "1".to_string(),
            name: "Amine".to_string(),
            balance: 11000,
        };
        assert_send_and_sign_instruction_process(AccountInstruction::Deposit(input), expected);
    }

    #[test]
    #[serial]
    fn test_deposit_does_not_update_balance() {
        let input = DepositParams {
            account: Account {
                id: "1".to_string(),
                name: "Amine".to_string(),
                balance: 10000,
            },
            value: u32::MAX,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let expected = Account {
            id: "1".to_string(),
            name: "Amine".to_string(),
            balance: 10000,
        };
        assert_send_and_sign_instruction_process(AccountInstruction::Deposit(input), expected);
    }

    #[test]
    #[serial]
    fn test_withdrawal_reduces_account_balance_by_value() {
        let input = WithdrawParams {
            account: Account {
                id: "1".to_string(),
                name: "Amine".to_string(),
                balance: 10000,
            },
            value: 4000,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let expected = Account {
            id: "1".to_string(),
            name: "Amine".to_string(),
            balance: 6000,
        };
        assert_send_and_sign_instruction_process(AccountInstruction::Withdraw(input), expected);
    }

    #[test]
    #[serial]
    fn test_withdrawal_does_not_update_balance() {
        let input = WithdrawParams {
            account: Account {
                id: "1".to_string(),
                name: "Amine".to_string(),
                balance: 10000,
            },
            value: 10001,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let expected = Account {
            id: "1".to_string(),
            name: "Amine".to_string(),
            balance: 10000,
        };
        assert_send_and_sign_instruction_process(AccountInstruction::Withdraw(input), expected);
    }
}
