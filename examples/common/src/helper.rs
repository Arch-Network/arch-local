//! This module contains helper methods for interacting with the HelloWorld program

use anyhow::{anyhow, Result};
use bitcoin::{
    absolute::LockTime,
    address::Address,
    key::{TapTweak, TweakedKeypair},
    opcodes::all::OP_RETURN,
    secp256k1::{self, Secp256k1},
    sighash::{Prevouts, SighashCache},
    transaction::Version,
    Amount, OutPoint, ScriptBuf, Sequence, TapSighashType, Transaction, TxIn, TxOut, Witness,
};
use bitcoincore_rpc::{Auth, Client, RawTx, RpcApi};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use serde_json::{from_str, json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Child;
use std::process::Command;
use std::str::FromStr;
use risc0_zkvm::Receipt;

use sdk::{Pubkey, Instruction, UtxoMeta, RuntimeTransaction, Message, Signature};

use crate::constants::{
    BITCOIN_NODE_ENDPOINT, BITCOIN_NODE_PASSWORD, BITCOIN_NODE_USERNAME,
    CALLER_FILE_PATH, GET_BEST_BLOCK_HASH, GET_BLOCK, GET_CONTRACT_ADDRESS,
    GET_PROCESSED_TRANSACTION, GET_PROGRAM, NODE1_ADDRESS, READ_UTXO,
    TRANSACTION_NOT_FOUND_CODE
};
use crate::models::{
    BitcoinRpcInfo, CallerInfo, DeployProgramParams, ReadUtxoParams,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ReadUtxoResult {
    pub utxo_id: String,
    pub data: Vec<u8>,
    pub authority: Pubkey,
}

#[derive(Clone, Debug, Deserialize, Serialize, BorshDeserialize, BorshSerialize)]
pub enum Status {
    Processing,
    Success,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessedTransaction {
    pub runtime_transaction: RuntimeTransaction,
    pub receipts: HashMap<String, Receipt>,
    pub status: Status,
    pub bitcoin_txids: HashMap<String, String>,
}

fn process_result(response: String) -> Result<Value> {
    let result = from_str::<Value>(&response).expect("result should be Value parseable");

    let result = match result {
        Value::Object(object) => object,
        _ => panic!("unexpected output"),
    };

    if let Some(err) = result.get("error") {
        return Err(anyhow!("{:?}", err));
    }

    Ok(result["result"].clone())
}

fn process_get_transaction_result(response: String) -> Result<Value> {
    let result = from_str::<Value>(&response).expect("result should be string parseable");

    let result = match result {
        Value::Object(object) => object,
        _ => panic!("unexpected output"),
    };

    if let Some(err) = result.get("error") {
        if let Value::Number(code) = result["error"]["code"].clone() {
            if code.as_i64() == Some(TRANSACTION_NOT_FOUND_CODE) {
                return Ok(Value::Null);
            }
        }
        return Err(anyhow!("{:?}", err));
    }

    Ok(result["result"].clone())
}

fn post(url: &str, method: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "curlycurl",
            "method": method,
        }))
        .send()
        .expect("post method should not fail");

    res.text().expect("result should be text decodable")
}

fn post_data<T: Serialize + std::fmt::Debug>(url: &str, method: &str, params: T) -> String {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "curlycurl",
            "method": method,
            "params": params,
        }))
        .send()
        .expect("post method should not fail");

    res.text().expect("result should be text decodable")
}

/// Returns a caller information using the secret key file specified
fn get_trader(trader_id: u64) -> Result<CallerInfo> {
    let file_path = &format!("../../.arch/trader{}.json", trader_id);
    Ok(CallerInfo::with_secret_key_file(file_path)?)
}

