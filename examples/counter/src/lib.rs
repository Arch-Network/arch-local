/// Running Tests
#[cfg(test)]
mod tests {
    use bitcoincore_rpc::{Auth, Client};
    use borsh::{BorshDeserialize, BorshSerialize};
    use common::constants::*;
    use common::helper::*;
    use sdk::{Pubkey, UtxoMeta};
    use serial_test::serial;
    use std::str::FromStr;

    #[derive(Clone, BorshSerialize, BorshDeserialize)]
    pub struct CounterParams {
        pub name: String,
        pub value: u32,
        pub tx_hex: Vec<u8>,
    }

    #[test]
    #[serial]
    fn back_2_back() {
        start_key_exchange();
        start_dkg();

        let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();

        let state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone()))
            .expect("read utxo should not fail");

        let instruction_data = CounterParams {
            name: "Amine".to_string(),
            value: 1,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("CounterParams should be serializable");

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
            "Amine's counter updated to 2!".as_bytes().to_vec()
        );

        let instruction_data = CounterParams {
            name: "Marouane".to_string(),
            value: 128,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("CounterParams should be serializable");

        let (txid, instruction_hash) = sign_and_send_instruction(
            deployed_program_id.clone(),
            vec![UtxoMeta {
                txid: state_txid.clone(),
                vout: 0,
            }],
            instruction_data,
        )
        .expect("signing and sending a transaction should not fail");

        let processed_tx = get_processed_transaction("http://127.0.0.1:9001/", txid)
            .expect("get processed transaction should not fail");
        println!("processed_tx {:?}", processed_tx);

        let state_txid = &processed_tx.bitcoin_txids[&instruction_hash];

        let utxo = read_utxo(
            "http://127.0.0.1:9001/",
            format!("{}:0", state_txid.clone()),
        )
        .expect("read utxo should not fail");

        assert_eq!(
            utxo.data,
            "Marouane's counter updated to 129!".as_bytes().to_vec()
        );
    }

    #[test]
    #[serial]
    fn multiple_instruction_tx() {
        let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();

        let first_state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", first_state_txid.clone()))
            .expect("read utxo should not fail");

        let second_state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", second_state_txid.clone()))
            .expect("read utxo should not fail");

        let instruction_data = CounterParams {
            name: "Amine".to_string(),
            value: 16,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("CounterParams should be serializable");

        let (txid, instruction_hash) = sign_and_send_instruction(
            deployed_program_id.clone(),
            vec![
                UtxoMeta {
                    txid: first_state_txid.clone(),
                    vout: 1,
                },
                UtxoMeta {
                    txid: second_state_txid.clone(),
                    vout: 1,
                },
            ],
            instruction_data,
        )
        .expect("signing and sending transaction should not fail");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
            .expect("get processed transaction should not fail");
        println!("processed_tx {:?}", processed_tx);

        let state_txid = &processed_tx.bitcoin_txids[&instruction_hash];

        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
            .expect("read utxo should not fail");
        assert_eq!(
            utxo.data,
            "Amine's counter updated to 17!".as_bytes().to_vec()
        );

        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone()))
            .expect("read utxo should not fail");
        assert_eq!(
            utxo.data,
            "Amine's counter updated to 17!".as_bytes().to_vec()
        );
    }
}
