use arch_program::{
    account::AccountInfo, account::AccountMeta, instruction::Instruction,
    program_error::ProgramError, pubkey::Pubkey, system_instruction::SystemInstruction,
};
use bitcoin::secp256k1;
use borsh::{BorshDeserialize, BorshSerialize};
use common::constants::{CALLER_FILE_PATH, NODE1_ADDRESS, PROGRAM_FILE_PATH};
use common::helper::{
    deploy_program_txs, get_processed_transaction, prepare_fees, read_account_info, send_utxo,
    sign_and_send_instruction, with_secret_key_file,
};
use env_logger;
use log::{debug, error, info};
use secp256k1::Keypair;
use serial_test::serial;

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
    // pub tx_hex: Vec<u8>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct WithdrawParams {
    pub account: BankAccount,
    pub value: u32,
    // pub tx_hex: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        program_pubkey: Pubkey,
        account_pubkey: Pubkey,
        program_keypair: Keypair,
        account_keypair: Keypair,
    }

    fn setup() -> TestContext {
        let _ = env_logger::builder().is_test(true).try_init();

        let (program_keypair, program_pubkey) =
            with_secret_key_file(PROGRAM_FILE_PATH).expect("Failed to get program key pair");
        let (account_keypair, account_pubkey) =
            with_secret_key_file(CALLER_FILE_PATH).expect("Failed to get caller key pair");

        info!("Program pubkey: {:?}", program_pubkey);
        info!("Account pubkey: {:?}", account_pubkey);

        TestContext {
            program_pubkey,
            account_pubkey,
            program_keypair,
            account_keypair,
        }
    }

    fn deploy_program(ctx: &TestContext) {
        let (txid, vout) = send_utxo(ctx.program_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for program pubkey: {:?}",
            txid,
            vout,
            hex::encode(ctx.program_pubkey)
        );

        sign_and_send_instruction(
            SystemInstruction::new_create_account_instruction(
                hex::decode(txid).unwrap().try_into().unwrap(),
                vout,
                ctx.program_pubkey.clone(),
            ),
            vec![ctx.program_keypair.clone()],
        )
        .expect("Failed to create program account");

        let txids = deploy_program_txs(
            ctx.program_keypair.clone(),
            "program/target/sbf-solana-solana/release/bank_account_program.so",
        );
        info!("Program deployed with transaction IDs: {:?}", txids);

        sign_and_send_instruction(
            Instruction {
                program_id: Pubkey::system_program(),
                accounts: vec![AccountMeta {
                    pubkey: ctx.program_pubkey.clone(),
                    is_signer: true,
                    is_writable: true,
                }],
                data: vec![2],
            },
            vec![ctx.program_keypair.clone()],
        )
        .expect("Failed to set program as executable");
    }

    fn create_caller_account(ctx: &TestContext) {
        let (txid, vout) = send_utxo(ctx.account_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for caller pubkey: {:?}",
            txid,
            vout,
            hex::encode(ctx.account_pubkey)
        );

        sign_and_send_instruction(
            SystemInstruction::new_create_account_instruction(
                hex::decode(txid).unwrap().try_into().unwrap(),
                vout,
                ctx.account_pubkey.clone(),
            ),
            vec![ctx.account_keypair.clone()],
        )
        .expect("Failed to create caller account");
    }

    fn create_bank_account(
        ctx: &TestContext,
        id: &str,
        name: &str,
        balance: u32,
    ) -> Result<(), ProgramError> {
        let bank_account = BankAccount {
            id: id.to_string(),
            name: name.to_string(),
            balance,
        };
        let required_space = borsh::to_vec(&bank_account).unwrap().len();

        sign_and_send_instruction(
            SystemInstruction::new_extend_bytes_instruction(
                vec![0; required_space],
                ctx.account_pubkey.clone(),
            ),
            vec![ctx.account_keypair.clone()],
        )
        .expect("Failed to extend account space");

        let instruction_data =
            borsh::to_vec(&AccountInstruction::CreateAccount(CreateAccountParams {
                id: id.to_string(),
                name: name.to_string(),
                balance,
            }))
            .unwrap();

        sign_and_send_instruction(
            Instruction {
                program_id: ctx.program_pubkey.clone(),
                accounts: vec![AccountMeta {
                    pubkey: ctx.account_pubkey.clone(),
                    is_signer: false,
                    is_writable: true,
                }],
                data: instruction_data,
            },
            vec![ctx.account_keypair.clone()],
        )
        .expect("Failed to create bank account");

        Ok(())
    }

    fn perform_operation(
        ctx: &TestContext,
        instruction: AccountInstruction,
    ) -> Result<(), ProgramError> {
        let instruction_data = borsh::to_vec(&instruction).unwrap();
        info!(
            "Performing operation with instruction data: {:?}",
            instruction_data
        );

        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: ctx.program_pubkey.clone(),
                accounts: vec![AccountMeta {
                    pubkey: ctx.account_pubkey.clone(),
                    is_signer: false,
                    is_writable: true,
                }],
                data: instruction_data,
            },
            vec![ctx.account_keypair.clone()],
        )
        .expect("Failed to perform operation");

        info!("Operation transaction ID: {}", txid);

        // Wait for the transaction to be processed
        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!("Processed transaction for operation: {:?}", processed_tx);

        Ok(())
    }

    fn get_account_balance(ctx: &TestContext) -> Result<BankAccount, ProgramError> {
        let account_info = read_account_info(NODE1_ADDRESS, ctx.account_pubkey)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        debug!("Raw account data: {:?}", account_info.data);
        debug!("Account owner: {:?}", account_info.owner);

        deserialize_bank_account(&account_info.data).map_err(|_| ProgramError::InvalidAccountData)
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
        let ctx = setup();
        info!("Starting test_create_account_creates_account");

        deploy_program(&ctx);
        create_caller_account(&ctx);
        create_bank_account(&ctx, "1", "Amine", 32768)?;

        // Verify the account data
        match get_account_balance(&ctx) {
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
                error!("Failed to get account balance: {:?}", e);
                return Err(ProgramError::InvalidAccountData);
            }
        }

        info!("test_create_account_creates_account completed successfully");
        Ok(())
    }

    #[test]
    #[serial]
    fn test_deposit() -> Result<(), ProgramError> {
        let ctx = setup();
        info!("Starting test_deposit");

        deploy_program(&ctx);
        create_caller_account(&ctx);
        create_bank_account(&ctx, "2", "Bob", 32768)?;

        // Get the initial balance
        let initial_account = get_account_balance(&ctx)?;
        info!("Initial account state: {:?}", initial_account);

        let deposit_amount = 500;
        info!("Attempting to deposit {} units", deposit_amount);

        perform_operation(
            &ctx,
            AccountInstruction::Deposit(DepositParams {
                account: initial_account.clone(),
                value: deposit_amount,
                // tx_hex: hex::decode(prepare_fees()).unwrap(),
            }),
        )?;

        // Wait for a moment to ensure the transaction is processed
        // std::thread::sleep(std::time::Duration::from_secs(5));

        match get_account_balance(&ctx) {
            Ok(account_data) => {
                info!("Account state after deposit: {:?}", account_data);
                let expected_balance = initial_account.balance + deposit_amount;
                assert_eq!(
                    account_data.balance, expected_balance,
                    "Balance should be {} after deposit of {}",
                    expected_balance, deposit_amount
                );
            }
            Err(e) => {
                error!("Failed to get account balance after deposit: {:?}", e);
                return Err(ProgramError::InvalidAccountData);
            }
        }

        info!("test_deposit completed successfully");
        Ok(())
    }

    #[test]
    #[serial]
    fn test_withdraw() -> Result<(), ProgramError> {
        let ctx = setup();
        info!("Starting test_withdraw");

        deploy_program(&ctx);
        create_caller_account(&ctx);
        create_bank_account(&ctx, "3", "Charlie", 2000)?;

        // Get the initial balance
        let initial_account = get_account_balance(&ctx)?;
        info!("Initial account state: {:?}", initial_account);

        let withdraw_amount = 750;
        info!("Attempting to withdraw {} units", withdraw_amount);

        perform_operation(
            &ctx,
            AccountInstruction::Withdraw(WithdrawParams {
                account: initial_account.clone(),
                value: withdraw_amount,
                // tx_hex: hex::decode(prepare_fees()).unwrap(),
            }),
        )?;

        // Wait for a moment to ensure the transaction is processed
        // std::thread::sleep(std::time::Duration::from_secs(5));

        match get_account_balance(&ctx) {
            Ok(account_data) => {
                info!("Account state after withdrawal: {:?}", account_data);
                let expected_balance = initial_account.balance - withdraw_amount;
                assert_eq!(
                    account_data.balance, expected_balance,
                    "Balance should be {} after withdrawal of {}",
                    expected_balance, withdraw_amount
                );
            }
            Err(e) => {
                error!("Failed to get account balance after withdrawal: {:?}", e);
                return Err(ProgramError::InvalidAccountData);
            }
        }

        info!("test_withdraw completed successfully");
        Ok(())
    }
}
