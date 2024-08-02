/// Running Tests
#[cfg(test)]
mod tests {
    use bitcoincore_rpc::{Auth, Client};
    use common::constants::*;
    use sdk::{Pubkey, UtxoMeta};
    use common::helper::*;
    use serial_test::serial;
    use common::models::*;
    use std::thread;
    use std::str::FromStr;
    use borsh::{BorshSerialize, BorshDeserialize};

    /// Represents the parameters for running the Hello World process
    #[derive(Clone, BorshSerialize, BorshDeserialize)]
    pub struct HelloWorldParams {
        name: String,
        tx_hex: Vec<u8>,
    }

    #[test]
    #[serial]
    fn back_2_back() {
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
        println!("utxo {:?}", format!("{}:1", state_txid.clone()));
        println!("read utxo {:?}", read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone())).expect("read utxo should not fail"));
 
        let instruction_data = HelloWorldParams {
            name: "Amine".to_string(),
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("HelloWorldParams should be serializable");

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

        println!("\ngetting new state\n");

        let state_txid = processed_tx.bitcoin_txids[&instruction_hash].clone();
        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
            .expect("read utxo should not fail");

        println!("read utxo {:?}", utxo);

        assert_eq!(
            utxo.data,
            "Hello Amine!".as_bytes().to_vec()
        );

        let instruction_data = HelloWorldParams {
            name: "Marouane".to_string(),
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("HelloWorldParams should be serializable");

        let (txid, instruction_hash) = sign_and_send_instruction(
            deployed_program_id.clone(),
            vec![UtxoMeta {
                txid: state_txid.clone(),
                vout: 0,
            }],
            instruction_data,
        )
        .expect("signing and sending a transaction should not fail");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
            .expect("get processed transaction should not fail");
        println!("processed_tx {:?}", processed_tx);

        let state_txid = processed_tx.bitcoin_txids[&instruction_hash].clone();
        let utxo = read_utxo(
            NODE1_ADDRESS,
            format!("{}:0", state_txid.clone()),
        )
        .expect("read utxo should not fail");

        assert_eq!(
            utxo.data,
            "Hello Marouane!".as_bytes().to_vec()
        );
        
    }

    #[test]
    #[serial]
    fn multiple_instruction_tx() {
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

        let first_state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", first_state_txid.clone()))
            .expect("read utxo should not fail");

        let second_state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", second_state_txid.clone()))
            .expect("read utxo should not fail");

        let instruction_data = HelloWorldParams {
            name: "Amine".to_string(),
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("HelloWorldParams should be serializable");

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

        let state_txid = processed_tx.bitcoin_txids[&instruction_hash].clone();

        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
            .expect("read utxo should not fail");
        assert_eq!(
            utxo.data,
            "Hello Amine!".as_bytes().to_vec()
        );

        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone()))
            .expect("read utxo should not fail");
        assert_eq!(
            utxo.data,
            "Hello Amine!".as_bytes().to_vec(),
            "failed at asset"
        );
    }
}
