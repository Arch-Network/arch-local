use arch_program::{
    account::AccountInfo, account::AccountMeta, instruction::Instruction, pubkey::Pubkey,
    system_instruction::SystemInstruction, utxo::UtxoMeta,
};
use bitcoincore_rpc::{Auth, Client};
use borsh::{BorshDeserialize, BorshSerialize};
use common::constants::*;
use common::helper::*;
use common::models::*;
use std::fs;
use std::str::FromStr;
use std::thread;

use env_logger;
use log::{debug, error, info, warn};

#[path = "../program/src/curve.rs"]
pub mod curve;
#[path = "../program/src/operations.rs"]
pub mod operations;
#[path = "../program/src/reward.rs"]
pub mod reward;
#[path = "../program/src/state.rs"]
pub mod state;

use crate::curve::calculate_swap_amount;
use crate::operations::add_liquidity;
use crate::operations::claim_rewards;
use crate::operations::remove_liquidity;
use crate::operations::unstake_tokens;
pub use crate::reward::RewardParams;
pub use crate::state::LiquidityParams;

// Define the `create_mock_account` function
pub fn create_mock_account<'a>(
    key: &'a Pubkey,
    data: &'a mut [u8],
    owner: &'a Pubkey,
    utxo: &'a UtxoMeta,
) -> AccountInfo<'a> {
    AccountInfo::new(
        key, data, owner, utxo, true,  // is_signer
        true,  // is_writable
        false, // is_executable
    )
}

