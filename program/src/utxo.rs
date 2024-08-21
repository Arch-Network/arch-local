#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct UtxoMeta([u8; 36]);

impl UtxoMeta {
    pub fn from(txid: [u8; 32], vout: u32) -> Self {
        let mut data: [u8; 36] = [0; 36];
        data[..32].copy_from_slice(&txid);
        data[32..].copy_from_slice(&vout.to_le_bytes());
        Self(data)
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Self(data[..36].try_into().expect("utxo meta is 36 bytes long"))
    }

    pub fn txid(&self) -> &[u8] {
        &self.0[..32]
    }

    pub fn txid_mut(&mut self) -> &mut [u8] {
        &mut self.0[..32]
    }

    pub fn vout_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0[32..]
    }

    pub fn vout(&self) -> u32 {
        u32::from_le_bytes(self.0[32..].try_into().expect("utxo meta unreachable"))
    }

    pub fn serialize(&self) -> [u8; 36] {
        self.0
    }
}

use core::fmt;

/// TODO:
///  Change this in future according to the correct base implementation
impl fmt::Display for UtxoMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl AsRef<[u8]> for UtxoMeta {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsMut<[u8]> for UtxoMeta {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl From<[u8; 36]> for UtxoMeta {
    fn from(value: [u8; 36]) -> Self {
        UtxoMeta(value)
    }
}
