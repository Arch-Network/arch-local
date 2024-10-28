#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_add_liquidity_to_vault() {
        let mut vault = Vault {
            token_a_amount: 100,
            token_b_amount: 100,
        };
        
        let result = add_liquidity_to_vault(&mut vault, 50, 50);
        assert!(result.is_ok());
        assert_eq!(vault.token_a_amount, 150);
        assert_eq!(vault.token_b_amount, 150);
    }

    #[test]
    fn test_remove_liquidity_from_vault() {
        let mut vault = Vault {
            token_a_amount: 100,
            token_b_amount: 100,
        };

        let result = remove_liquidity_from_vault(&mut vault, 50, 50);
        assert!(result.is_ok());
        assert_eq!(vault.token_a_amount, 50);
        assert_eq!(vault.token_b_amount, 50);
    }

    #[test]
    fn test_insufficient_liquidity_removal() {
        let mut vault = Vault {
            token_a_amount: 100,
            token_b_amount: 100,
        };

        let result = remove_liquidity_from_vault(&mut vault, 150, 150);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::Custom(507));
    }

    #[test]
    fn test_place_limit_order() {
        let owner = Pubkey::new_unique();
        let token_a = Pubkey::new_unique();
        let token_b = Pubkey::new_unique();
        let mut data = vec![0u8; 1000];
        let order_account = create_mock_account(
            &Pubkey::new_unique(),
            &mut data,
            &Pubkey::new_unique(),
            &UtxoMeta::default(),
        );

        let result = place_limit_order(
            &order_account,
            &owner,
            (token_a, token_b),
            100,
            50,
            OrderType::Buy,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_liquidity() {
        let mut liquidity_params = LiquidityParams {
            token_a_amount: 1000,
            token_b_amount: 1000,
            liquidity_amount: 2000,
        };

        let result = contribute_liquidity(
            &mut liquidity_params,
            500,
            500,
        );

        assert!(result.is_ok());
        assert_eq!(liquidity_params.token_a_amount, 1500);
        assert_eq!(liquidity_params.token_b_amount, 1500);
        assert_eq!(liquidity_params.liquidity_amount, 3000);
    }

    #[test]
    fn test_withdraw_liquidity() {
        let mut liquidity_params = LiquidityParams {
            token_a_amount: 1000,
            token_b_amount: 1000,
            liquidity_amount: 2000,
        };

        let result = withdraw_liquidity(
            &mut liquidity_params,
            500,
            500,
        );

        assert!(result.is_ok());
        assert_eq!(liquidity_params.token_a_amount, 500);
        assert_eq!(liquidity_params.token_b_amount, 500);
    }
}
