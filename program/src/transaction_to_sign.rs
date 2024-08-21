use crate::input_to_sign::InputToSign;

#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionToSign<'a> {
    pub tx_bytes: &'a [u8],
    pub inputs_to_sign: &'a [InputToSign],
}

// impl<'a> TransactionToSign<'a> {
//     pub fn serialise(&self) -> Vec<u8> {
//         let mut serialized = vec![];

//         serialized.push(self.tx_bytes.len() as u8);
//         serialized.extend_from_slice(self.tx_bytes);
//         serialized.push(self.inputs_to_sign.len() as u8);
//         for input_to_sign in self.inputs_to_sign.iter() {
//             serialized.extend_from_slice(&input_to_sign.index.to_le_bytes());
//             serialized.extend_from_slice(&input_to_sign.signer.serialize());
//         }

//         serialized
//     }

//     pub fn from_slice(data: &'a [u8]) -> Self {
//         let mut size = data[0] as usize + 1;
//         const inputs_to_sign_length: usize = data[size] as usize;
//         let mut program_return = TransactionToSign {
//             tx_bytes: &data[1..size],
//             inputs_to_sign: &[InputToSign {}; inputs_to_sign_length],
//         };

//         let mut inputs_to_sign = vec![];

//         size += 1;
//         for _ in 0..inputs_to_sign_length {
//             let index = u32::from_le_bytes(data[size..size+4].try_into().expect("slice with incorrect length"));
//             size += 4;
//             let signer = Pubkey::from_slice(&data[size..size+32]);
//             size += 32;
//             inputs_to_sign.push(InputToSign { index, signer });
//         }
//         program_return.inputs_to_sign = &inputs_to_sign;

//         program_return
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::{input_to_sign::InputToSign, pubkey::Pubkey};

//     use super::TransactionToSign;

//     #[test]
//     fn test_serialize_and_back() {
//         let program_return = TransactionToSign {
//             tx_bytes: vec![],
//             inputs_to_sign: vec![],
//         };
//         assert_eq!(program_return, TransactionToSign::from_slice(&program_return.serialise()));

//         let program_return = TransactionToSign {
//             tx_bytes: vec![0; 32],
//             inputs_to_sign: vec![InputToSign {
//                 index: 0,
//                 signer: Pubkey::system_program()
//             }],
//         };
//         assert_eq!(program_return, TransactionToSign::from_slice(&program_return.serialise()));
//     }

// }
