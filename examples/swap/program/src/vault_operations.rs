use crate::vault::Vault;
use crate::operations::{calculate_swap_amount};
use arch_program::{account::AccountInfo, program_error::ProgramError};
use solana_program::pubkey::Pubkey;

pub fn add_liquidity_to_vault(
    vault: &mut Vault,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<(), ProgramError> {
    vault.token_a_amount += token_a_amount;
    vault.token_b_amount += token_b_amount;
    Ok(())
}

pub fn remove_liquidity_from_vault(
    vault: &mut Vault,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<(), ProgramError> {
    if vault.token_a_amount < token_a_amount || vault.token_b_amount < token_b_amount {
        return Err(ProgramError::Custom(507)); // Insufficient liquidity
    }
    vault.token_a_amount -= token_a_amount;
    vault.token_b_amount -= token_b_amount;
    Ok(())
}

pub fn swap_tokens_in_vault(
    vault: &mut Vault,
    input_amount: u64,
    min_output_amount: u64,
    is_token_a_to_b: bool,
) -> Result<u64, ProgramError> {
    let output_amount = if is_token_a_to_b {
        calculate_swap_amount(vault.token_a_amount, vault.token_b_amount, input_amount)
    } else {
        calculate_swap_amount(vault.token_b_amount, vault.token_a_amount, input_amount)
    };

    if output_amount < min_output_amount {
        return Err(ProgramError::Custom(508)); // Slippage error
    }

    if is_token_a_to_b {
        vault.token_a_amount -= input_amount;
        vault.token_b_amount += output_amount;
    } else {
        vault.token_a_amount += output_amount;
        vault.token_b_amount -= input_amount;
    }

    Ok(output_amount)
}

pub fn log_swap_status(vault: &Vault) {
    println!("Current Vault Status: Token A: {}, Token B: {}", vault.token_a_amount, vault.token_b_amount);
}



