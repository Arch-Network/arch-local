use risc0_zkvm::declare_syscall;

declare_syscall!(pub SYS_GET_TX_SIGNERS);
declare_syscall!(pub SYS_GET_BITCOIN_TX);

#[cfg(target_os = "zkvm")]
pub mod vm {
    use super::*;
    use crate::Pubkey;
    use risc0_zkvm::guest::env::send_recv_slice;

    pub fn get_tx_signers() -> Result<Vec<Pubkey>, anyhow::Error> {
        let result: &[u8] = send_recv_slice::<u8, u8>(SYS_GET_TX_SIGNERS, &[]);
        Ok(borsh::from_slice(result)?)
    }

    pub fn get_bitcoin_tx(txid: String) -> Result<Vec<u8>, anyhow::Error> {
        let result: &[u8] = send_recv_slice::<u8, u8>(SYS_GET_BITCOIN_TX, &hex::decode(txid)?);
        Ok(result.to_vec())
    }
}

#[cfg(not(target_os = "zkvm"))]
pub mod host {
    use crate::Pubkey;
    use bitcoincore_rpc::{self, Client, RpcApi};
    use risc0_zkvm::Bytes;
    use std::str::FromStr;

    pub fn get_tx_signers(signers: &[Pubkey], _data: Bytes) -> Result<Bytes, anyhow::Error> {
        Ok(borsh::to_vec(&signers)?.into())
    }

    pub fn get_bitcoin_tx(rpc: &Client, data: Bytes) -> Result<Bytes, anyhow::Error> {
        let txid = bitcoincore_rpc::bitcoin::Txid::from_str(&hex::encode(data.as_ref()))?;
        let tx = rpc.get_raw_transaction_hex(&txid, None).unwrap();
        Ok(hex::decode(tx)?.into())
    }
}
