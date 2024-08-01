#[macro_export]
macro_rules! entrypoint {
    ($process_instruction:ident) => {
        use $crate::*;
        risc0_zkvm::guest::entry!(entrypoint);
        use std::collections::HashMap;

        pub fn entrypoint() {
            let serialized_instruction: Vec<u8> = risc0_zkvm::guest::env::read();
            let instruction: Instruction = borsh::from_slice(&serialized_instruction).unwrap();
            let program_id: Pubkey = instruction.program_id;
            let authorities: HashMap<String, Vec<u8>> = risc0_zkvm::guest::env::read();
            let data: HashMap<String, Vec<u8>> = risc0_zkvm::guest::env::read();

            let utxos = instruction
                .utxos
                .iter()
                .map(|utxo| {
                    use std::cell::RefCell;

                    UtxoInfo {
                        txid: utxo.txid.clone(),
                        vout: utxo.vout,
                        authority: RefCell::new(Pubkey::from_slice(
                            &authorities
                                .get(&utxo.id())
                                .expect("this utxo does not exist")
                                .to_vec(),
                        )),
                        data: RefCell::new(
                            data.get(&utxo.id())
                                .expect("this utxo does not exist")
                                .to_vec(),
                        ),
                    }
                })
                .collect::<Vec<UtxoInfo>>();

            let instruction_data: Vec<u8> = instruction.data;

            match $process_instruction(&program_id, &utxos, &instruction_data) {
                Ok(tx_hex) => {
                    let mut new_authorities: HashMap<String, Vec<u8>> = HashMap::new();
                    let mut new_data: HashMap<String, Vec<u8>> = HashMap::new();
                    utxos.iter().for_each(|utxo| {
                        new_authorities.insert(utxo.id(), utxo.authority.clone().into_inner().serialize().to_vec());
                        new_data.insert(utxo.id(), utxo.data.clone().into_inner());
                    });
                    risc0_zkvm::guest::env::commit(
                        &borsh::to_vec(&(new_authorities, new_data, tx_hex)).unwrap(),
                    )
                }
                Err(e) => panic!("err: {:?}", e),
            }
        }
    };
}