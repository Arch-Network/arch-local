use anyhow::{anyhow, Result};
use arch_program::account::AccountMeta;
use arch_program::instruction::Instruction;
use arch_program::pubkey::Pubkey;
use arch_program::utxo::UtxoMeta;
use bitcoin::key::{Keypair, UntweakedKeypair};
use bitcoin::XOnlyPublicKey;
use bitcoin::{address::Address, secp256k1::Secp256k1};
use borsh::BorshDeserialize;
use common::constants::NODE1_ADDRESS;
use common::helper::{
    get_processed_transaction, prepare_fees, read_account_info, send_utxo,
    sign_and_send_instruction,
};
use rand_core::OsRng;

use crate::counter_instructions::CounterData;

pub const DEFAULT_LOG_LEVEL: &str = "info";

pub(crate) fn log_scenario_start(
    scenario_index: u16,
    scenario_title: &str,
    scenario_description: &str,
) {
    println!();
    println!();
    println!();
    println!();
    println!("\x1b[1m\x1b[34m================================================================================================================================================================================================\x1b[0m");
    println!(
        "\x1b[1m\x1b[36m===== Scenario {} : \x1b[0m \x1b[1m {} \x1b[36m=====\x1b[0m",
        scenario_index, scenario_title
    );
    println!("\x1b[1m\x1b[34m================================================================================================================================================================================================\x1b[0m");
    println!(
        "\x1b[1m\x1b[3m\x1b[36m=====\x1b[0m \x1b[1m\x1b[3m {} \x1b[0m",
        scenario_description
    );
    println!("\x1b[1m\x1b[34m================================================================================================================================================================================================\x1b[0m");
}

pub(crate) fn log_scenario_end(scenario_index: u16, scenario_states: &str) {
    println!();
    println!("\x1b[1m\x1b[34m================================================================================================================================================================================================\x1b[0m");
    println!("\x1b[1m\x1b[32m===== Scenario {} Finished Successfully !\x1b[0m \x1b[1m Final state {} \x1b[32m=====\x1b[0m", scenario_index, scenario_states);
    println!("\x1b[1m\x1b[34m================================================================================================================================================================================================\x1b[0m");
}

pub(crate) fn init_logging() {
    use std::{env, sync::Once};

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", DEFAULT_LOG_LEVEL);
        }

        tracing_subscriber::fmt()
            .without_time()
            .with_file(false)
            .with_line_number(false)
            .with_env_filter(tracing_subscriber::EnvFilter::new(format!(
                "{},reqwest=off,hyper=off",
                env::var("RUST_LOG").unwrap()
            )))
            .init();
    });
}

pub(crate) fn assign_ownership_to_program(
    program_pubkey: &Pubkey,
    account_to_transfer_pubkey: Pubkey,
    current_owner_keypair: Keypair,
) {
    let mut instruction_data = vec![3];
    instruction_data.extend(program_pubkey.serialize());

    let (txid, _) = sign_and_send_instruction(
        Instruction {
            program_id: Pubkey::system_program(),
            accounts: vec![AccountMeta {
                pubkey: account_to_transfer_pubkey,
                is_signer: true,
                is_writable: true,
            }],
            data: instruction_data,
        },
        vec![current_owner_keypair],
    )
    .expect("signing and sending a transaction should not fail");

    let _processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
        .expect("get processed transaction should not fail");

    println!("\x1b[32m Step 2/3 Successful :\x1b[0m Ownership Successfully assigned to program!");
}

pub(crate) fn generate_new_keypair() -> (UntweakedKeypair, Pubkey, Address) {
    let secp = Secp256k1::new();

    let (secret_key, _public_key) = secp.generate_keypair(&mut OsRng);

    let key_pair = UntweakedKeypair::from_secret_key(&secp, &secret_key);

    let (x_only_public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    let address = Address::p2tr(&secp, x_only_public_key, None, bitcoin::Network::Regtest);

    let pubkey = Pubkey::from_slice(&XOnlyPublicKey::from_keypair(&key_pair).0.serialize());

    (key_pair, pubkey, address)
}

pub(crate) fn get_account_counter(account_pubkey: &Pubkey) -> Result<CounterData> {
    let account_info = read_account_info(NODE1_ADDRESS, account_pubkey.clone())
        .map_err(|e| anyhow!(format!("Error reading account content {}", e.to_string())))?;

    let mut account_info_data = account_info.data.as_slice();

    let account_counter = CounterData::deserialize(&mut account_info_data)
        .map_err(|e| anyhow!(format!("Error corrupted account data {}", e.to_string())))?;

    Ok(account_counter)
}

pub(crate) fn generate_anchoring(account_pubkey: &Pubkey) -> (UtxoMeta, Vec<u8>) {
    let (utxo_txid, utxo_vout) = send_utxo(account_pubkey.clone());

    let fees_psbt = prepare_fees();

    return (
        UtxoMeta::from(
            hex::decode(utxo_txid.clone()).unwrap().try_into().unwrap(),
            utxo_vout,
        ),
        hex::decode(fees_psbt).unwrap(),
    );
}
