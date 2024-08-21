use std::str::FromStr;

use bitcoin::{
    absolute::LockTime,
    opcodes::all::{OP_CHECKSIG, OP_ENDIF, OP_IF},
    script::PushBytesBuf,
    transaction::Version,
    OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};

use crate::{
    account::AccountInfo,
    program::{get_bitcoin_tx, get_network_xonly_pubkey},
};

pub fn get_state_trasition_tx(accounts: &[AccountInfo]) -> Transaction {
    Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: accounts
            .iter()
            .filter(|account| account.is_writable)
            .map(|account| TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_str(&hex::encode(account.utxo.txid())).unwrap(),
                    vout: account.utxo.vout(),
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            })
            .collect::<Vec<TxIn>>(),
        output: accounts
            .iter()
            .filter(|account| account.is_writable)
            .map(|account| {
                let tx: Transaction = bitcoin::consensus::deserialize(
                    &get_bitcoin_tx(account.utxo.txid().try_into().unwrap()).unwrap(),
                )
                .unwrap();

                //let x_only_pub_key = XOnlyPublicKey::from_slice(&get_network_xonly_pubkey()).map_err(|_| String::from("failed to deserialize XOnlyPublicKey")).unwrap();

                let mut script_builder = ScriptBuf::builder();
                script_builder = script_builder
                    .push_slice(get_network_xonly_pubkey())
                    .push_opcode(OP_CHECKSIG);

                script_builder = script_builder.push_opcode(OP_IF);
                let push_bytes_buf = PushBytesBuf::try_from(account.key.serialize().to_vec())
                    .map_err(|_| String::from("failed to deserialize PushBytesBuf"))
                    .unwrap();
                script_builder = script_builder.push_slice(push_bytes_buf);
                script_builder = script_builder.push_opcode(OP_ENDIF);

                TxOut {
                    value: tx.output[account.utxo.vout() as usize].value,
                    script_pubkey: script_builder.into_script(),
                }
            })
            .collect::<Vec<TxOut>>(),
    }
}