/// Creates an instruction, signs it as a message
/// and sends the signed message as a transaction
pub fn sign_and_send_instruction(
    program_id: Pubkey,
    utxos: Vec<UtxoMeta>,
    instruction_data: Vec<u8>,
) -> Result<(String, String)> {
    let caller = CallerInfo::with_secret_key_file(CALLER_FILE_PATH)?;

    let instruction = Instruction {
        program_id,
        utxos,
        data: instruction_data,
    };

    let message = Message {
        signers: vec![Pubkey::from_slice(&caller.public_key.serialize())],
        instructions: vec![instruction.clone()],
    };
    let digest_slice = hex::decode(message.hash().expect("message should be hashable"))
        .expect("hashed message should be decodable");
    let sig_message = secp256k1::Message::from_digest_slice(&digest_slice)
        .expect("signed message should be gotten from digest slice");

    let secp = Secp256k1::new();
    let sig = secp.sign_schnorr(&sig_message, &caller.key_pair);

    let params = RuntimeTransaction {
        version: 0,
        signatures: vec![Signature(sig.serialize().to_vec())],
        message,
    };
    let result = process_result(post_data(NODE1_ADDRESS, "send_transaction", params))
        .expect("send_transaction should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string();
    let hashed_instruction = instruction
        .hash()
        .expect("instruction hashing should not fail");

    Ok((result, hashed_instruction))
}

/// Deploys the HelloWorld program using the compiled ELF
pub fn deploy_program() -> String {
    let elf = fs::read("target/program.elf").expect("elf path should be available");
    let params = DeployProgramParams { elf };
    process_result(post_data(NODE1_ADDRESS, "deploy_program", params))
        .expect("deploy_program should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string()
}

/// Starts Key Exchange by calling the RPC method
pub fn start_key_exchange() {
    match process_result(post(NODE1_ADDRESS, "start_key_exchange")) {
        Err(err) => println!("Error starting Key Exchange: {:?}", err),
        Ok(val) => assert!(val.as_bool().unwrap())
    };
}

/// Starts a Distributed Key Generation round by calling the RPC method
pub fn start_dkg() {
    if let Err(err) = process_result(post(NODE1_ADDRESS, "start_dkg")) {
        println!("Error starting DKG: {:?}", err);
    };
}

/// Read Utxo given the utxo ID
pub fn read_utxo(url: &str, utxo_id: String) -> Result<ReadUtxoResult> {
    let params = ReadUtxoParams { utxo_id };
    let result = process_result(post_data(url, READ_UTXO, params))
        .expect("read_utxo should not fail");
    serde_json::from_value(result).map_err(|_| anyhow!("Unable to decode read_utxo result"))
}

/// Returns a program given the program ID
pub fn get_program(url: &str, program_id: String) -> String {
    process_result(post_data(url, GET_PROGRAM, program_id))
        .expect("get_program should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string()
}

/// Returns the best block
fn get_best_block() -> String {
    let best_block_hash = process_result(post(NODE1_ADDRESS, GET_BEST_BLOCK_HASH))
        .expect("best_block_hash should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string();
    process_result(post_data(NODE1_ADDRESS, GET_BLOCK, best_block_hash))
        .expect("get_block should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string()
}

/// Returns a processed transaction given the txid
/// Keeps trying for a maximum of 60 seconds if the processed transaction is not available
pub fn get_processed_transaction(url: &str, tx_id: String) -> Result<ProcessedTransaction> {
    let mut processed_tx = process_get_transaction_result(post_data(url, GET_PROCESSED_TRANSACTION, tx_id.clone()));
    if let Err(e) = processed_tx {
        return Err(anyhow!("{}", e));
    }

    let mut wait_time = 10;
    while let Ok(Value::Null) = processed_tx {
        println!("Processed transaction is not yet in the database. Retrying...");
        std::thread::sleep(std::time::Duration::from_secs(wait_time));
        processed_tx = process_get_transaction_result(post_data(url, GET_PROCESSED_TRANSACTION, tx_id.clone()));
        wait_time += 10;
        if wait_time >= 60 {
            println!("get_processed_transaction has run for more than 60 seconds");
            return Err(anyhow!("Failed to retrieve processed transaction"));
        }
    }

    if let Ok(ref tx) = processed_tx {
        let mut p = tx.clone();
        while p["status"].as_str().unwrap() != "Success" {
            println!("Processed transaction is not yet finalized. Retrying...");
            std::thread::sleep(std::time::Duration::from_secs(wait_time));
            p = process_get_transaction_result(post_data(url, GET_PROCESSED_TRANSACTION, tx_id.clone())).unwrap();
            wait_time += 10;
            if wait_time >= 60 {
                println!("get_processed_transaction has run for more than 60 seconds");
                return Err(anyhow!("Failed to retrieve processed transaction"));
            }
        }
        processed_tx = Ok(p);
    }

    Ok(serde_json::from_value::<ProcessedTransaction>(processed_tx?).unwrap())
}

pub fn prepare_fees() -> String {
    let userpass = Auth::UserPass(
        BITCOIN_NODE_USERNAME.to_string(),
        BITCOIN_NODE_PASSWORD.to_string(),
    );
    let rpc =
        Client::new(BITCOIN_NODE_ENDPOINT, userpass).expect("rpc shouldn not fail to be initiated");

    let caller = CallerInfo::with_secret_key_file(CALLER_FILE_PATH)
        .expect("getting caller info should not fail");

    let txid = rpc
        .send_to_address(
            &caller.address,
            Amount::from_sat(3000),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("SATs should be sent to address");

    let sent_tx = rpc
        .get_raw_transaction(&txid, None)
        .expect("should get raw transaction");
    let mut vout = 0;

    for (index, output) in sent_tx.output.iter().enumerate() {
        if output.script_pubkey == caller.address.script_pubkey() {
            vout = index as u32;
        }
    }

    let mut tx = Transaction {
        version: Version::TWO,
        input: vec![TxIn {
            previous_output: OutPoint { txid, vout },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        }],
        output: vec![],
        lock_time: LockTime::ZERO,
    };

    let sighash_type = TapSighashType::NonePlusAnyoneCanPay;
    let raw_tx = rpc
        .get_raw_transaction(&txid, None)
        .expect("raw transaction should not fail");
    let prevouts = vec![raw_tx.output[vout as usize].clone()];
    let prevouts = Prevouts::All(&prevouts);

    let mut sighasher = SighashCache::new(&mut tx);
    let sighash = sighasher
        .taproot_key_spend_signature_hash(0, &prevouts, sighash_type)
        .expect("should not fail to construct sighash");

    // Sign the sighash using the secp256k1 library (exported by rust-bitcoin).
    let secp = Secp256k1::new();
    let tweaked: TweakedKeypair = caller.key_pair.tap_tweak(&secp, None);
    let msg = secp256k1::Message::from(sighash);
    let signature = secp.sign_schnorr(&msg, &tweaked.to_inner());

    // Update the witness stack.
    let signature = bitcoin::taproot::Signature {
        sig: signature,
        hash_ty: sighash_type,
    };
    tx.input[0].witness.push(signature.to_vec());

    tx.raw_hex()
}

pub fn send_utxo() -> String {
    let userpass = Auth::UserPass(
        BITCOIN_NODE_USERNAME.to_string(),
        BITCOIN_NODE_PASSWORD.to_string(),
    );
    let rpc =
        Client::new(BITCOIN_NODE_ENDPOINT, userpass).expect("rpc shouldn not fail to be initiated");

    let caller = CallerInfo::with_secret_key_file(CALLER_FILE_PATH)
        .expect("getting caller info should not fail");

    let txid = rpc
        .send_to_address(
            &caller.address,
            Amount::from_sat(3000),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("SATs should be sent to address");

    let sent_tx = rpc
        .get_raw_transaction(&txid, None)
        .expect("should get raw transaction");
    let mut vout = 0;

    for (index, output) in sent_tx.output.iter().enumerate() {
        if output.script_pubkey == caller.address.script_pubkey() {
            vout = index as u32;
        }
    }

    let network_address = get_network_address("");

    let mut tx = Transaction {
        version: Version::TWO,
        input: vec![TxIn {
            previous_output: OutPoint { txid, vout },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        }],
        output: vec![
            TxOut {
                value: Amount::from_sat(0),
                script_pubkey: ScriptBuf::builder()
                    .push_opcode(OP_RETURN)
                    .push_x_only_key(&caller.public_key)
                    .into_script(),
            },
            TxOut {
                value: Amount::from_sat(1500),
                script_pubkey: Address::from_str(&network_address)
                    .unwrap()
                    .require_network(bitcoin::Network::Regtest)
                    .unwrap()
                    .script_pubkey(),
            },
        ],
        lock_time: LockTime::ZERO,
    };

    let sighash_type = TapSighashType::NonePlusAnyoneCanPay;
    let raw_tx = rpc
        .get_raw_transaction(&txid, None)
        .expect("raw transaction should not fail");
    let prevouts = vec![raw_tx.output[vout as usize].clone()];
    let prevouts = Prevouts::All(&prevouts);

    let mut sighasher = SighashCache::new(&mut tx);
    let sighash = sighasher
        .taproot_key_spend_signature_hash(0, &prevouts, sighash_type)
        .expect("should not fail to construct sighash");

    // Sign the sighash using the secp256k1 library (exported by rust-bitcoin).
    let secp = Secp256k1::new();
    let tweaked: TweakedKeypair = caller.key_pair.tap_tweak(&secp, None);
    let msg = secp256k1::Message::from(sighash);
    let signature = secp.sign_schnorr(&msg, &tweaked.to_inner());

    // Update the witness stack.
    let signature = bitcoin::taproot::Signature {
        sig: signature,
        hash_ty: sighash_type,
    };
    tx.input[0].witness.push(signature.to_vec());

    // BOOM! Transaction signed and ready to broadcast.
    rpc.send_raw_transaction(tx.raw_hex())
        .expect("sending raw transaction should not fail")
        .to_string()
}

fn get_network_address(data: &str) -> String {
    let mut params = HashMap::new();
    params.insert("data", data.as_bytes());
    process_result(post_data(NODE1_ADDRESS, GET_CONTRACT_ADDRESS, params))
        .expect("get_contract_address should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string()
}

fn get_address_utxos(rpc: &Client, address: String) -> Vec<Value> {
    let client = reqwest::blocking::Client::new();

    let res = client
        .get(format!(
            "https://mempool.dev.aws.archnetwork.xyz/api/address/{}/utxo",
            address
        ))
        .header("Accept", "application/json")
        .send()
        .unwrap();

    let utxos = from_str::<Value>(&res.text().unwrap()).unwrap();

    utxos
        .as_array()
        .unwrap()
        .iter()
        .filter(|utxo| {
            utxo["status"]["block_height"].as_u64().unwrap() <= rpc.get_block_count().unwrap() - 100
        })
        .map(|utxo| utxo.clone())
        .collect()
}

pub fn start_boot_node(port: u16, arch_nodes: &str, bitcoin_rpc_info: &BitcoinRpcInfo) -> Child {
    std::env::set_var("RISC0_DEV_MODE", "1");

    let mut command = Command::new("cargo");
    command.current_dir(env::current_dir().unwrap().parent().unwrap());

    command.args([
        "run",
        "-p",
        "zkvm",
        "--",
        "--is-boot-node",
        "--arch-nodes",
        arch_nodes,
        "--rpc-bind-port",
        &port.to_string(),
        "--bitcoin-rpc-endpoint",
        &bitcoin_rpc_info.endpoint,
        "--bitcoin-rpc-port",
        &bitcoin_rpc_info.port.to_string(),
        "--bitcoin-rpc-username",
        &bitcoin_rpc_info.username,
        "--bitcoin-rpc-password",
        &bitcoin_rpc_info.password,
    ]);

    command.spawn().expect("Failed to start node process")
}

pub fn start_node(port: u16, bitcoin_rpc_info: &BitcoinRpcInfo) -> Child {
    env::set_var("RISC0_DEV_MODE", "1");

    let mut command = Command::new("cargo");
    command.current_dir(env::current_dir().unwrap().parent().unwrap());

    command.args([
        "run",
        "-p",
        "arch-node",
        "--",
        "--rpc-bind-port",
        &port.to_string(),
        "--bitcoin-rpc-endpoint",
        &bitcoin_rpc_info.endpoint,
        "--bitcoin-rpc-port",
        &bitcoin_rpc_info.port.to_string(),
        "--bitcoin-rpc-username",
        &bitcoin_rpc_info.username,
        "--bitcoin-rpc-password",
        &bitcoin_rpc_info.password,
        "--data-dir",
        &format!(".participant{}", port),
    ]);

    command.spawn().expect("Failed to start node process")
}

async fn stop_node(mut child: Child) {
    match child.kill() {
        Ok(_) => println!("Node stopped successfully."),
        Err(e) => eprintln!("Failed to stop node: {}", e),
    }

    let _ = child.wait();
}
