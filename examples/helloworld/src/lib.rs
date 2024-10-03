/// Running Tests
#[cfg(test)]
mod tests {
    use arch_program::{
        account::AccountMeta, instruction::Instruction, pubkey::Pubkey,
        system_instruction::SystemInstruction, utxo::UtxoMeta,
    };
    use bitcoincore_rpc::{Auth, Client};
    use borsh::{BorshDeserialize, BorshSerialize};
    use common::constants::*;
    use common::helper::*;
    use common::models::*;
    use serial_test::serial;
    use std::fs;
    use std::str::FromStr;
    use std::thread;

    use env_logger;
    use log::{debug, error, info, warn};

    fn setup() {
        env_logger::init();
    }

    /// Represents the parameters for running the Hello World process
    #[derive(Clone, BorshSerialize, BorshDeserialize)]
    pub struct HelloWorldParams {
        name: String,
        tx_hex: Vec<u8>,
    }

    #[test]
    fn test_deploy_call() {
        setup();

        let hello_name = "Satoshi"; // Name to be used in the Hello World program

        info!("Starting test_deploy_call");
        debug!(
            "u64 10044 in little-endian bytes: {:?}",
            10044_u64.to_le_bytes()
        );
        debug!(
            "u64 10881 in little-endian bytes: {:?}",
            10881_u64.to_le_bytes()
        );

        // 1. Create Bitcoin RPC client
        // let rpc = Client::new(
        //     "https://bitcoin-node.dev.aws.archnetwork.xyz:18443/wallet/testwallet",
        //     Auth::UserPass(
        //         "bitcoin".to_string(),
        //         "428bae8f3c94f8c39c50757fc89c39bc7e6ebc70ebf8f618".to_string(),
        //     ),
        // )
        // .expect("Failed to create Bitcoin RPC client");

        // 2. Get program and caller key pairs
        let (program_keypair, program_pubkey) =
            with_secret_key_file(PROGRAM_FILE_PATH).expect("Failed to get program key pair");
        let (caller_keypair, caller_pubkey) =
            with_secret_key_file(CALLER_FILE_PATH).expect("Failed to get caller key pair");

        // 3. Send UTXO for program account
        let (txid, vout) = send_utxo(program_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for program pubkey: {:?}",
            txid,
            vout,
            hex::encode(program_pubkey)
        );

        // 4. Create program account
        let (txid, instruction_hash) = sign_and_send_instruction(
            SystemInstruction::new_create_account_instruction(
                hex::decode(txid).unwrap().try_into().unwrap(),
                vout,
                program_pubkey.clone(),
            ),
            vec![program_keypair.clone()],
        )
        .expect("Failed to sign and send create account instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for account creation: {:?}",
            processed_tx
        );

        // 5. Deploy program
        let txids = deploy_program_txs(
            program_keypair.clone(),
            "program/target/sbf-solana-solana/release/helloworldprogram.so",
        );
        info!("Program deployed with transaction IDs: {:?}", txids);

        // 6. Set program as executable
        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: Pubkey::system_program(),
                accounts: vec![AccountMeta {
                    pubkey: program_pubkey.clone(),
                    is_signer: true,
                    is_writable: true,
                }],
                data: vec![2],
            },
            vec![program_keypair],
        )
        .expect("Failed to sign and send set executable instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for setting executable: {:?}",
            processed_tx
        );

        // 7. Verify program is executable
        assert!(
            read_account_info(NODE1_ADDRESS, program_pubkey.clone())
                .expect("Failed to read account info")
                .is_executable,
            "Program should be marked as executable"
        );

        // 8. Create caller account
        let (txid, vout) = send_utxo(caller_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for caller pubkey: {:?}",
            txid,
            vout,
            hex::encode(caller_pubkey)
        );

        let (txid, instruction_hash) = sign_and_send_instruction(
            SystemInstruction::new_create_account_instruction(
                hex::decode(txid).unwrap().try_into().unwrap(),
                vout,
                caller_pubkey.clone(),
            ),
            vec![caller_keypair.clone()],
        )
        .expect("Failed to sign and send create caller account instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for caller account creation: {:?}",
            processed_tx
        );

        // 9. Assign ownership of caller account to program

        let mut instruction_data = vec![3];
        instruction_data.extend(program_pubkey.serialize());

        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: Pubkey::system_program(),
                accounts: vec![AccountMeta {
                    pubkey: caller_pubkey.clone(),
                    is_signer: true,
                    is_writable: true,
                }],
                data: instruction_data,
            },
            vec![caller_keypair.clone()],
        )
        .expect("Failed to sign and send Assign ownership of caller account instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for caller account ownership assignment: {:?}",
            processed_tx
        );

        // 10. Verify that the program is owner of caller account
        assert_eq!(
            read_account_info(NODE1_ADDRESS, caller_pubkey.clone())
                .unwrap()
                .owner,
            program_pubkey,
            "Program should be owner of caller account"
        );

        // 12. Call the program again
        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: program_pubkey.clone(),
                accounts: vec![AccountMeta {
                    pubkey: caller_pubkey.clone(),
                    is_signer: true,
                    is_writable: true,
                }],
                data: borsh::to_vec(&HelloWorldParams {
                    name: hello_name.to_string(),
                    tx_hex: hex::decode(prepare_fees()).unwrap(),
                })
                .unwrap(),
            },
            vec![caller_keypair],
        )
        .expect("Failed to sign and send program call instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!("Processed transaction for program call: {:?}", processed_tx);

        // 10. Check results
        let caller_account_info = read_account_info(NODE1_ADDRESS, caller_pubkey.clone())
            .expect("Failed to read caller account info");
        info!(
            "Caller account info after program call: {:?}",
            caller_account_info
        );
        assert_eq!(
            caller_account_info.utxo,
            format!("{}:{}", processed_tx.bitcoin_txids[0], 0),
            "Caller account utxo doesn't match bitcoin txid"
        );

        // 11. Call the program
        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: program_pubkey.clone(),
                accounts: vec![AccountMeta {
                    pubkey: caller_pubkey.clone(),
                    is_signer: true,
                    is_writable: true,
                }],
                data: borsh::to_vec(&HelloWorldParams {
                    name: hello_name.to_string(),
                    tx_hex: hex::decode(prepare_fees()).unwrap(),
                })
                .unwrap(),
            },
            vec![caller_keypair],
        )
        .expect("Failed to sign and send program call instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!("Processed transaction for program call: {:?}", processed_tx);

        // 10. Check results
        let caller_account_info = read_account_info(NODE1_ADDRESS, caller_pubkey.clone())
            .expect("Failed to read caller account info");
        info!(
            "Caller account info after program call: {:?}",
            caller_account_info
        );
        assert_eq!(
            caller_account_info.utxo,
            format!("{}:{}", processed_tx.bitcoin_txids[0], 0),
            "Caller account utxo doesn't match bitcoin txid"
        );

        info!("test_deploy_call completed successfully");
    }

    // #[test]
    // #[serial]
    // fn back_2_back() {
    //     start_key_exchange();
    //     start_dkg();

    //     let rpc = Client::new(
    //         "https://bitcoin-node.dev.aws.archnetwork.xyz:18443/wallet/testwallet",
    //         Auth::UserPass(
    //             "bitcoin".to_string(),
    //             "428bae8f3c94f8c39c50757fc89c39bc7e6ebc70ebf8f618".to_string(),
    //         ),
    //     )
    //     .unwrap();

    //     let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();

    //     let state_txid = send_utxo();
    //     println!("utxo {:?}", format!("{}:1", state_txid.clone()));
    //     println!("read utxo {:?}", read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone())).expect("read utxo should not fail"));

    //     let instruction_data = HelloWorldParams {
    //         name: "Amine".to_string(),
    //         tx_hex: hex::decode(prepare_fees()).unwrap(),
    //     };
    //     let instruction_data =
    //         borsh::to_vec(&instruction_data).expect("HelloWorldParams should be serializable");

    //     let (txid, instruction_hash) = sign_and_send_instruction(
    //         deployed_program_id.clone(),
    //         vec![UtxoMeta {
    //             txid: state_txid.clone(),
    //             vout: 1,
    //         }],
    //         instruction_data,
    //     )
    //     .expect("signing and sending a transaction should not fail");

    //     let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
    //         .expect("get processed transaction should not fail");
    //     println!("processed_tx {:?}", processed_tx);

    //     println!("\ngetting new state\n");

    //     let state_txid = processed_tx.bitcoin_txids[&instruction_hash].clone();
    //     let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
    //         .expect("read utxo should not fail");

    //     println!("read utxo {:?}", utxo);

    //     assert_eq!(
    //         utxo.data,
    //         "Hello Amine!".as_bytes().to_vec()
    //     );

    //     let instruction_data = HelloWorldParams {
    //         name: "Marouane".to_string(),
    //         tx_hex: hex::decode(prepare_fees()).unwrap(),
    //     };
    //     let instruction_data =
    //         borsh::to_vec(&instruction_data).expect("HelloWorldParams should be serializable");

    //     let (txid, instruction_hash) = sign_and_send_instruction(
    //         deployed_program_id.clone(),
    //         vec![UtxoMeta {
    //             txid: state_txid.clone(),
    //             vout: 0,
    //         }],
    //         instruction_data,
    //     )
    //     .expect("signing and sending a transaction should not fail");

    //     let processed_tx = get_processed_transaction("http://127.0.0.1:9001/", txid)
    //         .expect("get processed transaction should not fail");
    //     println!("processed_tx {:?}", processed_tx);

    //     let state_txid = processed_tx.bitcoin_txids[&instruction_hash].clone();
    //     let utxo = read_utxo(
    //         "http://127.0.0.1:9001/",
    //         format!("{}:0", state_txid.clone()),
    //     )
    //     .expect("read utxo should not fail");

    //     assert_eq!(
    //         utxo.data,
    //         "Hello Marouane!".as_bytes().to_vec()
    //     );

    // }

    // #[test]
    // #[serial]
    // fn multiple_instruction_tx() {
    //     start_key_exchange();
    //     start_dkg();

    //     let rpc = Client::new(
    //         "https://bitcoin-node.dev.aws.archnetwork.xyz:18443/wallet/testwallet",
    //         Auth::UserPass(
    //             "bitcoin".to_string(),
    //             "428bae8f3c94f8c39c50757fc89c39bc7e6ebc70ebf8f618".to_string(),
    //         ),
    //     )
    //     .unwrap();

    //     let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();

    //     let first_state_txid = send_utxo();
    //     read_utxo(NODE1_ADDRESS, format!("{}:1", first_state_txid.clone()))
    //         .expect("read utxo should not fail");

    //     let second_state_txid = send_utxo();
    //     read_utxo(NODE1_ADDRESS, format!("{}:1", second_state_txid.clone()))
    //         .expect("read utxo should not fail");

    //     let instruction_data = HelloWorldParams {
    //         name: "Amine".to_string(),
    //         tx_hex: hex::decode(prepare_fees()).unwrap(),
    //     };
    //     let instruction_data =
    //         borsh::to_vec(&instruction_data).expect("HelloWorldParams should be serializable");

    //     let (txid, instruction_hash) = sign_and_send_instruction(
    //         deployed_program_id.clone(),
    //         vec![
    //             UtxoMeta {
    //                 txid: first_state_txid.clone(),
    //                 vout: 1,
    //             },
    //             UtxoMeta {
    //                 txid: second_state_txid.clone(),
    //                 vout: 1,
    //             },
    //         ],
    //         instruction_data,
    //     )
    //     .expect("signing and sending transaction should not fail");

    //     let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
    //         .expect("get processed transaction should not fail");
    //     println!("processed_tx {:?}", processed_tx);

    //     let state_txid = processed_tx.bitcoin_txids[&instruction_hash].clone();

    //     let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
    //         .expect("read utxo should not fail");
    //     assert_eq!(
    //         utxo.data,
    //         "Hello Amine!".as_bytes().to_vec()
    //     );

    //     let utxo = read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone()))
    //         .expect("read utxo should not fail");
    //     assert_eq!(
    //         utxo.data,
    //         "Hello Amine!".as_bytes().to_vec(),
    //         "failed at asset"
    //     );
    // }

    // #[ignore]
    // #[test]
    // fn test_node_consistency() {
    //     let bitcoin_rpc_info = BitcoinRpcInfo {
    //         endpoint: "bitcoin-node.dev.aws.archnetwork.xyz".to_string(),
    //         port: 18443,
    //         username: "bitcoin".to_string(),
    //         password: "428bae8f3c94f8c39c50757fc89c39bc7e6ebc70ebf8f618".to_string(),
    //     };

    //     let mut boot_node = start_boot_node(
    //         9001,
    //         "http://127.0.0.1:9001,http://127.0.0.1:9002,http://127.0.0.1:9003",
    //         &bitcoin_rpc_info,
    //     );
    //     println!("Boot node started.");

    //     let mut node_1 = start_node(9002, &bitcoin_rpc_info);
    //     println!("Node 1 started.");

    //     let mut node_2 = start_node(9003, &bitcoin_rpc_info);
    //     println!("Node 2 started.");

    //     println!("Waiting for nodes to boot up...");
    //     thread::sleep(std::time::Duration::from_secs(60));

    //     // If nodes do not start at this point then you will get orphaned processes.
    //     println!("Starting DKG...");
    //     start_dkg();

    //     println!("Waiting for DKG to finish...");
    //     thread::sleep(std::time::Duration::from_secs(30));

    //     println!("Killing node 2...");
    //     node_2.kill().expect("Failed to kill node 2 process");

    //     println!("Creating Bitcoin RPC Client...");
    //     let rpc = Client::new(
    //         "https://bitcoin-node.dev.aws.archnetwork.xyz:18443/wallet/testwallet",
    //         Auth::UserPass(
    //             "bitcoin".to_string(),
    //             "428bae8f3c94f8c39c50757fc89c39bc7e6ebc70ebf8f618".to_string(),
    //         ),
    //     )
    //     .unwrap();

    //     println!("Deploying Program...");
    //     let program_id = deploy_program();
    //     let deployed_program_id = Pubkey::from_str(&program_id.clone()).unwrap();

    //     println!("Sending UTXO...");
    //     let state_txid = send_utxo();

    //     println!("Assigning UTXO...");
    //     read_utxo(
    //         "http://127.0.0.1:9001/",
    //         format!("{}:1", state_txid.clone()),
    //     ).expect("read utxo should not fail");

    //     let instruction_data = HelloWorldParams {
    //         name: "Bartol".to_string(),
    //         tx_hex: hex::decode(prepare_fees()).unwrap(),
    //     };
    //     let instruction_data =
    //         borsh::to_vec(&instruction_data).expect("HelloWorldParams should be serializable");

    //     let (txid, instruction_hash) = sign_and_send_instruction(
    //         deployed_program_id.clone(),
    //         vec![UtxoMeta {
    //             txid: state_txid.clone(),
    //             vout: 1,
    //         }],
    //         instruction_data,
    //     )
    //     .expect("signing and sending transaction should not fail");

    //     println!("Waiting for transaction to be processed...");
    //     thread::sleep(std::time::Duration::from_secs(30));

    //     let processed_tx_1 =
    //         get_processed_transaction("http://127.0.0.1:9001/", txid.clone()).unwrap();

    //     let mut node_2 = start_node(9003, &bitcoin_rpc_info);
    //     println!("Starting Node 2 again...");

    //     thread::sleep(std::time::Duration::from_secs(60));

    //     println!("Querying processed tx from Node 2...");
    //     let processed_tx_2 =
    //         get_processed_transaction("http://127.0.0.1:9003/", txid.clone()).unwrap();

    //     let utxo_node_1 = read_utxo(
    //         "http://127.0.0.1:9001/",
    //         format!("{}:1", state_txid.clone()),
    //     )
    //     .unwrap();
    //     let utxo_node_2 = read_utxo(
    //         "http://127.0.0.1:9002/",
    //         format!("{}:1", state_txid.clone()),
    //     )
    //     .unwrap();
    //     let utxo_node_3 = read_utxo(
    //         "http://127.0.0.1:9003/",
    //         format!("{}:1", state_txid.clone()),
    //     )
    //     .unwrap();

    //     println!("Getting program from boot node...");
    //     let program_node_1 = get_program("http://127.0.0.1:9001/", program_id.clone());
    //     println!("Success getting program from boot node!");

    //     println!("Getting program from node 2...");
    //     let program_node_2 = get_program("http://127.0.0.1:9002/", program_id.clone());
    //     println!("Success getting program from node 2!");

    //     println!("Getting program from node 3...");
    //     let program_node_3 = get_program("http://127.0.0.1:9003/", program_id.clone());
    //     println!("Success getting program from node 2!");

    //     println!("Killing boot node...");
    //     boot_node.kill().expect("Failed to kill boot node process");
    //     boot_node.wait().expect("Failed to wait boot node process");

    //     println!("Killing node 1...");
    //     node_1.kill().expect("Failed to kill node 1 process");
    //     node_1.wait().expect("Failed to wait node 1 process");

    //     println!("Killing node 2...");
    //     node_2.kill().expect("Failed to kill node 2 process");
    //     node_2.wait().expect("Failed to wait node 2 process");

    //     println!("Check that both nodes have the same Processed Tx");
    //     assert_eq!(processed_tx_1.to_vec().unwrap(), processed_tx_2.to_vec().unwrap());

    //     println!("Check that all nodes have the same UTXO");
    //     assert_eq!(utxo_node_1, utxo_node_2);
    //     assert_eq!(utxo_node_2, utxo_node_3);
    // }
}
