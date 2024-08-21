use crate::pubkey::Pubkey;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputToSign {
    pub index: u32,
    pub signer: Pubkey,
}
