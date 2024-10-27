use crate::state::{LiquidityParams, Vault};
use crate::curve::calculate_swap_amount;
use arch_program::{program_error::ProgramError, pubkey::Pubkey};

/// Represents a trading route through multiple pools or vaults.
pub struct TradingRoute<'a> {
    pub vaults: Vec<&'a Vault>,
}

impl<'a> TradingRoute<'a> {
    /// Calculates the best trading route based on the given input amount and desired output.
    pub fn calculate_best_route(
        &self,
        input_amount: u64,
        desired_output_token: Pubkey,
    ) -> Result<u64, ProgramError> {
        let mut total_output_amount = 0;
        let mut remaining_input = input_amount;

        for vault in &self.vaults {
            if remaining_input == 0 { break; }

            let output_amount = calculate_swap_amount(
                vault.token_a_amount,
                vault.token_b_amount,
                remaining_input,
            );

            if vault.token_b == desired_output_token {
                total_output_amount += output_amount;
                remaining_input -= output_amount;
            }
        }

        if total_output_amount == 0 {
            return Err(ProgramError::Custom(509)); // No suitable route found
        }

        Ok(total_output_amount)
    }

    /// Adjusts trading rates dynamically based on market conditions and pool sizes.
    pub fn adjust_rates(&mut self) {
        for vault in &mut self.vaults {
            let adjustment_factor = self.calculate_adjustment_factor(vault.token_a_amount, vault.token_b_amount);
            vault.token_a_amount = (vault.token_a_amount as f64 * adjustment_factor) as u64;
            vault.token_b_amount = (vault.token_b_amount as f64 * adjustment_factor) as u64;
        }
    }

    /// Calculates an adjustment factor based on the pool sizes.
    fn calculate_adjustment_factor(token_a_amount: u64, token_b_amount: u64) -> f64 {
        let total_tokens = token_a_amount + token_b_amount;
        1.0 + (total_tokens as f64 / 1_000_000.0) // adjustment logic
    }

    // Adding a constructor for easier initialization
    pub fn new(vaults: Vec<&'a Vault>) -> Self {
        TradingRoute { vaults }
    }

  
}
