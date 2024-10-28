use std::num::Wrapping;

pub fn calculate_swap_amount(token_a_amount: u64, token_b_amount: u64, input_amount: u64) -> u64 {
    // Implement dynamic rate adjustment logic here
    let rate_adjustment = calculate_rate_adjustment(token_a_amount, token_b_amount);
    let k = Wrapping(token_a_amount) * Wrapping(token_b_amount);
    let input_amount_wrapping = Wrapping(input_amount) * Wrapping(rate_adjustment);

    // Simple x * y = k calculation
    let output_amount =
        Wrapping(token_b_amount) - (k / (Wrapping(token_a_amount) + input_amount_wrapping));
    output_amount.0
}

fn calculate_rate_adjustment(token_a_amount: u64, token_b_amount: u64) -> u64 {
    if token_a_amount > token_b_amount {
        let excess = token_a_amount - token_b_amount;
        1 + excess / token_b_amount // Increasing multiplier based on the excess proportion
    } else if token_a_amount < token_b_amount {
        let shortage = token_b_amount - token_a_amount;
        1 + shortage / token_a_amount // Increasing multiplier based on the shortage proportion
    } else {
        1 // No adjustment if amounts are equal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_swap_amount() {
        //  input values
        let token_a_reserve = 1000;
        let token_b_reserve = 1000;
        let token_a_amount = 100;

        // Expected output (this should be calculated based on the logic of your function)
        let expected_token_b_amount = 90; // Replace with the correct expected value

        // Call the function
        let result = calculate_swap_amount(token_a_reserve, token_b_reserve, token_a_amount);

        // Assert that the result is as expected
        assert_eq!(result, expected_token_b_amount);
    }
}
