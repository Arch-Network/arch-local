use arch_program::input_to_sign::InputToSign;

#[derive(Debug, Clone, Default)]
pub struct TransactionToSign {
    pub tx_bytes: Vec<u8>,
    pub inputs_to_sign: Vec<InputToSign>,
}
