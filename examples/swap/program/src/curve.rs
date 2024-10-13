use std::num::Wrapping;

pub fn calculate_swap_amount(token_a_amount: u64, token_b_amount: u64, input_amount: u64) -> u64 {
    let k = Wrapping(token_a_amount) * Wrapping(token_b_amount);
    let input_amount_wrapping = Wrapping(input_amount);

    // Simple x * y = k calculation
    let output_amount =
        Wrapping(token_b_amount) - (k / (Wrapping(token_a_amount) + input_amount_wrapping));
    output_amount.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_swap_amount() {
        // Example input values
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