fn setup() {
    env_logger::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_call() {
        setup();

        let hello_name = "Satoshi"; // Name to be used in the Swap program

        info!("Starting test_deploy_call");
        debug!(
            "u64 10044 in little-endian bytes: {:?}",
            10044_u64.to_le_bytes()
        );
        debug!(
            "u64 10881 in little-endian bytes: {:?}",
            10881_u64.to_le_bytes()
        );

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
            "program/target/sbf-solana-solana/release/swapprogram.so",
        );
        info!("Program deployed with transaction IDs: {:?}", txids);

        println!("program id: {:?}", Pubkey::system_program());

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

        info!("test_deploy_call completed successfully");
    }

    #[test]
    fn test_create_mock_account() {
        let pubkey = Pubkey::from_slice(&[0u8; 32]); // Use a dummy pubkey
        let owner = Pubkey::from_slice(&[1u8; 32]); // Use another dummy pubkey
        let mut data = Box::new([0u8; 64]); // Allocate data on the heap
        let utxo = Box::new(UtxoMeta::from_slice(&[0u8; 36])); // Allocate UTXO on the heap

        let account = create_mock_account(
            &pubkey, &mut *data, // Dereference Box to get a mutable reference
            &owner, &utxo,
        );

        assert_eq!(account.data_len(), 64);
        assert!(!account.data_is_empty());
        assert_eq!(account.key.as_ref(), [0u8; 32]);
    }

    #[test]
    fn test_add_liquidity() {
        let pubkey = Pubkey::from_slice(&[0u8; 32]); // Dummy pubkey
        let owner = Pubkey::from_slice(&[1u8; 32]); // Another dummy pubkey

        // Create LiquidityParams with example initial values
        let mut liquidity_params = LiquidityParams::new(1000, 2000);

        // Serialize the LiquidityParams to get the correct size for the mock data
        let serialized_data = borsh::to_vec(&liquidity_params).unwrap();
        let mut data = Box::new([0u8; 24]); // Allocate data on the heap

        // If the serialized size is smaller than the mock data size, adjust it
        let data_len = serialized_data.len();
        data[..data_len].copy_from_slice(&serialized_data);

        let utxo = Box::new(UtxoMeta::from_slice(&[0u8; 36])); // Mock UTXO

        let account = create_mock_account(&pubkey, &mut *data, &owner, &utxo);

        // Simulate adding liquidity
        let token_a_amount = 500;
        let token_b_amount = 1000;

        let result = add_liquidity(
            &account,
            &mut liquidity_params,
            token_a_amount,
            token_b_amount,
        );

        // Assert success or failure based on your implementation
        assert!(result.is_ok());

        // Validate expected account state changes
        assert_eq!(liquidity_params.token_a_amount, 1500); // Adjust expected values as needed
        assert_eq!(liquidity_params.token_b_amount, 3000); // Adjust expected values as needed
        assert_eq!(liquidity_params.liquidity_amount, 4500); // Total liquidity
    }

    #[test]
    fn test_remove_liquidity() {
        let pubkey = Pubkey::from_slice(&[0u8; 32]); // Dummy pubkey
        let owner = Pubkey::from_slice(&[1u8; 32]); // Another dummy pubkey

        // Create LiquidityParams with example initial values
        let mut liquidity_params = LiquidityParams::new(1500, 2500);

        // Serialize the LiquidityParams to get the correct size for the mock data
        let serialized_data = borsh::to_vec(&liquidity_params).unwrap();
        let mut data = Box::new([0u8; 24]); // Allocate data on the heap

        // If the serialized size is smaller than the mock data size, adjust it
        let data_len = serialized_data.len();
        data[..data_len].copy_from_slice(&serialized_data);

        let utxo = Box::new(UtxoMeta::from_slice(&[0u8; 36])); // Mock UTXO

        let account = create_mock_account(&pubkey, &mut *data, &owner, &utxo);

        // Simulate removing liquidity
        let token_a_amount = 500;
        let token_b_amount = 1000;
        let liquidity_amount = 1500; // Added missing u64 argument

        let result = remove_liquidity(
            &account,
            &mut liquidity_params,
            token_a_amount,
            token_b_amount,
            liquidity_amount,
        );

        // Assert success or failure based on your implementation
        assert!(result.is_ok());

        // Validate expected account state changes
        assert_eq!(liquidity_params.token_a_amount, 1000); // Adjust expected values as needed
        assert_eq!(liquidity_params.token_b_amount, 1500); // Adjust expected values as needed
        assert_eq!(liquidity_params.liquidity_amount, 2500); // Total liquidity
    }
    #[test]
    fn test_get_liquidity_amount() {
        let pubkey = Pubkey::from_slice(&[0u8; 32]); // Dummy pubkey
        let owner = Pubkey::from_slice(&[1u8; 32]); // Another dummy pubkey

        // Create LiquidityParams with example initial values
        let liquidity_params = LiquidityParams::new(1000, 2000);

        // Serialize the LiquidityParams to get the correct size for the mock data
        let serialized_data = borsh::to_vec(&liquidity_params).unwrap();
        let mut data = Box::new([0u8; 24]); // Allocate data on the heap

        // If the serialized size is smaller than the mock data size, adjust it
        let data_len = serialized_data.len();
        data[..data_len].copy_from_slice(&serialized_data);

        let utxo = Box::new(UtxoMeta::from_slice(&[0u8; 36])); // Mock UTXO

        let account = create_mock_account(&pubkey, &mut *data, &owner, &utxo);

        // Get liquidity amount
        let liquidity_amount = liquidity_params.get_liquidity_amount();

        // Validate the liquidity amount
        assert_eq!(liquidity_amount, 3000); // Total liquidity (token_a + token_b)
    }

    #[test]
    fn test_swap_tokens() {
        let pubkey = Pubkey::from_slice(&[0u8; 32]); // Dummy pubkey
        let owner = Pubkey::from_slice(&[1u8; 32]); // Another dummy pubkey

        // Create LiquidityParams with example initial values
        let mut liquidity_params = LiquidityParams::new(1500, 2500);

        // Serialize the LiquidityParams to get the correct size for the mock data
        let serialized_data = borsh::to_vec(&liquidity_params).unwrap();
        let mut data = Box::new([0u8; 24]); // Allocate data on the heap

        // If the serialized size is smaller than the mock data size, adjust it
        let data_len = serialized_data.len();
        data[..data_len].copy_from_slice(&serialized_data);

        let utxo = Box::new(UtxoMeta::from_slice(&[0u8; 36])); // Mock UTXO

        let account = create_mock_account(&pubkey, &mut *data, &owner, &utxo);

        // Simulate a token swap using `calculate_swap_amount`
        let swap_input_amount = 500; // Amount of token A to swap
        let swap_output_amount = calculate_swap_amount(
            liquidity_params.token_a_amount,
            liquidity_params.token_b_amount,
            swap_input_amount,
        );

        // Update the liquidity params after the swap
        liquidity_params.token_a_amount += swap_input_amount;
        liquidity_params.token_b_amount -= swap_output_amount;

        // Validate the swap results
        assert_eq!(swap_output_amount, 625); // Expected output amount based on swap logic
        assert_eq!(liquidity_params.token_a_amount, 2000); // Updated token A amount
        assert_eq!(liquidity_params.token_b_amount, 1875); // Updated token B amount
    }

    #[test]
    fn test_stake_unstake_claim_rewards() {
        let pubkey = Pubkey::from_slice(&[0u8; 32]); // Dummy pubkey
        let owner = Pubkey::from_slice(&[1u8; 32]); // Another dummy pubkey

        // Initialize RewardParams using the default constructor
        let mut reward_params = RewardParams::new();

        // Manually set the fields since the `new()` function takes no arguments
        reward_params.total_staked = 1000; // Initial staked amount
        reward_params.total_rewards = 8; // Total rewards available
        reward_params.reward_rate = 0.001; // Reward rate per block/transaction/etc.

        // Serialize the RewardParams to get the correct size for the mock data
        let serialized_data = borsh::to_vec(&reward_params).unwrap();
        let mut data = Box::new([0u8; 24]); // Allocate data on the heap

        // If the serialized size is smaller than the mock data size, adjust it
        let data_len = serialized_data.len();
        data[..data_len].copy_from_slice(&serialized_data);

        let utxo = Box::new(UtxoMeta::from_slice(&[0u8; 36])); // Mock UTXO

        let account = create_mock_account(&pubkey, &mut *data, &owner, &utxo);

        // Step 1: Stake tokens
        let stake_amount = 5; // Amount to stake
        //need a fix
        let stake_result = unstake_tokens(&account, &mut reward_params, stake_amount);
        assert!(stake_result.is_ok());

        // Validate that the staked amount has increased
        assert_eq!(reward_params.total_staked, 1005);

        // Step 2: Unstake tokens
        let unstake_amount = 2; // Amount to unstake
        let unstake_result = unstake_tokens(&account, &mut reward_params, unstake_amount);
        assert!(unstake_result.is_ok());

        // Validate that the staked amount has decreased
        assert_eq!(reward_params.total_staked, 1003);

        // Step 3: Claim rewards
        let claim_result = claim_rewards(&account, &mut reward_params);
        assert!(claim_result.is_ok());

        // Validate that rewards have been claimed and total rewards have been reduced
        let claimed_rewards = claim_result.unwrap();
        assert!(claimed_rewards > 0.002); // Ensure some rewards were claimed
        assert!((reward_params.total_rewards as f64) >= claimed_rewards); // Total rewards should be greater than or equal to the claimed amount

        // Optionally log claimed rewards for better visibility (can be removed in production)
        println!("Claimed rewards: {}", claimed_rewards);

    }}
