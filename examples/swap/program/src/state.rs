use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LiquidityParams {
    pub liquidity_amount: u64, // Total liquidity in the pool
    pub token_a_amount: u64,   // Amount of Token A in the pool
    pub token_b_amount: u64,   // Amount of Token B in the pool
}

impl LiquidityParams {
    // Constructor function to initialize a new liquidity pool
    pub fn new(initial_token_a: u64, initial_token_b: u64) -> Self {
        LiquidityParams {
            liquidity_amount: initial_token_a + initial_token_b, // Initially, total liquidity is added
            token_a_amount: initial_token_a,                     // Initial Token A balance
            token_b_amount: initial_token_b,                     // Initial Token B balance
        }
    }

    // Method to get the current liquidity amount
    pub fn get_liquidity_amount(&self) -> u64 {
        self.liquidity_amount
    }
}
