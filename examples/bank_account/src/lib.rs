use arch_program::{
    account::AccountInfo, instruction::Instruction, pubkey::Pubkey,
    system_instruction::SystemInstruction,
};
use borsh::{BorshDeserialize, BorshSerialize};
use common::helper::{
    get_processed_transaction, read_account_info, send_utxo, sign_and_send_instruction,
    with_secret_key_file,
};
use log::{debug, error, info, warn};

// Structs and enums that mirror those in the on-chain program
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BankAccount {
    pub id: String,
    pub name: String,
    pub balance: u32,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum AccountInstruction {
    CreateAccount(CreateAccountParams),
    Deposit(DepositParams),
    Withdraw(WithdrawParams),
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CreateAccountParams {
    pub id: String,
    pub name: String,
    pub balance: u32,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct DepositParams {
    pub account: BankAccount,
    pub value: u32,
    pub tx_hex: Vec<u8>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct WithdrawParams {
    pub account: BankAccount,
    pub value: u32,
    pub tx_hex: Vec<u8>,
}

// Custom deserialization function to handle padding

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use arch_program::{account::AccountMeta, program_error::ProgramError};
    use common::{
        constants::{CALLER_FILE_PATH, NODE1_ADDRESS, PROGRAM_FILE_PATH},
        helper::deploy_program_txs,
    };
    use env_logger;
    use serial_test::serial;
    use solana_sdk::instruction;

    fn setup() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn deserialize_bank_account(data: &[u8]) -> Result<BankAccount, Box<dyn std::error::Error>> {
        let mut slice = data;
        let id = String::deserialize(&mut slice)?;
        let name = String::deserialize(&mut slice)?;
        let balance = u32::deserialize(&mut slice)?;

        Ok(BankAccount { id, name, balance })
    }

    #[test]
    #[serial]
    fn test_create_account_creates_account() -> Result<(), ProgramError> {
        setup();
        info!("Starting test_create_account_creates_account");

        // Generate program and account keypairs
        let (program_keypair, program_pubkey) =
            with_secret_key_file(PROGRAM_FILE_PATH).expect("Failed to get program key pair");
        info!("Program pubkey: {:?}", program_pubkey);

        let (account_keypair, account_pubkey) =
            with_secret_key_file(CALLER_FILE_PATH).expect("Failed to get caller key pair");
        info!("Account pubkey: {:?}", account_pubkey);

        // Send UTXO to create the account
        let (txid, vout) = send_utxo(program_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for program pubkey: {:?}",
            txid,
            vout,
            hex::encode(program_pubkey)
        );

        // Create the account
        let (txid, instruction_hash) = sign_and_send_instruction(
            SystemInstruction::new_create_account_instruction(
                hex::decode(txid).unwrap().try_into().unwrap(),
                vout,
                program_pubkey.clone(),
            ),
            vec![program_keypair.clone()],
        )
        .expect("Failed to sign and send create account instruction");

        // Wait for the transaction to be processed
        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for account creation: {:?}",
            processed_tx
        );

        // 4. Deploy program
        let txids = deploy_program_txs(
            program_keypair.clone(),
            "program/target/sbf-solana-solana/release/bank_account_program.so",
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

        // Wait for the transaction to be processed
        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for CreateAccount: {:?}",
            processed_tx
        );

        // 7. Create caller account
        let (txid, vout) = send_utxo(account_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for caller pubkey: {:?}",
            txid,
            vout,
            hex::encode(account_pubkey)
        );

        let (txid, instruction_hash) = sign_and_send_instruction(
            SystemInstruction::new_create_account_instruction(
                hex::decode(txid).unwrap().try_into().unwrap(),
                vout,
                account_pubkey.clone(),
            ),
            vec![account_keypair.clone()],
        )
        .expect("Failed to sign and send create caller account instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for caller account creation: {:?}",
            processed_tx
        );

        // 8. Call the program
        let instruction_data =
            borsh::to_vec(&AccountInstruction::CreateAccount(CreateAccountParams {
                id: "1".to_string(),
                name: "Amine".to_string(),
                balance: 32768,
            }))
            .unwrap();

        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: program_pubkey.clone(),
                accounts: vec![AccountMeta {
                    pubkey: account_pubkey.clone(),
                    is_signer: false,
                    is_writable: true,
                }],
                data: instruction_data,
            },
            vec![account_keypair],
        )
        .expect("Failed to sign and send create account instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!("Processed transaction for program call: {:?}", processed_tx);

        // Verify the account data
        let account_info =
            read_account_info(NODE1_ADDRESS, account_pubkey).expect("Failed to read account info");
        debug!("On Chain Account Info: {:?}", account_info);

        match deserialize_bank_account(&account_info.data) {
            Ok(account_data) => {
                info!("Successfully deserialized account data: {:?}", account_data);
                assert_eq!(account_data.id, "1", "Account ID should be 1");
                assert_eq!(account_data.name, "Amine", "Account name should be Amine");
                assert_eq!(
                    account_data.balance, 32768,
                    "Account balance should be 32768"
                );
            }
            Err(e) => {
                info!("Failed to deserialize account data: {:?}", e);
                info!("Raw account data: {:?}", account_info.data);
                return Err(ProgramError::InvalidAccountData);
            }
        }

        info!("test_create_account_creates_account completed successfully");
        Ok(())
    }
}
