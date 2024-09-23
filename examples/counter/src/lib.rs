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

    #[derive(Clone, BorshSerialize, BorshDeserialize)]
    pub struct CounterParams {
        pub name: String,
        pub tx_hex: Vec<u8>,
    }

    #[test]
    fn test_deploy_call_counter() {
        setup();

        info!("Starting test_deploy_call_counter");

        // 1. Get program and caller key pairs
        let (program_keypair, program_pubkey) =
            with_secret_key_file(PROGRAM_FILE_PATH).expect("Failed to get program key pair");
        let (caller_keypair, caller_pubkey) =
            with_secret_key_file(CALLER_FILE_PATH).expect("Failed to get caller key pair");

        // 2. Send UTXO for program account
        let (txid, vout) = send_utxo(program_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for program pubkey: {:?}",
            txid,
            vout,
            hex::encode(program_pubkey)
        );

        // 3. Create program account
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

        // 4. Deploy program
        let txids = deploy_program_txs(
            program_keypair.clone(),
            "program/target/sbf-solana-solana/release/counterprogram.so",
        );
        info!("Program deployed with transaction IDs: {:?}", txids);

        // 5. Set program as executable
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

        // 6. Verify program is executable
        assert!(
            read_account_info(NODE1_ADDRESS, program_pubkey.clone())
                .expect("Failed to read account info")
                .is_executable,
            "Program should be marked as executable"
        );

        // 7. Create caller account
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

        // 8. Call the program
        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: program_pubkey.clone(),
                accounts: vec![AccountMeta {
                    pubkey: caller_pubkey.clone(),
                    is_signer: true,
                    is_writable: true,
                }],
                data: borsh::to_vec(&CounterParams {
                    name: "Satoshi".to_string(),
                    tx_hex: vec![], // Empty vector for tx_hex
                })
                .unwrap(),
            },
            vec![caller_keypair],
        )
        .expect("Failed to sign and send program call instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!("Processed transaction for program call: {:?}", processed_tx);

        // 9. Check results
        let caller_account_info = read_account_info(NODE1_ADDRESS, caller_pubkey.clone())
            .expect("Failed to read caller account info");
        info!(
            "Caller account info after program call: {:?}",
            caller_account_info
        );

        // 10. Verify the counter value
        let account_data = caller_account_info.data;
        info!("Account data length: {}", account_data.len());

        if account_data.is_empty() {
            error!("Account data is empty. The program may not have stored any data.");
        } else if account_data.len() < 4 {
            error!(
                "Account data is too short. Expected at least 4 bytes, got {}",
                account_data.len()
            );
        } else {
            let counter_value = u32::from_le_bytes(account_data[0..4].try_into().unwrap());
            info!("Counter value: {}", counter_value);

            if account_data.len() > 4 {
                let message = String::from_utf8_lossy(&account_data[4..]);
                info!("Message: {}", message);

                assert_eq!(counter_value, 1, "Counter should be incremented to 1");
                assert_eq!(
                    message, "Satoshi's counter updated to 1!",
                    "Unexpected message in account data"
                );
            } else {
                warn!("Account data contains only the counter value, no message.");
            }
        }

        info!("test_deploy_call_counter completed");
    }
}
